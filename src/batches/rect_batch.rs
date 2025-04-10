use crate::{
  buffers::{StaticBuffer, StreamBuffer},
  collections::SparseVec,
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
use atomic_refcell::AtomicRefCell;
use gpu_allocator::vulkan::Allocator;
use rayon::prelude::*;
use std::{cell::RefCell, mem, ptr, rc::Rc};

const VERTICES: &[RectVertex] = &[
  RectVertex {
    position: (0.0, 0.0),
  },
  RectVertex {
    position: (1.0, 0.0),
  },
  RectVertex {
    position: (1.0, 1.0),
  },
  RectVertex {
    position: (0.0, 1.0),
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
  rects: SparseVec<Rect>,
}

impl RectBatch<'_> {
  pub(crate) fn new(
    device: Rc<Device>,
    memory_allocator: Rc<RefCell<Allocator>>,
    cap: usize,
  ) -> Self {
    let vert_shader = RectVertShader::new(device.clone());
    let frag_shader = RectFragShader::new(device.clone());

    let inst_buffer = StreamBuffer::new(
      device.clone(),
      memory_allocator.clone(),
      "rect_inst_buffer",
      (cap * mem::size_of::<Rect>()) as _,
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
      rects: SparseVec::with_capacity(cap),
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
    if self.rects.is_empty() {
      return;
    }

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

      self.device.cmd_draw_indexed(
        command_buffer,
        INDICES.len() as _,
        self.rects.len() as _,
        0,
        0,
        0,
      );
    }
  }

  pub(crate) fn add(&mut self, rect: Rect) -> u16 {
    let mapped_inst_alloc = self.inst_buffer.alloc.mapped_ptr().unwrap();

    unsafe {
      ptr::copy_nonoverlapping(
        &rect,
        (mapped_inst_alloc.as_ptr() as *mut Rect).add(self.rects.len()),
        1,
      );
    }

    self.rects.push(rect)
  }

  pub(crate) fn batch_add(&mut self, rects: Vec<Rect>) -> Vec<u16> {
    let mapped_inst_alloc = self.inst_buffer.alloc.mapped_ptr().unwrap();

    unsafe {
      ptr::copy_nonoverlapping(
        rects.as_ptr(),
        (mapped_inst_alloc.as_ptr() as *mut Rect).add(self.rects.len()),
        rects.len(),
      );
    }

    self.rects.par_extend(rects)
  }

  pub(crate) fn update(&mut self, id: u16, rect: Rect) {
    let mapped_inst_alloc = self.inst_buffer.alloc.mapped_ptr().unwrap();

    unsafe {
      ptr::copy_nonoverlapping(
        &rect,
        (mapped_inst_alloc.as_ptr() as *mut Rect).add(self.rects.get_dense_index(id) as _),
        1,
      );
    }

    self.rects[id] = AtomicRefCell::new(rect);
  }

  pub(crate) fn batch_update(&mut self, ids: &[u16], rects: Vec<Rect>) {
    ids
      .par_iter()
      .zip(rects.par_iter())
      .for_each(|(&id, rect)| unsafe {
        let mapped_inst_alloc = self.inst_buffer.alloc.mapped_ptr().unwrap();

        ptr::copy_nonoverlapping(
          rect,
          (mapped_inst_alloc.as_ptr() as *mut Rect).add(self.rects.get_dense_index(id) as _),
          1,
        );
      });

    self.rects.par_set(ids, rects);
  }

  pub(crate) fn remove(&mut self, id: u16) -> Rect {
    let index = self.rects.get_dense_index(id);
    let result = self.rects.remove(id);

    let Some(rect) = self.rects.get_by_dense_index(index) else {
      return result;
    };

    let rect = rect.borrow();
    let mapped_inst_alloc = self.inst_buffer.alloc.mapped_ptr().unwrap();

    unsafe {
      ptr::copy_nonoverlapping(
        &*rect,
        (mapped_inst_alloc.as_ptr() as *mut Rect).add(index as _),
        1,
      );
    }

    result
  }

  pub(crate) fn batch_remove(&mut self, ids: &[u16]) {
    let indices = ids
      .par_iter()
      .map(|&id| self.rects.get_dense_index(id))
      .collect::<Vec<_>>();

    self.rects.par_remove(ids);

    indices
      .into_par_iter()
      .filter_map(|index| {
        self
          .rects
          .get_by_dense_index(index)
          .map(|rect| (index, rect))
      })
      .for_each(|(index, rect)| {
        let rect = rect.borrow();
        let mapped_inst_alloc = self.inst_buffer.alloc.mapped_ptr().unwrap();

        unsafe {
          ptr::copy_nonoverlapping(
            &*rect,
            (mapped_inst_alloc.as_ptr() as *mut Rect).add(index as _),
            1,
          );
        }
      });
  }

  pub(crate) fn clear(&mut self) {
    self.rects.clear();
  }
}
