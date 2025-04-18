use crate::{
  buffers::StreamBuffer,
  collections::SparseVec,
  font_atlas::FontAtlas,
  models::Glyph,
  pipelines::{GlyphPipeline, GlyphPushConstant},
  shaders::{GlyphFragShader, GlyphVertShader},
};
use ash::{
  Device,
  vk::{
    self, AccessFlags, BufferDeviceAddressInfo, BufferUsageFlags, CommandBuffer, DependencyFlags,
    DescriptorImageInfo, DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutBinding,
    DescriptorSetLayoutCreateInfo, DescriptorType, DeviceAddress, Extent2D, ImageAspectFlags,
    ImageLayout, ImageMemoryBarrier, ImageSubresourceRange, PipelineBindPoint, PipelineLayout,
    PipelineLayoutCreateInfo, PipelineStageFlags, PushConstantRange, RenderPass, ShaderStageFlags,
    WriteDescriptorSet,
  },
};
use atomic_refcell::AtomicRefCell;
use gpu_allocator::vulkan::Allocator;
use rayon::prelude::*;
use std::{cell::RefCell, mem, ptr, rc::Rc};

pub(crate) struct GlyphBatch<'a> {
  device: Rc<Device>,
  vert_shader: GlyphVertShader<'a>,
  frag_shader: GlyphFragShader<'a>,
  pub(crate) descriptor_set_layout: DescriptorSetLayout,
  pipeline_layout: PipelineLayout,
  pub(crate) mesh_buffer: StreamBuffer,
  mesh_buffer_addr: DeviceAddress,
  pub(crate) font_atlas: FontAtlas,
  pub(crate) pipeline: Option<GlyphPipeline>,
  glyphs: SparseVec<Glyph>,
}

impl GlyphBatch<'_> {
  pub(crate) fn new(
    device: Rc<Device>,
    memory_allocator: Rc<RefCell<Allocator>>,
    cap: usize,
  ) -> Self {
    let vert_shader = GlyphVertShader::new(device.clone());
    let frag_shader = GlyphFragShader::new(device.clone());

    let sampler_layout_binding = DescriptorSetLayoutBinding {
      binding: 0,
      descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
      descriptor_count: 1,
      stage_flags: ShaderStageFlags::FRAGMENT,
      ..Default::default()
    };

    let descriptor_set_layout_create_info = DescriptorSetLayoutCreateInfo {
      binding_count: 1,
      p_bindings: &sampler_layout_binding,
      ..Default::default()
    };

    let descriptor_set_layout = unsafe {
      device
        .create_descriptor_set_layout(&descriptor_set_layout_create_info, None)
        .unwrap()
    };

    let push_const_range = PushConstantRange {
      stage_flags: ShaderStageFlags::VERTEX,
      size: mem::size_of::<GlyphPushConstant>() as _,
      ..Default::default()
    };

    let pipeline_layout_create_info = PipelineLayoutCreateInfo {
      set_layout_count: 1,
      p_set_layouts: &descriptor_set_layout,
      push_constant_range_count: 1,
      p_push_constant_ranges: &push_const_range,
      ..Default::default()
    };

    let pipeline_layout = unsafe {
      device
        .create_pipeline_layout(&pipeline_layout_create_info, None)
        .unwrap()
    };

    let mesh_buffer = StreamBuffer::new(
      device.clone(),
      memory_allocator.clone(),
      "glyph_mesh_buffer",
      (cap * mem::size_of::<Glyph>()) as _,
      BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
    );

    let mesh_buffer_addr_info = BufferDeviceAddressInfo {
      buffer: mesh_buffer.buffer,
      ..Default::default()
    };

    let mesh_buffer_addr = unsafe { device.get_buffer_device_address(&mesh_buffer_addr_info) };

    let font_atlas = FontAtlas::new(
      device.clone(),
      memory_allocator.clone(),
      "assets/fonts/arial.ttf",
      48,
      '0'..='9',
      (256, 256),
    );

    Self {
      device,
      vert_shader,
      frag_shader,
      descriptor_set_layout,
      pipeline_layout,
      mesh_buffer,
      mesh_buffer_addr,
      font_atlas,
      pipeline: None,
      glyphs: SparseVec::with_capacity(cap),
    }
  }

  pub(crate) fn init_descriptor_set(&self, descriptor_set: DescriptorSet) {
    let descriptor_image_info = DescriptorImageInfo {
      sampler: self.frag_shader.sampler,
      image_view: self.font_atlas.image.view,
      image_layout: ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    };

    let image_descriptor_set_write = WriteDescriptorSet {
      dst_set: descriptor_set,
      dst_binding: 0,
      dst_array_element: 0,
      descriptor_count: 1,
      descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
      p_image_info: &descriptor_image_info,
      ..Default::default()
    };

    unsafe {
      self
        .device
        .update_descriptor_sets(&[image_descriptor_set_write], &[])
    };
  }

  pub(crate) fn on_swapchain_suboptimal(
    &mut self,
    surface_extent: Extent2D,
    render_pass: RenderPass,
  ) {
    let glyph_pipeline = GlyphPipeline::new(
      self.device.clone(),
      surface_extent,
      &self.vert_shader,
      &self.frag_shader,
      self.pipeline_layout,
      render_pass,
      self.pipeline.as_ref(),
    );

    drop(mem::take(&mut self.pipeline));
    self.pipeline = Some(glyph_pipeline);
  }

  pub(crate) fn record_init_commands(&self, command_buffer: CommandBuffer) {
    let write_image_memory_barrier = ImageMemoryBarrier {
      src_access_mask: AccessFlags::NONE,
      dst_access_mask: AccessFlags::TRANSFER_WRITE,
      old_layout: ImageLayout::UNDEFINED,
      new_layout: ImageLayout::TRANSFER_DST_OPTIMAL,
      src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
      dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
      image: self.font_atlas.image.image,
      subresource_range: ImageSubresourceRange {
        aspect_mask: ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
      },
      ..Default::default()
    };

    let read_image_memory_barrier = ImageMemoryBarrier {
      src_access_mask: AccessFlags::TRANSFER_WRITE,
      dst_access_mask: AccessFlags::SHADER_READ,
      old_layout: ImageLayout::TRANSFER_DST_OPTIMAL,
      new_layout: ImageLayout::SHADER_READ_ONLY_OPTIMAL,
      src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
      dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
      image: self.font_atlas.image.image,
      subresource_range: ImageSubresourceRange {
        aspect_mask: ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
      },
      ..Default::default()
    };

    unsafe {
      self.device.cmd_pipeline_barrier(
        command_buffer,
        PipelineStageFlags::TOP_OF_PIPE,
        PipelineStageFlags::TRANSFER,
        DependencyFlags::empty(),
        &[],
        &[],
        &[write_image_memory_barrier],
      );

      self.device.cmd_copy_buffer_to_image(
        command_buffer,
        self.font_atlas.image.staging_buffer,
        self.font_atlas.image.image,
        ImageLayout::TRANSFER_DST_OPTIMAL,
        &self.font_atlas.buffer_image_copies,
      );

      self.device.cmd_pipeline_barrier(
        command_buffer,
        PipelineStageFlags::TRANSFER,
        PipelineStageFlags::FRAGMENT_SHADER,
        DependencyFlags::empty(),
        &[],
        &[],
        &[read_image_memory_barrier],
      );
    }
  }

  pub(crate) fn record_draw_commands(
    &self,
    command_buffer: CommandBuffer,
    descriptor_set: DescriptorSet,
    surface_extent: Extent2D,
  ) {
    if self.glyphs.is_empty() {
      return;
    }

    let glyph_push_const = GlyphPushConstant {
      camera_size: (surface_extent.width as _, surface_extent.height as _),
      mesh_buffer_addr: self.mesh_buffer_addr,
    };

    let pipeline = self.pipeline.as_ref().unwrap();

    unsafe {
      self.device.cmd_bind_pipeline(
        command_buffer,
        PipelineBindPoint::GRAPHICS,
        pipeline.pipeline,
      );

      self.device.cmd_bind_descriptor_sets(
        command_buffer,
        PipelineBindPoint::GRAPHICS,
        self.pipeline_layout,
        0,
        &[descriptor_set],
        &[],
      );

      self.device.cmd_push_constants(
        command_buffer,
        self.pipeline_layout,
        ShaderStageFlags::VERTEX,
        0,
        crate::as_bytes(&glyph_push_const),
      );

      self
        .device
        .cmd_draw(command_buffer, (6 * self.glyphs.len()) as _, 1, 0, 0);
    }
  }

  pub(crate) fn add(&mut self, glyph: Glyph) -> u16 {
    let mapped_mesh_alloc = self.mesh_buffer.alloc.mapped_ptr().unwrap();

    unsafe {
      ptr::copy_nonoverlapping(
        &glyph,
        (mapped_mesh_alloc.as_ptr() as *mut Glyph).add(self.glyphs.len()),
        1,
      );
    }

    self.glyphs.push(glyph)
  }

  pub(crate) fn batch_add(&mut self, glyphs: Vec<Glyph>) -> Vec<u16> {
    let mapped_mesh_alloc = self.mesh_buffer.alloc.mapped_ptr().unwrap();

    unsafe {
      ptr::copy_nonoverlapping(
        glyphs.as_ptr(),
        (mapped_mesh_alloc.as_ptr() as *mut Glyph).add(self.glyphs.len()),
        glyphs.len(),
      );
    }

    self.glyphs.par_extend(glyphs)
  }

  pub(crate) fn update(&mut self, id: u16, glyph: Glyph) {
    let mapped_mesh_alloc = self.mesh_buffer.alloc.mapped_ptr().unwrap();

    unsafe {
      ptr::copy_nonoverlapping(
        &glyph,
        (mapped_mesh_alloc.as_ptr() as *mut Glyph).add(self.glyphs.get_dense_index(id) as _),
        1,
      );
    }

    self.glyphs[id] = AtomicRefCell::new(glyph);
  }

  pub(crate) fn batch_update(&mut self, ids: &[u16], glyphs: Vec<Glyph>) {
    ids
      .par_iter()
      .zip(glyphs.par_iter())
      .for_each(|(&id, glyph)| unsafe {
        let mapped_mesh_alloc = self.mesh_buffer.alloc.mapped_ptr().unwrap();

        ptr::copy_nonoverlapping(
          glyph,
          (mapped_mesh_alloc.as_ptr() as *mut Glyph).add(self.glyphs.get_dense_index(id) as _),
          1,
        );
      });

    self.glyphs.par_set(ids, glyphs);
  }

  pub(crate) fn remove(&mut self, id: u16) -> Glyph {
    let index = self.glyphs.get_dense_index(id);
    let result = self.glyphs.remove(id);

    let Some(glyph) = self.glyphs.get_by_dense_index(index) else {
      return result;
    };

    let glyph = glyph.borrow();
    let mapped_mesh_alloc = self.mesh_buffer.alloc.mapped_ptr().unwrap();

    unsafe {
      ptr::copy_nonoverlapping(
        &*glyph,
        (mapped_mesh_alloc.as_ptr() as *mut Glyph).add(index as _),
        1,
      );
    }

    result
  }

  pub(crate) fn batch_remove(&mut self, ids: &[u16]) {
    let indices = ids
      .par_iter()
      .map(|&id| self.glyphs.get_dense_index(id))
      .collect::<Vec<_>>();

    self.glyphs.par_remove(ids);

    indices
      .into_par_iter()
      .filter_map(|index| {
        self
          .glyphs
          .get_by_dense_index(index)
          .map(|glyph| (index, glyph))
      })
      .for_each(|(index, glyph)| {
        let glyph = glyph.borrow();
        let mapped_mesh_alloc = self.mesh_buffer.alloc.mapped_ptr().unwrap();

        unsafe {
          ptr::copy_nonoverlapping(
            &*glyph,
            (mapped_mesh_alloc.as_ptr() as *mut Glyph).add(index as _),
            1,
          );
        }
      });
  }

  pub(crate) fn clear(&mut self) {
    self.glyphs.clear();
  }
}

impl Drop for GlyphBatch<'_> {
  fn drop(&mut self) {
    unsafe {
      self
        .device
        .destroy_pipeline_layout(self.pipeline_layout, None);

      self
        .device
        .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
    }
  }
}
