use crate::{
  Write,
  collections::SparseSet,
  models::Rect,
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

pub(crate) struct RectBatch<State> {
  device: Rc<Device>,
  rects: SparseSet<Rect>,
  writes_queues: VecDeque<Vec<Write>>,
  cam_position: (f32, f32),
  state: State,
}

impl RectBatch<Creating> {
  pub(crate) fn new(vk_device: Rc<Device>, cap: usize) -> Self {
    let pipeline = GraphicsPipeline::new(
      vk_device.clone(),
      include_bytes!("../../../target/spv/rect.vert.spv"),
      include_bytes!("../../../target/spv/rect.frag.spv"),
      &[vk::PushConstantRange {
        stage_flags: vk::ShaderStageFlags::VERTEX,
        size: mem::size_of::<RectPushConst>() as _,
        ..Default::default()
      }],
    );

    Self {
      device: vk_device,
      rects: SparseSet::with_capacity(cap),
      writes_queues: VecDeque::from_iter([vec![]]),
      cam_position: (0.0, 0.0),
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
      rects: self.rects,
      writes_queues: self.writes_queues,
      cam_position: self.cam_position,
      state: Created { pipeline },
    }
  }

  pub(crate) fn drop(self) {
    self.state.pipeline.drop();
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
    );

    let device = self.device.get();

    unsafe {
      device.cmd_bind_pipeline(
        command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        self.state.pipeline.get(),
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
      rects: self.rects,
      writes_queues: self.writes_queues,
      cam_position: self.cam_position,
      state: Creating { pipeline },
    }
  }

  pub(crate) fn drop(self) {
    self.state.pipeline.drop();
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
}
