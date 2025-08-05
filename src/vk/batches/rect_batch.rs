use crate::{
  Write,
  collections::SparseSet,
  models::{Anchor, GlyphMetrics, Rect, Text, TextId},
  vk::{Device, GraphicsPipeline, StreamBuffer, graphics_pipeline},
};
use ash::vk;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use std::{collections::VecDeque, mem, rc::Rc};

#[repr(C, align(8))]
struct RectPushConst {
  rect_buf_addr: vk::DeviceAddress,
  cam_position: (f32, f32),
  cam_size: (f32, f32),
  atlas_size: (f32, f32),
}

impl RectPushConst {
  const fn new(
    rect_buf_addr: vk::DeviceAddress,
    cam_position: (f32, f32),
    cam_size: (u32, u32),
    atlas_size: (u32, u32),
  ) -> Self {
    Self {
      rect_buf_addr,
      cam_position,
      cam_size: (cam_size.0 as _, cam_size.1 as _),
      atlas_size: (atlas_size.0 as _, atlas_size.1 as _),
    }
  }
}

pub(crate) struct Creating {
  pipeline: GraphicsPipeline<graphics_pipeline::Creating>,
}

pub(crate) struct Created {
  pipeline: GraphicsPipeline<graphics_pipeline::Created>,
}

pub(crate) struct RectBatch<State> {
  device: Rc<Device>,
  sampler: vk::Sampler,
  descriptor_set_layout: vk::DescriptorSetLayout,
  descriptor_set: vk::DescriptorSet,
  rects: SparseSet<Rect>,
  writes_queues: VecDeque<Vec<Write>>,
  cam_position: (f32, f32),
  char_to_glyph_metrics: FxHashMap<char, GlyphMetrics>,
  text_ids: SparseSet<TextId>,
  state: State,
}

impl RectBatch<Creating> {
  pub(crate) fn new(
    vk_device: Rc<Device>,
    descriptor_pool: vk::DescriptorPool,
    glyph_atlas_view: vk::ImageView,
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

    let descriptor_set_layout_bindings = [vk::DescriptorSetLayoutBinding {
      binding: 0,
      descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
      descriptor_count: 1,
      stage_flags: vk::ShaderStageFlags::FRAGMENT,
      p_immutable_samplers: &sampler,
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

    let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo {
      descriptor_pool,
      descriptor_set_count: 1,
      p_set_layouts: &descriptor_set_layout,
      ..Default::default()
    };

    let descriptor_set = unsafe {
      device
        .allocate_descriptor_sets(&descriptor_set_alloc_info)
        .unwrap()[0]
    };

    let descriptor_image_infos = [vk::DescriptorImageInfo {
      sampler,
      image_view: glyph_atlas_view,
      image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    }];

    let descriptor_set_writes = [vk::WriteDescriptorSet {
      dst_set: descriptor_set,
      dst_binding: 0,
      dst_array_element: 0,
      descriptor_count: descriptor_image_infos.len() as _,
      descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
      p_image_info: descriptor_image_infos.as_ptr(),
      ..Default::default()
    }];

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

    Self {
      device: vk_device,
      sampler,
      descriptor_set_layout,
      descriptor_set,
      rects: SparseSet::with_capacity(cap),
      writes_queues: VecDeque::from_iter([vec![]]),
      cam_position: (0.0, 0.0),
      char_to_glyph_metrics,
      text_ids: SparseSet::new(),
      state: Creating { pipeline },
    }
  }

  pub(crate) fn finish(
    self,
    render_pass: vk::RenderPass,
    swapchain_image_extent: vk::Extent2D,
  ) -> RectBatch<Created> {
    let pipeline = self
      .state
      .pipeline
      .finish(render_pass, swapchain_image_extent);

    RectBatch {
      device: self.device,
      sampler: self.sampler,
      descriptor_set_layout: self.descriptor_set_layout,
      descriptor_set: self.descriptor_set,
      rects: self.rects,
      writes_queues: self.writes_queues,
      cam_position: self.cam_position,
      char_to_glyph_metrics: self.char_to_glyph_metrics,
      text_ids: self.text_ids,
      state: Created { pipeline },
    }
  }

  pub(crate) fn drop(self) {
    let device = self.device.get();
    self.state.pipeline.drop();

    unsafe {
      device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
      device.destroy_sampler(self.sampler, None);
    }
  }
}

impl RectBatch<Created> {
  pub(crate) fn record_draw_commands(
    &mut self,
    command_buffer: vk::CommandBuffer,
    instance_buffer: &StreamBuffer,
    swapchain_image_extent: vk::Extent2D,
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
      crate::consts::GLYPH_ATLAS_SIZE,
    );

    let device = self.device.get();

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
        &[self.descriptor_set],
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
  }

  pub(crate) fn on_swapchain_suboptimal(self) -> RectBatch<Creating> {
    let pipeline = self.state.pipeline.on_swapchain_suboptimal();

    RectBatch {
      device: self.device,
      sampler: self.sampler,
      descriptor_set_layout: self.descriptor_set_layout,
      descriptor_set: self.descriptor_set,
      rects: self.rects,
      writes_queues: self.writes_queues,
      cam_position: self.cam_position,
      char_to_glyph_metrics: self.char_to_glyph_metrics,
      text_ids: self.text_ids,
      state: Creating { pipeline },
    }
  }

  pub(crate) fn drop(self) {
    let device = self.device.get();
    self.state.pipeline.drop();

    unsafe {
      device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
      device.destroy_sampler(self.sampler, None);
    }
  }
}

impl<State> RectBatch<State> {
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
          .tex_position(glyph_metrics.position)
          .tex_size(glyph_metrics.size)
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
    let first_glyph_metrics = &self.char_to_glyph_metrics[&first_ch];

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
}
