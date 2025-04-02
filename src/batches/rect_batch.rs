use crate::{
  buffers::{StaticBuffer, StreamBuffer},
  models::Rect,
  pipelines::{RectPipeline, RectPushConstant},
  shaders::{RectFragShader, RectVertShader, RectVertex},
};
use ash::{
  Device,
  vk::{
    BufferUsageFlags, CommandBuffer, Extent2D, IndexType, PipelineBindPoint, RenderPass,
    ShaderStageFlags,
  },
};
use gpu_allocator::vulkan::Allocator;
use std::{cell::RefCell, mem, rc::Rc};

const VERTICES: &[RectVertex] = &[
  RectVertex {
    position: (0.0, 0.0),
  },
  RectVertex {
    position: (0.05, 0.0),
  },
  RectVertex {
    position: (0.05, 0.05),
  },
  RectVertex {
    position: (0.0, 0.05),
  },
];

const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

pub(crate) struct RectBatch<'a> {
  device: Rc<Device>,
  vert_shader: RectVertShader<'a>,
  frag_shader: RectFragShader<'a>,
  pub(crate) inst_buffer: StreamBuffer<'a>,
  pub(crate) vert_buffer: StaticBuffer<'a>,
  pub(crate) index_buffer: StaticBuffer<'a>,
  pub(crate) pipeline: Option<RectPipeline>,
  rect_count: u32,
}

impl RectBatch<'_> {
  pub(crate) fn new(device: Rc<Device>, memory_allocator: Rc<RefCell<Allocator>>) -> Self {
    let vert_shader = RectVertShader::new(device.clone());
    let frag_shader = RectFragShader::new(device.clone());

    let inst_buffer = StreamBuffer::new(
      device.clone(),
      memory_allocator.clone(),
      "rect_inst_buffer",
      mem::size_of::<Rect>() as _,
      BufferUsageFlags::VERTEX_BUFFER,
    );

    let vert_buffer = StaticBuffer::new(
      device.clone(),
      memory_allocator.clone(),
      "rect_vert_buffer",
      BufferUsageFlags::VERTEX_BUFFER,
      VERTICES,
    );

    let index_buffer = StaticBuffer::new(
      device.clone(),
      memory_allocator.clone(),
      "rect_index_buffer",
      BufferUsageFlags::INDEX_BUFFER,
      INDICES,
    );

    Self {
      device,
      vert_shader,
      frag_shader,
      inst_buffer,
      vert_buffer,
      index_buffer,
      pipeline: None,
      rect_count: 0,
    }
  }

  pub(crate) fn on_swapchain_suboptimal(
    &mut self,
    surface_extent: Extent2D,
    render_pass: RenderPass,
  ) {
    let rect_pipeline = RectPipeline::new(
      self.device.clone(),
      surface_extent,
      &self.vert_shader,
      &self.frag_shader,
      render_pass,
      self.pipeline.as_ref(),
    );

    drop(mem::take(&mut self.pipeline));
    self.pipeline = Some(rect_pipeline);
  }

  pub(crate) fn record_init_commands(&self, command_buffer: CommandBuffer) {
    unsafe {
      self.device.cmd_copy_buffer(
        command_buffer,
        self.vert_buffer.staging_buffer,
        self.vert_buffer.buffer,
        &[self.vert_buffer.buffer_copy],
      );

      self.device.cmd_copy_buffer(
        command_buffer,
        self.index_buffer.staging_buffer,
        self.index_buffer.buffer,
        &[self.index_buffer.buffer_copy],
      );
    }
  }

  pub(crate) fn record_draw_commands(
    &self,
    command_buffer: CommandBuffer,
    surface_extent: Extent2D,
  ) {
    let rect_push_const = RectPushConstant {
      camera_size: (surface_extent.width as _, surface_extent.height as _),
    };

    let pipeline = self.pipeline.as_ref().unwrap();

    unsafe {
      self.device.cmd_bind_pipeline(
        command_buffer,
        PipelineBindPoint::GRAPHICS,
        pipeline.pipeline,
      );

      self.device.cmd_bind_vertex_buffers(
        command_buffer,
        0,
        &[self.vert_buffer.buffer, self.inst_buffer.buffer],
        &[0, 0],
      );

      self.device.cmd_bind_index_buffer(
        command_buffer,
        self.index_buffer.buffer,
        0,
        IndexType::UINT16,
      );

      self.device.cmd_push_constants(
        command_buffer,
        pipeline.layout,
        ShaderStageFlags::VERTEX,
        0,
        crate::as_bytes(&rect_push_const),
      );

      self
        .device
        .cmd_draw_indexed(command_buffer, INDICES.len() as _, self.rect_count, 0, 0, 0);
    }
  }

  pub(crate) fn add(&mut self, rect: Rect) {
    let mut mapped_inst_alloc = self.inst_buffer.alloc.try_as_mapped_slab().unwrap();
    presser::copy_to_offset(&rect, &mut mapped_inst_alloc, 0).unwrap();
    self.rect_count += 1;
  }
}
