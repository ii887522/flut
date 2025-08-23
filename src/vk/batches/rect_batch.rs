use crate::{
  IdGenerator, Write,
  collections::SparseSet,
  models::{Anchor, GlyphMetrics, Icon, IconName, Rect, Text, TextId},
  vk::{Device, DynamicImage, GraphicsPipeline, StreamBuffer, graphics_pipeline},
};
use ash::vk;
use rayon::prelude::*;
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use sdl2::{pixels::Color, ttf::Font};
use std::{collections::VecDeque, mem, ptr, rc::Rc};

#[repr(C, align(8))]
struct RectPushConst {
  rect_buf_addr: vk::DeviceAddress,
  cam_position: (f32, f32),
  cam_size: (f32, f32),
}

impl RectPushConst {
  const fn new(
    rect_buf_addr: vk::DeviceAddress,
    cam_position: (f32, f32),
    cam_size: (u32, u32),
  ) -> Self {
    Self {
      rect_buf_addr,
      cam_position,
      cam_size: (cam_size.0 as _, cam_size.1 as _),
    }
  }
}

pub(crate) struct Creating {
  pipeline: GraphicsPipeline<graphics_pipeline::Creating>,
}

pub(crate) struct Created {
  pipeline: GraphicsPipeline<graphics_pipeline::Created>,
}

pub(crate) struct RectBatch<'ttf, State> {
  device: Rc<Device>,
  sampler: vk::Sampler,
  descriptor_set_layout: vk::DescriptorSetLayout,
  descriptor_sets: Box<[vk::DescriptorSet]>,
  icon_font: Font<'ttf, 'static>,
  icon_atlas: DynamicImage,
  rects: SparseSet<Rect>,
  icon_index_generator: IdGenerator,
  writes_queues: VecDeque<Vec<Write>>,
  cam_position: (f32, f32),
  char_to_glyph_metrics: FxHashMap<char, GlyphMetrics>,
  icon_name_to_glyph_metrics: FxHashMap<IconName, GlyphMetrics>,
  text_ids: SparseSet<TextId>,
  id_to_icon_name: FxHashMap<u32, IconName>,
  icon_pixels: Vec<u8>,
  icon_regions_queues: VecDeque<Vec<vk::BufferImageCopy2<'static>>>,
  flush_icons_finished_semaphores: Box<[vk::Semaphore]>,
  flush_icons_fences: Box<[vk::Fence]>,
  render_finished_semaphore_sets: Box<[FxHashSet<vk::Semaphore>]>,
  state: State,
}

impl<'ttf> RectBatch<'ttf, Creating> {
  pub(crate) fn new(
    vk_device: Rc<Device>,
    descriptor_pool: vk::DescriptorPool,
    glyph_atlas_view: vk::ImageView,
    icon_atlas: DynamicImage,
    icon_font: Font<'ttf, 'static>,
    char_to_glyph_metrics: FxHashMap<char, GlyphMetrics>,
    cap: usize,
  ) -> Self {
    let device = vk_device.get();

    let sampler_create_info = vk::SamplerCreateInfo {
      mag_filter: vk::Filter::LINEAR,
      min_filter: vk::Filter::LINEAR,
      mipmap_mode: vk::SamplerMipmapMode::NEAREST,
      address_mode_u: vk::SamplerAddressMode::CLAMP_TO_BORDER,
      address_mode_v: vk::SamplerAddressMode::CLAMP_TO_BORDER,
      address_mode_w: vk::SamplerAddressMode::CLAMP_TO_BORDER,
      mip_lod_bias: 0.0,
      anisotropy_enable: vk::FALSE,
      compare_enable: vk::FALSE,
      compare_op: vk::CompareOp::ALWAYS,
      min_lod: 0.0,
      max_lod: vk::LOD_CLAMP_NONE,
      border_color: vk::BorderColor::FLOAT_TRANSPARENT_BLACK,
      unnormalized_coordinates: vk::FALSE,
      ..Default::default()
    };

    let sampler = unsafe { device.create_sampler(&sampler_create_info, None).unwrap() };

    let descriptor_image_infos = icon_atlas
      .get_views()
      .iter()
      .map(|&icon_atlas_view| {
        [
          vk::DescriptorImageInfo {
            image_view: glyph_atlas_view,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            ..Default::default()
          },
          vk::DescriptorImageInfo {
            image_view: icon_atlas_view,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            ..Default::default()
          },
        ]
      })
      .collect::<Box<_>>();

    let immutable_samplers = [sampler, sampler];

    let descriptor_set_layout_bindings = [vk::DescriptorSetLayoutBinding {
      binding: 0,
      descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
      descriptor_count: descriptor_image_infos[0].len() as _,
      stage_flags: vk::ShaderStageFlags::FRAGMENT,
      p_immutable_samplers: immutable_samplers.as_ptr(),
      ..Default::default()
    }];

    let descriptor_set_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
      binding_count: descriptor_set_layout_bindings.len() as _,
      p_bindings: descriptor_set_layout_bindings.as_ptr(),
      ..Default::default()
    };

    let descriptor_set_layout = unsafe {
      device
        .create_descriptor_set_layout(&descriptor_set_layout_create_info, None)
        .unwrap()
    };

    let descriptor_set_layouts = [descriptor_set_layout; crate::consts::SUB_DYNAMIC_BUFFER_COUNT];

    let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo {
      descriptor_pool,
      descriptor_set_count: descriptor_set_layouts.len() as _,
      p_set_layouts: descriptor_set_layouts.as_ptr(),
      ..Default::default()
    };

    let descriptor_sets = unsafe {
      device
        .allocate_descriptor_sets(&descriptor_set_alloc_info)
        .unwrap()
    };

    let descriptor_set_writes = descriptor_image_infos
      .iter()
      .zip(descriptor_sets.iter())
      .map(
        |(descriptor_image_infos, &descriptor_set)| vk::WriteDescriptorSet {
          dst_set: descriptor_set,
          dst_binding: 0,
          dst_array_element: 0,
          descriptor_count: descriptor_image_infos.len() as _,
          descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
          p_image_info: descriptor_image_infos.as_ptr(),
          ..Default::default()
        },
      )
      .collect::<Box<_>>();

    unsafe {
      device.update_descriptor_sets(&descriptor_set_writes, &[]);
    }

    let pipeline = GraphicsPipeline::new(
      vk_device.clone(),
      include_bytes!("../../../target/spv/rect.vert.spv"),
      include_bytes!("../../../target/spv/rect.frag.spv"),
      &[descriptor_set_layout],
      &[vk::PushConstantRange {
        stage_flags: vk::ShaderStageFlags::VERTEX,
        size: mem::size_of::<RectPushConst>() as _,
        ..Default::default()
      }],
    );

    let (flush_icons_finished_semaphores, flush_icons_fences): (Vec<_>, Vec<_>) = (0
      ..crate::consts::SUB_DYNAMIC_BUFFER_COUNT)
      .map(|_| {
        let semaphore_create_info = vk::SemaphoreCreateInfo::default();

        let fence_create_info = vk::FenceCreateInfo {
          flags: vk::FenceCreateFlags::SIGNALED,
          ..Default::default()
        };

        unsafe {
          (
            device
              .create_semaphore(&semaphore_create_info, None)
              .unwrap(),
            device.create_fence(&fence_create_info, None).unwrap(),
          )
        }
      })
      .unzip();

    let render_finished_semaphore_sets = (0..crate::consts::SUB_DYNAMIC_BUFFER_COUNT)
      .map(|_| {
        FxHashSet::with_capacity_and_hasher(crate::consts::MAX_IN_FLIGHT_FRAME_COUNT, FxBuildHasher)
      })
      .collect::<Box<_>>();

    Self {
      device: vk_device,
      sampler,
      descriptor_set_layout,
      descriptor_sets: descriptor_sets.into_boxed_slice(),
      icon_font,
      icon_atlas,
      rects: SparseSet::with_capacity(cap),
      icon_index_generator: IdGenerator::new(),
      writes_queues: VecDeque::from_iter([vec![]]),
      cam_position: (0.0, 0.0),
      char_to_glyph_metrics,
      icon_name_to_glyph_metrics: FxHashMap::with_capacity_and_hasher(
        crate::consts::ICON_COUNT as _,
        FxBuildHasher,
      ),
      text_ids: SparseSet::new(),
      id_to_icon_name: FxHashMap::with_hasher(FxBuildHasher),
      icon_pixels: vec![],
      icon_regions_queues: VecDeque::from_iter([vec![]]),
      flush_icons_finished_semaphores: flush_icons_finished_semaphores.into_boxed_slice(),
      flush_icons_fences: flush_icons_fences.into_boxed_slice(),
      render_finished_semaphore_sets,
      state: Creating { pipeline },
    }
  }

  pub(crate) fn finish(
    self,
    render_pass: vk::RenderPass,
    swapchain_image_extent: vk::Extent2D,
  ) -> RectBatch<'ttf, Created> {
    let pipeline = self
      .state
      .pipeline
      .finish(render_pass, swapchain_image_extent);

    RectBatch {
      device: self.device,
      sampler: self.sampler,
      descriptor_set_layout: self.descriptor_set_layout,
      descriptor_sets: self.descriptor_sets,
      icon_font: self.icon_font,
      icon_atlas: self.icon_atlas,
      rects: self.rects,
      icon_index_generator: self.icon_index_generator,
      writes_queues: self.writes_queues,
      cam_position: self.cam_position,
      char_to_glyph_metrics: self.char_to_glyph_metrics,
      icon_name_to_glyph_metrics: self.icon_name_to_glyph_metrics,
      text_ids: self.text_ids,
      id_to_icon_name: self.id_to_icon_name,
      icon_pixels: self.icon_pixels,
      icon_regions_queues: self.icon_regions_queues,
      flush_icons_fences: self.flush_icons_fences,
      flush_icons_finished_semaphores: self.flush_icons_finished_semaphores,
      render_finished_semaphore_sets: self.render_finished_semaphore_sets,
      state: Created { pipeline },
    }
  }

  pub(crate) fn drop(self) {
    let device = self.device.get();
    self.state.pipeline.drop();

    unsafe {
      self
        .flush_icons_fences
        .into_iter()
        .for_each(|fence| device.destroy_fence(fence, None));

      self
        .flush_icons_finished_semaphores
        .into_iter()
        .for_each(|semaphore| device.destroy_semaphore(semaphore, None));
    }

    self.icon_atlas.drop();

    unsafe {
      device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
      device.destroy_sampler(self.sampler, None);
    }
  }
}

impl<'ttf> RectBatch<'ttf, Created> {
  pub(crate) fn flush_icons(
    &mut self,
    transfer_queue: vk::Queue,
    transfer_command_pools: &[vk::CommandPool],
    transfer_command_buffers: &[vk::CommandBuffer],
    transfer_queue_family_index: u32,
    graphics_queue_family_index: u32,
  ) -> Option<vk::Semaphore> {
    let icon_regions = self.icon_regions_queues.back().unwrap();

    if icon_regions.is_empty() {
      return None;
    }

    let device = self.device.get();
    let transfer_command_pool = transfer_command_pools[self.icon_atlas.get_write_view_index()];
    let transfer_command_buffer = transfer_command_buffers[self.icon_atlas.get_write_view_index()];
    let finished_semaphore =
      self.flush_icons_finished_semaphores[self.icon_atlas.get_write_view_index()];
    let finished_fence = self.flush_icons_fences[self.icon_atlas.get_write_view_index()];

    let (render_finished_semaphores, transfer_wait_dst_stage_masks): (Vec<_>, Vec<_>) = self
      .render_finished_semaphore_sets[self.icon_atlas.get_write_view_index()]
    .iter()
    .map(|&semaphore| (semaphore, vk::PipelineStageFlags::TRANSFER))
    .unzip();

    let transfer_command_buffer_begin_info = vk::CommandBufferBeginInfo {
      flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
      ..Default::default()
    };

    let transfer_queue_submit_info = vk::SubmitInfo {
      wait_semaphore_count: render_finished_semaphores.len() as _,
      p_wait_semaphores: render_finished_semaphores.as_ptr(),
      p_wait_dst_stage_mask: transfer_wait_dst_stage_masks.as_ptr(),
      command_buffer_count: 1,
      p_command_buffers: &transfer_command_buffer,
      signal_semaphore_count: 1,
      p_signal_semaphores: &finished_semaphore,
      ..Default::default()
    };

    unsafe {
      device
        .wait_for_fences(&[finished_fence], true, u64::MAX)
        .unwrap();

      ptr::copy_nonoverlapping(
        self.icon_pixels.as_ptr(),
        self.icon_atlas.get_staging_mapped_data() as *mut _,
        self.icon_pixels.len(),
      );

      device
        .reset_command_pool(transfer_command_pool, vk::CommandPoolResetFlags::empty())
        .unwrap();

      device
        .begin_command_buffer(transfer_command_buffer, &transfer_command_buffer_begin_info)
        .unwrap();

      self.icon_atlas.record_flush_commands(
        transfer_command_buffer,
        &mut self.icon_regions_queues,
        transfer_queue_family_index,
        graphics_queue_family_index,
      );

      device.end_command_buffer(transfer_command_buffer).unwrap();
      device.reset_fences(&[finished_fence]).unwrap();

      device
        .queue_submit(
          transfer_queue,
          &[transfer_queue_submit_info],
          finished_fence,
        )
        .unwrap();
    }

    if self.icon_regions_queues.len() >= crate::consts::SUB_DYNAMIC_BUFFER_COUNT {
      self.icon_regions_queues.pop_front();
    }

    self.icon_regions_queues.push_back(vec![]);
    Some(finished_semaphore)
  }

  pub(crate) fn on_render_finished_semaphore_signaled(
    &mut self,
    render_finished_semaphore: vk::Semaphore,
  ) {
    for render_finished_semaphores in &mut self.render_finished_semaphore_sets {
      render_finished_semaphores.remove(&render_finished_semaphore);
    }
  }

  pub(crate) fn record_draw_commands(
    &mut self,
    command_buffer: vk::CommandBuffer,
    instance_buffer: &StreamBuffer,
    swapchain_image_extent: vk::Extent2D,
    render_finished_semaphore: vk::Semaphore,
  ) {
    crate::flush_writes(
      &mut self.writes_queues,
      self.rects.get_dense_ptr(),
      instance_buffer.get_mapped_data(),
    );

    let rect_push_const = RectPushConst::new(
      instance_buffer.get_addr(),
      self.cam_position,
      (swapchain_image_extent.width, swapchain_image_extent.height),
    );

    let device = self.device.get();
    let descriptor_set = self.descriptor_sets[self.icon_atlas.get_read_view_index()];

    unsafe {
      device.cmd_bind_pipeline(
        command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        self.state.pipeline.get(),
      );

      device.cmd_bind_descriptor_sets(
        command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        self.state.pipeline.get_layout(),
        0,
        &[descriptor_set],
        &[],
      );

      device.cmd_push_constants(
        command_buffer,
        self.state.pipeline.get_layout(),
        vk::ShaderStageFlags::VERTEX,
        0,
        crate::as_bytes(&rect_push_const),
      );

      device.cmd_draw(command_buffer, (self.rects.len() * 6) as _, 1, 0, 0);
    }

    self.render_finished_semaphore_sets[self.icon_atlas.get_read_view_index()]
      .insert(render_finished_semaphore);
  }

  pub(crate) fn on_swapchain_suboptimal(self) -> RectBatch<'ttf, Creating> {
    let pipeline = self.state.pipeline.on_swapchain_suboptimal();

    RectBatch {
      device: self.device,
      sampler: self.sampler,
      descriptor_set_layout: self.descriptor_set_layout,
      descriptor_sets: self.descriptor_sets,
      icon_font: self.icon_font,
      icon_atlas: self.icon_atlas,
      rects: self.rects,
      icon_index_generator: self.icon_index_generator,
      writes_queues: self.writes_queues,
      cam_position: self.cam_position,
      char_to_glyph_metrics: self.char_to_glyph_metrics,
      icon_name_to_glyph_metrics: self.icon_name_to_glyph_metrics,
      text_ids: self.text_ids,
      id_to_icon_name: self.id_to_icon_name,
      icon_pixels: self.icon_pixels,
      icon_regions_queues: self.icon_regions_queues,
      flush_icons_fences: self.flush_icons_fences,
      flush_icons_finished_semaphores: self.flush_icons_finished_semaphores,
      render_finished_semaphore_sets: self.render_finished_semaphore_sets,
      state: Creating { pipeline },
    }
  }

  pub(crate) fn drop(self) {
    let device = self.device.get();
    self.state.pipeline.drop();

    unsafe {
      self
        .flush_icons_fences
        .into_iter()
        .for_each(|fence| device.destroy_fence(fence, None));

      self
        .flush_icons_finished_semaphores
        .into_iter()
        .for_each(|semaphore| device.destroy_semaphore(semaphore, None));
    }

    self.icon_atlas.drop();

    unsafe {
      device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
      device.destroy_sampler(self.sampler, None);
    }
  }
}

impl<State> RectBatch<'_, State> {
  fn calc_glyph_metrics(&mut self, icon_name: IconName) -> GlyphMetrics {
    if let Some(&glyph_metrics) = self.icon_name_to_glyph_metrics.get(&icon_name) {
      glyph_metrics
    } else {
      let codepoint = char::from_u32(icon_name as _).unwrap();

      let glyph = self
        .icon_font
        .render_char(codepoint)
        .shaded(Color::WHITE, Color::BLACK)
        .unwrap();

      let icon_index = self.icon_index_generator.generate();

      let icon_position = (
        icon_index as u32 % crate::consts::ICON_COL_COUNT
          * (glyph.width() + crate::consts::ICON_GAP),
        icon_index as u32 / crate::consts::ICON_COL_COUNT
          * (glyph.height() + crate::consts::ICON_GAP),
      );

      let region = vk::BufferImageCopy2 {
        buffer_offset: (self.icon_atlas.get_staging_buffer_offset() + self.icon_pixels.len()) as _,
        buffer_row_length: glyph.pitch(),
        buffer_image_height: glyph.height(),
        image_subresource: vk::ImageSubresourceLayers {
          aspect_mask: vk::ImageAspectFlags::COLOR,
          mip_level: 0,
          base_array_layer: self.icon_atlas.get_write_layer_index() as _,
          layer_count: 1,
        },
        image_offset: vk::Offset3D {
          x: icon_position.0 as _,
          y: icon_position.1 as _,
          z: 0,
        },
        image_extent: vk::Extent3D {
          width: glyph.width(),
          height: glyph.height(),
          depth: 1,
        },
        ..Default::default()
      };

      let glyph_metrics = GlyphMetrics {
        position: (icon_position.0 as _, icon_position.1 as _, 1.0),
        size: (glyph.width() as _, glyph.height() as _),
        advance: glyph.width() as _,
      };

      self.icon_pixels.par_extend(glyph.without_lock().unwrap());

      let icon_regions = self.icon_regions_queues.back_mut().unwrap();
      icon_regions.push(region);

      self
        .icon_name_to_glyph_metrics
        .insert(icon_name, glyph_metrics);

      glyph_metrics
    }
  }

  pub(crate) fn set_cam_position(&mut self, cam_position: (f32, f32)) {
    self.cam_position = cam_position;
  }

  pub(crate) fn add_rect(&mut self, rect: Rect) -> u32 {
    let writes = self.writes_queues.back_mut().unwrap();

    writes.push(Write {
      from: self.rects.len() as _,
      size: 1,
    });

    self.rects.push(rect)
  }

  pub(crate) fn add_rects(&mut self, rects: Vec<Rect>) -> Box<[u32]> {
    let writes = self.writes_queues.back_mut().unwrap();

    writes.push(Write {
      from: self.rects.len() as _,
      size: rects.len() as _,
    });

    self.rects.par_extend(rects)
  }

  pub(crate) fn update_rect(&mut self, id: u32, rect: Rect) {
    let update_resp = self.rects.update(id, rect);
    let writes = self.writes_queues.back_mut().unwrap();

    writes.push(Write {
      from: update_resp.dense_index,
      size: 1,
    });
  }

  pub(crate) fn update_rects(&mut self, rects: FxHashMap<u32, Rect>) {
    let update_resps = self.rects.par_update(rects);
    let writes = self.writes_queues.back_mut().unwrap();

    writes.par_extend(update_resps.into_par_iter().map(|update_resp| Write {
      from: update_resp.dense_index,
      size: 1,
    }));
  }

  pub(crate) fn remove_rect(&mut self, id: u32) -> Option<Rect> {
    let remove_resp = self.rects.remove(id)?;

    if remove_resp.dense_index >= self.rects.len() as _ {
      return Some(remove_resp.item);
    }

    let writes = self.writes_queues.back_mut().unwrap();

    writes.push(Write {
      from: remove_resp.dense_index,
      size: 1,
    });

    Some(remove_resp.item)
  }

  pub(crate) fn remove_rects(&mut self, ids: FxHashSet<u32>) -> Box<[(u32, Rect)]> {
    let remove_resps = self.rects.par_remove(ids);

    let rects = remove_resps
      .par_iter()
      .map(|remove_resp| (remove_resp.id, remove_resp.item))
      .collect::<Box<_>>();

    let writes = self.writes_queues.back_mut().unwrap();

    writes.par_extend(remove_resps.into_par_iter().filter_map(|remove_resp| {
      if remove_resp.dense_index < self.rects.len() as _ {
        Some(Write {
          from: remove_resp.dense_index,
          size: 1,
        })
      } else {
        None
      }
    }));

    rects
  }

  pub(crate) fn add_text(&mut self, text: Text) -> u32 {
    let scale = text.font_size as f32 / crate::consts::FONT_SIZE as f32;
    let mut glyph_position = text.position;
    let mut max_glyph_height = 0f32;

    let rects = text
      .text
      .chars()
      .map(|ch| {
        let glyph_metrics = &self.char_to_glyph_metrics[&ch];

        let rect = Rect::new()
          .position(glyph_position)
          .size((glyph_metrics.size.0 * scale, glyph_metrics.size.1 * scale))
          .color(text.color)
          .tex_position((
            glyph_metrics.position.0 / crate::consts::GLYPH_ATLAS_SIZE.0 as f32,
            glyph_metrics.position.1 / crate::consts::GLYPH_ATLAS_SIZE.1 as f32,
            glyph_metrics.position.2,
          ))
          .tex_size((
            glyph_metrics.size.0 / crate::consts::GLYPH_ATLAS_SIZE.0 as f32,
            glyph_metrics.size.1 / crate::consts::GLYPH_ATLAS_SIZE.1 as f32,
          ))
          .call();

        glyph_position.0 += glyph_metrics.advance as f32 * scale;
        max_glyph_height = max_glyph_height.max(glyph_metrics.size.1 * scale);
        rect
      })
      .collect::<Box<_>>();

    let last_glyph = rects.last().unwrap();

    let text_size = (
      last_glyph.position.0 + last_glyph.size.0 - text.position.0,
      max_glyph_height,
    );

    let offset = match text.anchor {
      Anchor::TopLeft => (0.0, 0.0),
      Anchor::Top => (-text_size.0 * 0.5, 0.0),
      Anchor::TopRight => (-text_size.0, 0.0),
      Anchor::Left => (0.0, -text_size.1 * 0.5),
      Anchor::Center => (-text_size.0 * 0.5, -text_size.1 * 0.5),
      Anchor::Right => (-text_size.0, -text_size.1 * 0.5),
      Anchor::BottomLeft => (0.0, -text_size.1),
      Anchor::Bottom => (-text_size.0 * 0.5, -text_size.1),
      Anchor::BottomRight => (-text_size.0, -text_size.1),
    };

    let rects = rects
      .into_iter()
      .map(|rect| {
        Rect::new()
          .position((rect.position.0 + offset.0, rect.position.1 + offset.1))
          .size(rect.size)
          .color(crate::unpack_color(rect.color))
          .tex_position(rect.tex_position)
          .tex_size(rect.tex_size)
          .call()
      })
      .collect();

    let glyph_ids = self.add_rects(rects);

    let text_id = TextId {
      glyph_ids,
      text: text.text,
    };

    self.text_ids.push(text_id)
  }

  pub(crate) fn remove_text(&mut self, id: u32) -> Option<Text> {
    let remove_resp = self.text_ids.remove(id)?;
    let text_id = remove_resp.item;
    let rects = self.remove_rects(FxHashSet::from_iter(text_id.glyph_ids));
    let (_, first_rect) = rects[0];
    let first_ch = text_id.text.chars().next().unwrap();
    let first_glyph_metrics = self.char_to_glyph_metrics[&first_ch];

    let font_size =
      (first_rect.size.0 / first_glyph_metrics.size.0 * crate::consts::FONT_SIZE as f32) as u16;

    let text = Text::new()
      .position(first_rect.position)
      .font_size(font_size)
      .text(text_id.text)
      .color(crate::unpack_color(first_rect.color))
      .call();

    Some(text)
  }

  pub(crate) fn add_icon(&mut self, icon: Icon) -> u32 {
    let rect = icon.into_rect(self.calc_glyph_metrics(icon.name));
    let id = self.add_rect(rect);
    self.id_to_icon_name.insert(id, icon.name);
    id
  }

  pub(crate) fn update_icon(&mut self, id: u32, icon: Icon) {
    let rect = icon.into_rect(self.calc_glyph_metrics(icon.name));
    self.update_rect(id, rect);
    *self.id_to_icon_name.get_mut(&id).unwrap() = icon.name;
  }

  pub(crate) fn remove_icon(&mut self, id: u32) -> Option<Icon> {
    let icon_name = self.id_to_icon_name.remove(&id)?;
    let rect = self.remove_rect(id)?;
    let glyph_metrics = self.icon_name_to_glyph_metrics[&icon_name];
    let font_size = (rect.size.0 / glyph_metrics.size.0 * crate::consts::FONT_SIZE as f32) as u16;

    let icon = Icon::new(icon_name)
      .position(rect.position)
      .font_size(font_size)
      .color(crate::unpack_color(rect.color))
      .call();

    Some(icon)
  }
}
