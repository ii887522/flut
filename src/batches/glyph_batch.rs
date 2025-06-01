use crate::{
  atlases::{FontAtlas, IconAtlas},
  batch::Batch,
  models::Glyph,
};
use ash::{
  Device,
  vk::{
    self, BorderColor, CommandBuffer, CommandPool, DescriptorImageInfo, DescriptorSet,
    DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorType,
    DeviceAddress, Extent2D, Filter, ImageLayout, RenderPass, Sampler, SamplerAddressMode,
    SamplerCreateInfo, SamplerMipmapMode, ShaderStageFlags, WriteDescriptorSet,
  },
};
use gpu_allocator::vulkan::Allocator;
use std::{cell::RefCell, rc::Rc, sync::Arc};

#[repr(C, align(8))]
#[derive(Clone, Copy)]
struct PushConstant {
  camera_position: (f32, f32),
  camera_size: (f32, f32),
  pixel_size: f32,
  pad: f32,
  mesh_buffer_addr: DeviceAddress,
}

pub(crate) struct GlyphBatch {
  batch: Batch<Glyph>,
  pub(crate) descriptor_set_layout: DescriptorSetLayout,
  sampler: Sampler,
  pub(crate) font_atlas: FontAtlas,
  pub(crate) icon_atlas: IconAtlas,
  camera_position: (f32, f32),
}

impl GlyphBatch {
  pub(crate) fn new(
    device: Arc<Device>,
    memory_allocator: Rc<RefCell<Allocator>>,
    transfer_command_pool: CommandPool,
    cap: usize,
  ) -> Self {
    let layout_bindings = [
      DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: DescriptorType::SAMPLER,
        descriptor_count: 1,
        stage_flags: ShaderStageFlags::FRAGMENT,
        ..Default::default()
      },
      DescriptorSetLayoutBinding {
        binding: 1,
        descriptor_type: DescriptorType::SAMPLED_IMAGE,
        descriptor_count: 2,
        stage_flags: ShaderStageFlags::FRAGMENT,
        ..Default::default()
      },
    ];

    let descriptor_set_layout_create_info = DescriptorSetLayoutCreateInfo {
      binding_count: layout_bindings.len() as _,
      p_bindings: layout_bindings.as_ptr(),
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
      mag_filter: Filter::LINEAR,
      min_filter: Filter::LINEAR,
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
      device.clone(),
      memory_allocator.clone(),
      "assets/fonts/arial.ttf",
      48,
      ' '..='~',
      (512, 512),
    );

    let icon_atlas = IconAtlas::new(
      device,
      memory_allocator,
      transfer_command_pool,
      "assets/fonts/MaterialSymbolsOutlined-Regular.ttf",
      64,
      (1024, 1024),
    );

    Self {
      batch,
      descriptor_set_layout,
      sampler,
      font_atlas,
      icon_atlas,
      camera_position: (0.0, 0.0),
    }
  }

  pub(crate) fn init_descriptor_sets(&self, descriptor_sets: &[DescriptorSet]) {
    let descriptor_sampler_info = DescriptorImageInfo {
      sampler: self.sampler,
      ..Default::default()
    };

    let descriptor_image_infos = self
      .icon_atlas
      .image
      .views
      .iter()
      .map(|&icon_image_view| {
        [
          DescriptorImageInfo {
            image_view: self.font_atlas.image.view,
            image_layout: ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            ..Default::default()
          },
          DescriptorImageInfo {
            image_view: icon_image_view,
            image_layout: ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            ..Default::default()
          },
        ]
      })
      .collect::<Vec<_>>();

    let descriptor_writes = descriptor_image_infos
      .iter()
      .zip(descriptor_sets.iter())
      .flat_map(|(descriptor_image_infos, &descriptor_set)| {
        [
          WriteDescriptorSet {
            dst_set: descriptor_set,
            dst_binding: 0,
            dst_array_element: 0,
            descriptor_count: 1,
            descriptor_type: DescriptorType::SAMPLER,
            p_image_info: &descriptor_sampler_info,
            ..Default::default()
          },
          WriteDescriptorSet {
            dst_set: descriptor_set,
            dst_binding: 1,
            dst_array_element: 0,
            descriptor_count: descriptor_image_infos.len() as _,
            descriptor_type: DescriptorType::SAMPLED_IMAGE,
            p_image_info: descriptor_image_infos.as_ptr(),
            ..Default::default()
          },
        ]
      })
      .collect::<Vec<_>>();

    unsafe {
      self
        .batch
        .device
        .update_descriptor_sets(&descriptor_writes, &[]);
    }
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

  pub(crate) fn record_draw_commands(
    &self,
    command_buffer: CommandBuffer,
    descriptor_set: DescriptorSet,
    surface_extent: Extent2D,
    pixel_size: f32,
  ) {
    let push_const = PushConstant {
      camera_position: self.camera_position,
      camera_size: (surface_extent.width as _, surface_extent.height as _),
      pixel_size,
      pad: 0.0,
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

impl Drop for GlyphBatch {
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
