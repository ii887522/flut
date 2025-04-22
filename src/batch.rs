use crate::{
  buffers::StreamBuffer, collections::SparseVec, graphics_pipeline::GraphicsPipeline,
  shader::Shader,
};
use ash::{
  Device,
  vk::{
    BufferDeviceAddressInfo, BufferUsageFlags, CommandBuffer, DescriptorSet, DescriptorSetLayout,
    DeviceAddress, Extent2D, PipelineBindPoint, PipelineLayout, PipelineLayoutCreateInfo,
    PushConstantRange, RenderPass, ShaderStageFlags,
  },
};
use atomic_refcell::AtomicRefCell;
use gpu_allocator::vulkan::Allocator;
use rayon::prelude::*;
use std::{cell::RefCell, mem, ptr, rc::Rc};

pub(super) struct Batch<'a, Mesh> {
  pub(super) device: Rc<Device>,
  vert_shader: Shader<'a>,
  frag_shader: Shader<'a>,
  pipeline_layout: PipelineLayout,
  pub(super) mesh_buffer: StreamBuffer,
  pub(super) mesh_buffer_addr: DeviceAddress,
  pub(super) pipeline: Option<GraphicsPipeline>,
  meshes: SparseVec<Mesh>,
}

impl<Mesh> Batch<'_, Mesh> {
  pub(super) fn new<PushConstant>(
    device: Rc<Device>,
    memory_allocator: Rc<RefCell<Allocator>>,
    cap: usize,
    vert_shader_code: &[u8],
    frag_shader_code: &[u8],
    descriptor_set_layouts: &[DescriptorSetLayout],
    mesh_buffer_name: &str,
  ) -> Self {
    let vert_shader = Shader::new(device.clone(), ShaderStageFlags::VERTEX, vert_shader_code);
    let frag_shader = Shader::new(device.clone(), ShaderStageFlags::FRAGMENT, frag_shader_code);

    let push_const_range = PushConstantRange {
      stage_flags: ShaderStageFlags::VERTEX,
      size: mem::size_of::<PushConstant>() as _,
      ..Default::default()
    };

    let pipeline_layout_create_info = PipelineLayoutCreateInfo {
      set_layout_count: descriptor_set_layouts.len() as _,
      p_set_layouts: descriptor_set_layouts.as_ptr(),
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
      mesh_buffer_name,
      (cap * mem::size_of::<Mesh>()) as _,
      BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
    );

    let mesh_buffer_addr_info = BufferDeviceAddressInfo {
      buffer: mesh_buffer.buffer,
      ..Default::default()
    };

    let mesh_buffer_addr = unsafe { device.get_buffer_device_address(&mesh_buffer_addr_info) };

    Self {
      device,
      vert_shader,
      frag_shader,
      pipeline_layout,
      mesh_buffer,
      mesh_buffer_addr,
      pipeline: None,
      meshes: SparseVec::with_capacity(cap),
    }
  }

  pub(super) fn on_swapchain_suboptimal(
    &mut self,
    surface_extent: Extent2D,
    render_pass: RenderPass,
  ) {
    let pipeline = GraphicsPipeline::new(
      self.device.clone(),
      surface_extent,
      &self.vert_shader,
      &self.frag_shader,
      self.pipeline_layout,
      render_pass,
      self.pipeline.as_ref(),
    );

    drop(mem::take(&mut self.pipeline));
    self.pipeline = Some(pipeline);
  }

  pub(super) fn record_draw_commands<PushConstant>(
    &self,
    command_buffer: CommandBuffer,
    descriptor_sets: &[DescriptorSet],
    push_const: PushConstant,
  ) {
    if self.meshes.is_empty() {
      return;
    }

    let pipeline = self.pipeline.as_ref().unwrap();

    unsafe {
      self.device.cmd_bind_pipeline(
        command_buffer,
        PipelineBindPoint::GRAPHICS,
        pipeline.pipeline,
      );

      if !descriptor_sets.is_empty() {
        self.device.cmd_bind_descriptor_sets(
          command_buffer,
          PipelineBindPoint::GRAPHICS,
          self.pipeline_layout,
          0,
          descriptor_sets,
          &[],
        );
      }

      self.device.cmd_push_constants(
        command_buffer,
        self.pipeline_layout,
        ShaderStageFlags::VERTEX,
        0,
        crate::as_bytes(&push_const),
      );

      self
        .device
        .cmd_draw(command_buffer, (6 * self.meshes.len()) as _, 1, 0, 0);
    }
  }
}

impl<Mesh: Copy + Send + Sync> Batch<'_, Mesh> {
  pub(super) fn add(&mut self, mesh: Mesh) -> u16 {
    let mapped_mesh_alloc = self.mesh_buffer.alloc.mapped_ptr().unwrap();

    unsafe {
      ptr::copy_nonoverlapping(
        &mesh,
        (mapped_mesh_alloc.as_ptr() as *mut Mesh).add(self.meshes.len()),
        1,
      );
    }

    self.meshes.push(mesh)
  }

  pub(super) fn batch_add(&mut self, meshes: Vec<Mesh>) -> Vec<u16> {
    let mapped_mesh_alloc = self.mesh_buffer.alloc.mapped_ptr().unwrap();

    unsafe {
      ptr::copy_nonoverlapping(
        meshes.as_ptr(),
        (mapped_mesh_alloc.as_ptr() as *mut Mesh).add(self.meshes.len()),
        meshes.len(),
      );
    }

    self.meshes.par_extend(meshes)
  }

  pub(super) fn update(&mut self, id: u16, mesh: Mesh) {
    let mapped_mesh_alloc = self.mesh_buffer.alloc.mapped_ptr().unwrap();

    unsafe {
      ptr::copy_nonoverlapping(
        &mesh,
        (mapped_mesh_alloc.as_ptr() as *mut Mesh).add(self.meshes.get_dense_index(id) as _),
        1,
      );
    }

    self.meshes[id] = AtomicRefCell::new(mesh);
  }

  pub(super) fn batch_update(&mut self, ids: &[u16], meshes: Vec<Mesh>) {
    ids
      .par_iter()
      .zip(meshes.par_iter())
      .for_each(|(&id, mesh)| unsafe {
        let mapped_mesh_alloc = self.mesh_buffer.alloc.mapped_ptr().unwrap();

        ptr::copy_nonoverlapping(
          mesh,
          (mapped_mesh_alloc.as_ptr() as *mut Mesh).add(self.meshes.get_dense_index(id) as _),
          1,
        );
      });

    self.meshes.par_set(ids, meshes);
  }

  pub(super) fn remove(&mut self, id: u16) -> Mesh {
    let index = self.meshes.get_dense_index(id);
    let result = self.meshes.remove(id);

    let Some(mesh) = self.meshes.get_by_dense_index(index) else {
      return result;
    };

    let mesh = mesh.borrow();
    let mapped_mesh_alloc = self.mesh_buffer.alloc.mapped_ptr().unwrap();

    unsafe {
      ptr::copy_nonoverlapping(
        &*mesh,
        (mapped_mesh_alloc.as_ptr() as *mut Mesh).add(index as _),
        1,
      );
    }

    result
  }

  pub(super) fn batch_remove(&mut self, ids: &[u16]) {
    let indices = ids
      .par_iter()
      .map(|&id| self.meshes.get_dense_index(id))
      .collect::<Vec<_>>();

    self.meshes.par_remove(ids);

    indices
      .into_par_iter()
      .filter_map(|index| {
        self
          .meshes
          .get_by_dense_index(index)
          .map(|mesh| (index, mesh))
      })
      .for_each(|(index, mesh)| {
        let mesh = mesh.borrow();
        let mapped_mesh_alloc = self.mesh_buffer.alloc.mapped_ptr().unwrap();

        unsafe {
          ptr::copy_nonoverlapping(
            &*mesh,
            (mapped_mesh_alloc.as_ptr() as *mut Mesh).add(index as _),
            1,
          );
        }
      });
  }

  pub(super) fn clear(&mut self) {
    self.meshes.clear();
  }
}

impl<Mesh> Drop for Batch<'_, Mesh> {
  fn drop(&mut self) {
    unsafe {
      self
        .device
        .destroy_pipeline_layout(self.pipeline_layout, None);
    }
  }
}
