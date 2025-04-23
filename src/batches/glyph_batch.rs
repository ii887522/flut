use crate::{batch::Batch, font_atlas::FontAtlas, models::Glyph};
use ash::{
  Device,
  vk::{
    self, AccessFlags, BorderColor, CommandBuffer, DependencyFlags, DescriptorImageInfo,
    DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo,
    DescriptorType, DeviceAddress, Extent2D, Filter, ImageAspectFlags, ImageLayout,
    ImageMemoryBarrier, ImageSubresourceRange, PipelineStageFlags, RenderPass, Sampler,
    SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode, ShaderStageFlags, WriteDescriptorSet,
  },
};
use gpu_allocator::vulkan::Allocator;
use std::{cell::RefCell, rc::Rc, sync::Arc};

#[repr(C, align(8))]
#[derive(Clone, Copy)]
struct PushConstant {
  camera_position: (f32, f32),
  camera_size: (f32, f32),
  mesh_buffer_addr: DeviceAddress,
}

pub(crate) struct GlyphBatch<'a> {
  batch: Batch<'a, Glyph>,
  pub(crate) descriptor_set_layout: DescriptorSetLayout,
  sampler: Sampler,
  pub(crate) font_atlas: FontAtlas,
  camera_position: (f32, f32),
}

impl GlyphBatch<'_> {
  pub(crate) fn new(
    device: Arc<Device>,
    memory_allocator: Rc<RefCell<Allocator>>,
    cap: usize,
  ) -> Self {
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

    let batch = Batch::new::<PushConstant>(
      device.clone(),
      memory_allocator.clone(),
      cap,
      include_bytes!("../../target/shaders/glyph.vert.spv"),
      include_bytes!("../../target/shaders/glyph.frag.spv"),
      &[descriptor_set_layout],
      "glyph_buffer",
    );

    let sampler_create_info = SamplerCreateInfo {
      mag_filter: Filter::NEAREST,
      min_filter: Filter::NEAREST,
      mipmap_mode: SamplerMipmapMode::NEAREST,
      address_mode_u: SamplerAddressMode::CLAMP_TO_BORDER,
      address_mode_v: SamplerAddressMode::CLAMP_TO_BORDER,
      address_mode_w: SamplerAddressMode::CLAMP_TO_BORDER,
      mip_lod_bias: 0.0,
      anisotropy_enable: vk::FALSE,
      compare_enable: vk::FALSE,
      border_color: BorderColor::FLOAT_OPAQUE_WHITE,
      unnormalized_coordinates: vk::TRUE,
      ..Default::default()
    };

    let sampler = unsafe { device.create_sampler(&sampler_create_info, None).unwrap() };

    let font_atlas = FontAtlas::new(
      device,
      memory_allocator,
      "assets/fonts/arial.ttf",
      48,
      '0'..='9',
      (256, 256),
    );

    Self {
      batch,
      descriptor_set_layout,
      sampler,
      font_atlas,
      camera_position: (0.0, 0.0),
    }
  }

  pub(crate) fn init_descriptor_set(&self, descriptor_set: DescriptorSet) {
    let descriptor_image_info = DescriptorImageInfo {
      sampler: self.sampler,
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
        .batch
        .device
        .update_descriptor_sets(&[image_descriptor_set_write], &[])
    };
  }

  pub(crate) fn on_swapchain_suboptimal(
    &mut self,
    surface_extent: Extent2D,
    render_pass: RenderPass,
  ) {
    self
      .batch
      .on_swapchain_suboptimal(surface_extent, render_pass);
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
      self.batch.device.cmd_pipeline_barrier(
        command_buffer,
        PipelineStageFlags::TOP_OF_PIPE,
        PipelineStageFlags::TRANSFER,
        DependencyFlags::empty(),
        &[],
        &[],
        &[write_image_memory_barrier],
      );

      self.batch.device.cmd_copy_buffer_to_image(
        command_buffer,
        self.font_atlas.image.staging_buffer,
        self.font_atlas.image.image,
        ImageLayout::TRANSFER_DST_OPTIMAL,
        &self.font_atlas.buffer_image_copies,
      );

      self.batch.device.cmd_pipeline_barrier(
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
    let push_const = PushConstant {
      camera_position: self.camera_position,
      camera_size: (surface_extent.width as _, surface_extent.height as _),
      mesh_buffer_addr: self.batch.mesh_buffer_addr,
    };

    self
      .batch
      .record_draw_commands(command_buffer, &[descriptor_set], push_const);
  }

  pub(crate) fn set_camera_position(&mut self, camera_position: (f32, f32)) {
    self.camera_position = camera_position;
  }

  pub(crate) fn add(&mut self, glyph: Glyph) -> u16 {
    self.batch.add(glyph)
  }

  pub(crate) fn batch_add(&mut self, glyphs: Vec<Glyph>) -> Vec<u16> {
    self.batch.batch_add(glyphs)
  }

  pub(crate) fn update(&mut self, id: u16, glyph: Glyph) {
    self.batch.update(id, glyph);
  }

  pub(crate) fn batch_update(&self, ids: &[u16], glyphs: Vec<Glyph>) {
    self.batch.batch_update(ids, glyphs);
  }

  pub(crate) fn remove(&mut self, id: u16) -> Glyph {
    self.batch.remove(id)
  }

  pub(crate) fn batch_remove(&mut self, ids: &[u16]) {
    self.batch.batch_remove(ids);
  }

  pub(crate) fn clear(&mut self) {
    self.batch.clear();
  }
}

impl Drop for GlyphBatch<'_> {
  fn drop(&mut self) {
    unsafe {
      self.batch.device.destroy_sampler(self.sampler, None);

      self
        .batch
        .device
        .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
    }
  }
}
