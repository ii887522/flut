use crate::{
  collections::{
    LruCache, SparseSet,
    sparse_set::{self, Id},
  },
  models::{Align, Text},
  pipelines::glyph_pipeline::Glyph,
  renderers::{MAX_IN_FLIGHT_FRAME_COUNT, ModelRenderer, model_renderer},
};
use ash::vk;
use etagere::{AllocId, BucketedAtlasAllocator, Size};
use font_kit::{
  canvas::{Canvas, Format, RasterizationOptions},
  family_name::FamilyName,
  font::Font,
  hinting::HintingOptions,
  properties::Properties,
  source::SystemSource,
};
use pathfinder_geometry::{
  transform2d::Transform2F,
  vector::{Vector2F, Vector2I},
};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use sdl3::video::Window;
use std::{collections::VecDeque, ffi::c_void, mem, ptr};
use vk_mem::{self, Alloc};

const RESOLUTION_SCALE: f32 = 2.0;
const GLYPH_MARGIN: i32 = 1;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct GlyphKey {
  ch: char,
  font_size: u16,
}

#[derive(Clone, Copy)]
enum GlyphMetrics {
  Visible {
    advance: (f32, f32),
    origin: (f32, f32),
    position: (f32, f32),
    size: (f32, f32),
    glyph_id: u32,
    alloc_id: AllocId,
    ref_count: u32,
  },
  Invisible {
    advance: (f32, f32),
    ref_count: u32,
  },
}

struct TextId {
  glyph_keys: Box<[GlyphKey]>,
  glyph_ids: Box<[sparse_set::Id]>,
}

pub(super) trait State {
  type GlyphState: model_renderer::State<Model = Glyph>;

  fn get_glyph_renderer_mut(&mut self) -> &mut ModelRenderer<Self::GlyphState>;
}

pub(super) struct Creating {
  glyph_renderer: ModelRenderer<model_renderer::Creating<Glyph>>,
}

impl State for Creating {
  type GlyphState = model_renderer::Creating<Glyph>;

  #[inline]
  fn get_glyph_renderer_mut(&mut self) -> &mut ModelRenderer<Self::GlyphState> {
    &mut self.glyph_renderer
  }
}

pub(super) struct Created {
  glyph_renderer: ModelRenderer<model_renderer::Created<Glyph>>,
}

impl State for Created {
  type GlyphState = model_renderer::Created<Glyph>;

  #[inline]
  fn get_glyph_renderer_mut(&mut self) -> &mut ModelRenderer<Self::GlyphState> {
    &mut self.glyph_renderer
  }
}

pub(super) struct TextRenderer<S: State> {
  glyph_atlas_allocator: BucketedAtlasAllocator,
  staging_glyph_atlas_buffer: vk::Buffer,
  staging_glyph_atlas_buffer_alloc: vk_mem::Allocation,
  staging_glyph_atlas_buffer_data: *mut c_void,
  glyph_atlas_images: Vec<vk::Image>,
  glyph_atlas_image_allocs: Vec<vk_mem::Allocation>,
  glyph_atlas_image_views: Vec<vk::ImageView>,
  descriptor_sets: Box<[vk::DescriptorSet]>,
  font: Font,
  unused_glyph_cache: LruCache<GlyphKey>,
  glyph_metrics_map: FxHashMap<GlyphKey, GlyphMetrics>,
  glyph_keys_queue: VecDeque<FxHashSet<GlyphKey>>,
  glyph_atlas_region_queue: Vec<vk::BufferImageCopy2<'static>>,
  text_ids: SparseSet<TextId>,
  read_image_index: usize,
  window_display_scale: f32,
  state: S,
}

impl TextRenderer<Creating> {
  pub(super) fn new(
    window: Window,
    device: &ash::Device,
    vk_allocator: &vk_mem::Allocator,
    transition_command_buffer: vk::CommandBuffer,
    descriptor_pool: vk::DescriptorPool,
    glyph_capacity: usize,
    glyph_atlas_size: (u32, u32),
  ) -> Self {
    let glyph_atlas_allocator =
      BucketedAtlasAllocator::new(Size::new(glyph_atlas_size.0 as _, glyph_atlas_size.1 as _));

    let staging_glyph_atlas_buffer_create_info = vk::BufferCreateInfo {
      size: (MAX_IN_FLIGHT_FRAME_COUNT * (glyph_atlas_size.0 * glyph_atlas_size.1) as usize) as _,
      usage: vk::BufferUsageFlags::TRANSFER_SRC,
      sharing_mode: vk::SharingMode::EXCLUSIVE,
      ..Default::default()
    };

    let staging_glyph_atlas_buffer_alloc_create_info = vk_mem::AllocationCreateInfo {
      flags: vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE
        | vk_mem::AllocationCreateFlags::MAPPED,
      usage: vk_mem::MemoryUsage::AutoPreferHost,
      ..Default::default()
    };

    let (staging_glyph_atlas_buffer, staging_glyph_atlas_buffer_alloc) = unsafe {
      vk_allocator
        .create_buffer(
          &staging_glyph_atlas_buffer_create_info,
          &staging_glyph_atlas_buffer_alloc_create_info,
        )
        .unwrap()
    };

    let staging_glyph_atlas_buffer_alloc_info =
      vk_allocator.get_allocation_info(&staging_glyph_atlas_buffer_alloc);

    let staging_glyph_atlas_buffer_data = staging_glyph_atlas_buffer_alloc_info.mapped_data;

    let glyph_atlas_image_create_info = vk::ImageCreateInfo {
      image_type: vk::ImageType::TYPE_2D,
      format: vk::Format::R8_UNORM,
      extent: vk::Extent3D {
        width: glyph_atlas_size.0,
        height: glyph_atlas_size.1,
        depth: 1,
      },
      mip_levels: 1,
      array_layers: 1,
      samples: vk::SampleCountFlags::TYPE_1,
      tiling: vk::ImageTiling::OPTIMAL,
      usage: vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
      sharing_mode: vk::SharingMode::EXCLUSIVE,
      initial_layout: vk::ImageLayout::UNDEFINED,
      ..Default::default()
    };

    let glyph_atlas_image_alloc_create_info = vk_mem::AllocationCreateInfo {
      usage: vk_mem::MemoryUsage::AutoPreferDevice,
      ..Default::default()
    };

    let (glyph_atlas_images, (glyph_atlas_image_allocs, glyph_atlas_image_views)): (
      Vec<_>,
      (Vec<_>, Vec<_>),
    ) = (0..MAX_IN_FLIGHT_FRAME_COUNT)
      .map(|_| unsafe {
        let (glyph_atlas_image, glyph_atlas_image_alloc) = vk_allocator
          .create_image(
            &glyph_atlas_image_create_info,
            &glyph_atlas_image_alloc_create_info,
          )
          .unwrap();

        let glyph_atlas_image_view_create_info = vk::ImageViewCreateInfo {
          image: glyph_atlas_image,
          view_type: vk::ImageViewType::TYPE_2D,
          format: vk::Format::R8_UNORM,
          subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
          },
          ..Default::default()
        };

        let glyph_atlas_image_view = device
          .create_image_view(&glyph_atlas_image_view_create_info, None)
          .unwrap();

        (
          glyph_atlas_image,
          (glyph_atlas_image_alloc, glyph_atlas_image_view),
        )
      })
      .unzip();

    let font_handle = SystemSource::new()
      .select_best_match(&[FamilyName::SansSerif], &Properties::new())
      .unwrap();

    let font = font_handle.load().unwrap();
    let unique_glyph_capacity = ((glyph_atlas_size.0 * glyph_atlas_size.1) >> 8) as _;
    let unused_glyph_cache = LruCache::with_capacity(unique_glyph_capacity);

    let glyph_metrics_map =
      FxHashMap::with_capacity_and_hasher(unique_glyph_capacity, FxBuildHasher);

    let glyph_keys_queue = VecDeque::from_iter([FxHashSet::default()]);
    let glyph_atlas_region_queue = vec![];
    let text_ids = SparseSet::new();
    let read_image_index = 0;
    let window_display_scale = window.display_scale();
    let glyph_renderer = ModelRenderer::new(device, glyph_capacity);

    let image_shader_barriers = glyph_atlas_images
      .iter()
      .map(|&glyph_atlas_image| vk::ImageMemoryBarrier {
        src_access_mask: vk::AccessFlags::empty(),
        dst_access_mask: vk::AccessFlags::SHADER_READ,
        old_layout: vk::ImageLayout::UNDEFINED,
        new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        image: glyph_atlas_image,
        subresource_range: vk::ImageSubresourceRange {
          aspect_mask: vk::ImageAspectFlags::COLOR,
          base_mip_level: 0,
          level_count: 1,
          base_array_layer: 0,
          layer_count: 1,
        },
        ..Default::default()
      })
      .collect::<Box<_>>();

    unsafe {
      device.cmd_pipeline_barrier(
        transition_command_buffer,
        vk::PipelineStageFlags::TOP_OF_PIPE,
        vk::PipelineStageFlags::FRAGMENT_SHADER,
        vk::DependencyFlags::BY_REGION,
        &[],
        &[],
        &image_shader_barriers,
      );
    }

    let descriptor_set_layouts =
      [glyph_renderer.get_descriptor_set_layout(); MAX_IN_FLIGHT_FRAME_COUNT];

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

    let descriptor_image_infos = glyph_atlas_image_views
      .iter()
      .map(|&image_view| vk::DescriptorImageInfo {
        image_view,
        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        ..Default::default()
      })
      .collect::<Box<_>>();

    let descriptor_writes = descriptor_sets
      .iter()
      .zip(descriptor_image_infos.iter())
      .map(
        |(&descriptor_set, descriptor_image_info)| vk::WriteDescriptorSet {
          dst_set: descriptor_set,
          dst_binding: 0,
          dst_array_element: 0,
          descriptor_count: 1,
          descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
          p_image_info: descriptor_image_info,
          ..Default::default()
        },
      )
      .collect::<Box<_>>();

    unsafe {
      device.update_descriptor_sets(&descriptor_writes, &[]);
    }

    Self {
      glyph_atlas_allocator,
      staging_glyph_atlas_buffer,
      staging_glyph_atlas_buffer_alloc,
      staging_glyph_atlas_buffer_data,
      glyph_atlas_images,
      glyph_atlas_image_allocs,
      glyph_atlas_image_views,
      descriptor_sets: descriptor_sets.into_boxed_slice(),
      font,
      unused_glyph_cache,
      glyph_metrics_map,
      glyph_keys_queue,
      glyph_atlas_region_queue,
      text_ids,
      read_image_index,
      window_display_scale,
      state: Creating { glyph_renderer },
    }
  }

  pub(super) fn finish(
    self,
    device: &ash::Device,
    render_pass: vk::RenderPass,
    pipeline_cache: vk::PipelineCache,
    swapchain_image_extent: vk::Extent2D,
    msaa_samples: vk::SampleCountFlags,
  ) -> TextRenderer<Created> {
    let glyph_renderer = self.state.glyph_renderer.finish(
      device,
      render_pass,
      pipeline_cache,
      swapchain_image_extent,
      msaa_samples,
    );

    TextRenderer {
      glyph_atlas_allocator: self.glyph_atlas_allocator,
      staging_glyph_atlas_buffer: self.staging_glyph_atlas_buffer,
      staging_glyph_atlas_buffer_alloc: self.staging_glyph_atlas_buffer_alloc,
      staging_glyph_atlas_buffer_data: self.staging_glyph_atlas_buffer_data,
      glyph_atlas_images: self.glyph_atlas_images,
      glyph_atlas_image_allocs: self.glyph_atlas_image_allocs,
      glyph_atlas_image_views: self.glyph_atlas_image_views,
      descriptor_sets: self.descriptor_sets,
      font: self.font,
      unused_glyph_cache: self.unused_glyph_cache,
      glyph_metrics_map: self.glyph_metrics_map,
      glyph_keys_queue: self.glyph_keys_queue,
      glyph_atlas_region_queue: self.glyph_atlas_region_queue,
      text_ids: self.text_ids,
      read_image_index: self.read_image_index,
      window_display_scale: self.window_display_scale,
      state: Created { glyph_renderer },
    }
  }

  pub(super) fn drop(mut self, device: &ash::Device, vk_allocator: &vk_mem::Allocator) {
    self.state.glyph_renderer.drop(device);

    unsafe {
      self
        .glyph_atlas_image_views
        .iter()
        .for_each(|&image_view| device.destroy_image_view(image_view, None));

      self
        .glyph_atlas_images
        .iter()
        .zip(self.glyph_atlas_image_allocs.iter_mut())
        .for_each(|(&image, image_alloc)| vk_allocator.destroy_image(image, image_alloc));

      vk_allocator.destroy_buffer(
        self.staging_glyph_atlas_buffer,
        &mut self.staging_glyph_atlas_buffer_alloc,
      );
    }
  }
}

impl TextRenderer<Created> {
  #[inline]
  pub(super) const fn get_glyph_renderer(&self) -> &ModelRenderer<model_renderer::Created<Glyph>> {
    &self.state.glyph_renderer
  }

  #[inline]
  pub(super) const fn get_descriptor_set(&self) -> vk::DescriptorSet {
    self.descriptor_sets[self.read_image_index]
  }

  #[inline]
  pub(super) const fn get_read_image_index(&self) -> usize {
    self.read_image_index
  }

  #[inline]
  pub(super) const fn get_write_image_index(&self) -> usize {
    (self.read_image_index + 1) % self.glyph_atlas_images.len()
  }

  pub(super) fn on_swapchain_suboptimal(self) -> TextRenderer<Creating> {
    TextRenderer {
      glyph_atlas_allocator: self.glyph_atlas_allocator,
      staging_glyph_atlas_buffer: self.staging_glyph_atlas_buffer,
      staging_glyph_atlas_buffer_alloc: self.staging_glyph_atlas_buffer_alloc,
      staging_glyph_atlas_buffer_data: self.staging_glyph_atlas_buffer_data,
      glyph_atlas_images: self.glyph_atlas_images,
      glyph_atlas_image_allocs: self.glyph_atlas_image_allocs,
      glyph_atlas_image_views: self.glyph_atlas_image_views,
      descriptor_sets: self.descriptor_sets,
      font: self.font,
      unused_glyph_cache: self.unused_glyph_cache,
      glyph_metrics_map: self.glyph_metrics_map,
      glyph_keys_queue: self.glyph_keys_queue,
      glyph_atlas_region_queue: self.glyph_atlas_region_queue,
      text_ids: self.text_ids,
      read_image_index: self.read_image_index,
      window_display_scale: self.window_display_scale,
      state: Creating {
        glyph_renderer: self.state.glyph_renderer.on_swapchain_suboptimal(),
      },
    }
  }

  pub(super) fn flush_atlas_updates(
    &mut self,
    device: &ash::Device,
    glyph_atlas_semaphore: vk::Semaphore,
    glyph_atlas_semaphore_value: u64,
  ) -> bool {
    let glyph_key_count = self
      .glyph_keys_queue
      .iter()
      .map(|glyph_keys| glyph_keys.len())
      .sum();

    if glyph_key_count == 0 {
      return false;
    }

    let write_image_index = self.get_write_image_index();
    let glyph_atlas_size = self.glyph_atlas_allocator.size();
    let glyph_atlas_byte_count = (glyph_atlas_size.width * glyph_atlas_size.height) as usize;
    let buffer_offset = write_image_index * glyph_atlas_byte_count;
    let mut pixels_offset = buffer_offset;

    let (pixels, regions) = self.glyph_keys_queue.iter().flatten().fold(
      (
        Vec::with_capacity(glyph_atlas_byte_count),
        Vec::with_capacity(glyph_key_count),
      ),
      |(mut pixels, mut regions), glyph_key| {
        let GlyphMetrics::Visible {
          origin,
          position,
          size,
          glyph_id,
          ..
        } = self.glyph_metrics_map[glyph_key]
        else {
          return (pixels, regions);
        };

        let mut canvas = Canvas::new(
          Vector2I::new(
            (size.0 as i32) + (GLYPH_MARGIN << 1),
            (size.1 as i32) + (GLYPH_MARGIN << 1),
          ),
          Format::A8,
        );

        self
          .font
          .rasterize_glyph(
            &mut canvas,
            glyph_id,
            glyph_key.font_size as f32 * self.window_display_scale * RESOLUTION_SCALE,
            Transform2F::from_translation(Vector2F::new(
              GLYPH_MARGIN as f32 - origin.0,
              GLYPH_MARGIN as f32 - origin.1,
            )),
            HintingOptions::Vertical(glyph_key.font_size as f32 * self.window_display_scale),
            RasterizationOptions::GrayscaleAa,
          )
          .unwrap();

        let region = vk::BufferImageCopy2 {
          buffer_offset: pixels_offset as _,
          buffer_row_length: canvas.stride as _,
          buffer_image_height: canvas.size.y() as _,
          image_subresource: vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
          },
          image_offset: vk::Offset3D {
            x: (position.0 as i32) - GLYPH_MARGIN,
            y: (position.1 as i32) - GLYPH_MARGIN,
            z: 0,
          },
          image_extent: vk::Extent3D {
            width: ((size.0 as i32) + (GLYPH_MARGIN << 1)) as _,
            height: ((size.1 as i32) + (GLYPH_MARGIN << 1)) as _,
            depth: 1,
          },
          ..Default::default()
        };

        pixels_offset += canvas.stride * canvas.size.y() as usize;
        pixels.extend(canvas.pixels);
        regions.push(region);
        (pixels, regions)
      },
    );

    let glyph_atlas_semaphore_wait_info = vk::SemaphoreWaitInfo {
      semaphore_count: 1,
      p_semaphores: &glyph_atlas_semaphore,
      p_values: &glyph_atlas_semaphore_value,
      ..Default::default()
    };

    let glyph_atlas_semaphore_signal_info = vk::SemaphoreSignalInfo {
      semaphore: glyph_atlas_semaphore,
      value: glyph_atlas_semaphore_value + 1,
      ..Default::default()
    };

    unsafe {
      device
        .wait_semaphores(&glyph_atlas_semaphore_wait_info, u64::MAX)
        .unwrap();

      ptr::copy_nonoverlapping(
        pixels.as_ptr(),
        (self.staging_glyph_atlas_buffer_data as *mut u8).add(buffer_offset),
        pixels.len(),
      );

      device
        .signal_semaphore(&glyph_atlas_semaphore_signal_info)
        .unwrap();
    }

    self.glyph_atlas_region_queue = regions;
    self.glyph_keys_queue.push_back(FxHashSet::default());

    if self.glyph_keys_queue.len() > MAX_IN_FLIGHT_FRAME_COUNT {
      self.glyph_keys_queue.pop_front();
    }

    true
  }

  pub(super) fn record_copy_commands(
    &mut self,
    device: &ash::Device,
    transfer_command_buffer: vk::CommandBuffer,
    graphics_queue_family_index: u32,
    transfer_queue_family_index: u32,
  ) {
    let write_image_index = self.get_write_image_index();
    let glyph_atlas_image = self.glyph_atlas_images[write_image_index];

    let image_transfer_barrier = vk::ImageMemoryBarrier {
      src_access_mask: vk::AccessFlags::SHADER_READ,
      dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
      old_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
      new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
      src_queue_family_index: graphics_queue_family_index,
      dst_queue_family_index: transfer_queue_family_index,
      image: glyph_atlas_image,
      subresource_range: vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
      },
      ..Default::default()
    };

    let regions = mem::take(&mut self.glyph_atlas_region_queue);

    let copy_buffer_to_image_info = vk::CopyBufferToImageInfo2 {
      src_buffer: self.staging_glyph_atlas_buffer,
      dst_image: glyph_atlas_image,
      dst_image_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
      region_count: regions.len() as _,
      p_regions: regions.as_ptr(),
      ..Default::default()
    };

    let image_shader_barrier = vk::ImageMemoryBarrier {
      src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
      dst_access_mask: vk::AccessFlags::SHADER_READ,
      old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
      new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
      src_queue_family_index: transfer_queue_family_index,
      dst_queue_family_index: graphics_queue_family_index,
      image: glyph_atlas_image,
      subresource_range: vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
      },
      ..Default::default()
    };

    unsafe {
      device.cmd_pipeline_barrier(
        transfer_command_buffer,
        vk::PipelineStageFlags::FRAGMENT_SHADER,
        vk::PipelineStageFlags::TRANSFER,
        vk::DependencyFlags::BY_REGION,
        &[],
        &[],
        &[image_transfer_barrier],
      );

      device.cmd_copy_buffer_to_image2(transfer_command_buffer, &copy_buffer_to_image_info);

      device.cmd_pipeline_barrier(
        transfer_command_buffer,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::FRAGMENT_SHADER,
        vk::DependencyFlags::BY_REGION,
        &[],
        &[],
        &[image_shader_barrier],
      );
    }

    self.read_image_index = write_image_index;
  }

  pub(super) fn drop(mut self, device: &ash::Device, vk_allocator: &vk_mem::Allocator) {
    self.state.glyph_renderer.drop(device);

    unsafe {
      self
        .glyph_atlas_image_views
        .iter()
        .for_each(|&image_view| device.destroy_image_view(image_view, None));

      self
        .glyph_atlas_images
        .iter()
        .zip(self.glyph_atlas_image_allocs.iter_mut())
        .for_each(|(&image, image_alloc)| vk_allocator.destroy_image(image, image_alloc));

      vk_allocator.destroy_buffer(
        self.staging_glyph_atlas_buffer,
        &mut self.staging_glyph_atlas_buffer_alloc,
      );
    }
  }
}

impl<S: State> TextRenderer<S> {
  #[inline]
  pub(super) fn get_glyph_renderer_mut(&mut self) -> &mut ModelRenderer<S::GlyphState> {
    self.state.get_glyph_renderer_mut()
  }

  pub(super) fn add_text(&mut self, text: Text) -> Id {
    let text_id = self.prepare_text(text);
    let add_resp = self.text_ids.add(text_id);
    add_resp.id
  }

  pub(super) fn update_text(&mut self, id: Id, text: Text) {
    let new_text_id = self.prepare_text(text);
    let text_id = self.text_ids.get_mut(id).unwrap();
    let old_text_id = mem::replace(text_id, new_text_id);
    self.cleanup_text(old_text_id);
  }

  pub(super) fn remove_text(&mut self, id: Id) {
    let remove_resp = self.text_ids.remove(id);
    self.cleanup_text(remove_resp.item);
  }

  pub(super) fn bulk_add_text(&mut self, texts: Box<[Text]>) -> Box<[Id]> {
    texts.into_iter().map(|text| self.add_text(text)).collect()
  }

  pub(super) fn bulk_update_text(&mut self, updates: Box<[(Id, Text)]>) {
    for (id, text) in updates {
      self.update_text(id, text);
    }
  }

  pub(super) fn bulk_remove_text(&mut self, ids: &[Id]) {
    for &id in ids {
      self.remove_text(id);
    }
  }

  fn prepare_text(&mut self, text: Text) -> TextId {
    let mut current_glyph_position = text.position;

    let (glyph_keys, glyphs): (Vec<_>, Vec<_>) = text
      .text
      .chars()
      .filter_map(|mut ch| {
        let (glyph_key, glyph_metrics) = loop {
          let glyph_key = GlyphKey {
            ch,
            font_size: text.font_size,
          };

          if let Some(glyph_metrics) = self.glyph_metrics_map.get_mut(&glyph_key) {
            let ref_count = *match glyph_metrics {
              GlyphMetrics::Visible { ref_count, .. } => ref_count,
              GlyphMetrics::Invisible { ref_count, .. } => ref_count,
            };

            if ref_count == 0 {
              self.unused_glyph_cache.remove(&glyph_key);
            }

            break (glyph_key, glyph_metrics);
          }

          let Some(glyph_id) = self.font.glyph_for_char(ch) else {
            if ch == '?' {
              panic!("'?' not found in font");
            }

            println!("'{ch}' not found in font, fallback to '?'");
            ch = '?';
            continue;
          };

          let advance = self.font.advance(glyph_id).unwrap();

          let glyph_bounds = self
            .font
            .raster_bounds(
              glyph_id,
              glyph_key.font_size as f32 * self.window_display_scale * RESOLUTION_SCALE,
              Transform2F::default(),
              HintingOptions::Vertical(glyph_key.font_size as f32 * self.window_display_scale),
              RasterizationOptions::GrayscaleAa,
            )
            .unwrap();

          let glyph_metrics = if glyph_bounds.width() > 0 && glyph_bounds.height() > 0 {
            let mut unused_glyph_drop_count = 1;

            let glyph_alloc = loop {
              if let Some(glyph_alloc) = self.glyph_atlas_allocator.allocate(Size::new(
                glyph_bounds.width() + (GLYPH_MARGIN << 1),
                glyph_bounds.height() + (GLYPH_MARGIN << 1),
              )) {
                break glyph_alloc;
              }

              for index in 0..unused_glyph_drop_count {
                let Some(unused_glyph_key) = self.unused_glyph_cache.invalidate_one() else {
                  if index == 0 {
                    panic!("Not enough space to allocate in glyph atlas");
                  }

                  break;
                };

                for glyph_keys in &mut self.glyph_keys_queue {
                  glyph_keys.remove(&unused_glyph_key);
                }

                let GlyphMetrics::Visible { alloc_id, .. } =
                  self.glyph_metrics_map.remove(&unused_glyph_key).unwrap()
                else {
                  continue;
                };

                self.glyph_atlas_allocator.deallocate(alloc_id);
              }

              unused_glyph_drop_count <<= 1;
            };

            let glyph_keys = self.glyph_keys_queue.back_mut().unwrap();
            glyph_keys.insert(glyph_key);

            GlyphMetrics::Visible {
              advance: (advance.x(), advance.y()),
              origin: (glyph_bounds.origin_x() as _, glyph_bounds.origin_y() as _),
              position: (
                (glyph_alloc.rectangle.min.x + GLYPH_MARGIN) as _,
                (glyph_alloc.rectangle.min.y + GLYPH_MARGIN) as _,
              ),
              size: (glyph_bounds.width() as _, glyph_bounds.height() as _),
              glyph_id,
              alloc_id: glyph_alloc.id,
              ref_count: 0,
            }
          } else {
            GlyphMetrics::Invisible {
              advance: (advance.x(), advance.y()),
              ref_count: 0,
            }
          };

          self.glyph_metrics_map.insert(glyph_key, glyph_metrics);

          break (
            glyph_key,
            self.glyph_metrics_map.get_mut(&glyph_key).unwrap(),
          );
        };

        match glyph_metrics {
          GlyphMetrics::Visible {
            advance,
            origin,
            position,
            size,
            ref_count,
            ..
          } => {
            *ref_count += 1;

            let glyph = Glyph {
              position: (
                current_glyph_position.0
                  + origin.0 / (self.window_display_scale * RESOLUTION_SCALE),
                current_glyph_position.1
                  + origin.1 / (self.window_display_scale * RESOLUTION_SCALE),
              ),
              size: (
                size.0 / (self.window_display_scale * RESOLUTION_SCALE),
                size.1 / (self.window_display_scale * RESOLUTION_SCALE),
              ),
              color: text.color,
              atlas_position: *position,
              atlas_size: *size,
            };

            current_glyph_position.0 += advance.0 * glyph_key.font_size as f32 / 2048.0;
            current_glyph_position.1 += advance.1 * glyph_key.font_size as f32 / 2048.0;
            Some((glyph_key, glyph))
          }
          GlyphMetrics::Invisible {
            advance, ref_count, ..
          } => {
            *ref_count += 1;
            current_glyph_position.0 += advance.0 * glyph_key.font_size as f32 / 2048.0;
            current_glyph_position.1 += advance.1 * glyph_key.font_size as f32 / 2048.0;
            None
          }
        }
      })
      .unzip();

    let text_width = current_glyph_position.0 - text.position.0;

    let glyph_offset = match text.align {
      Align::Left => 0.0,
      Align::Center => -text_width * 0.5,
      Align::Right => -text_width,
    };

    let glyphs = glyphs
      .into_iter()
      .map(|glyph| Glyph {
        position: (glyph.position.0 + glyph_offset, glyph.position.1),
        ..glyph
      })
      .collect::<Box<_>>();

    let glyph_ids = self.state.get_glyph_renderer_mut().bulk_add_models(glyphs);

    TextId {
      glyph_keys: glyph_keys.into_boxed_slice(),
      glyph_ids,
    }
  }

  fn cleanup_text(&mut self, text_id: TextId) {
    for &glyph_key in &text_id.glyph_keys {
      let glyph_metrics = self.glyph_metrics_map.get_mut(&glyph_key).unwrap();

      let ref_count = match glyph_metrics {
        GlyphMetrics::Visible { ref_count, .. } => ref_count,
        GlyphMetrics::Invisible { ref_count, .. } => ref_count,
      };

      *ref_count -= 1;

      if *ref_count == 0 {
        self.unused_glyph_cache.add(glyph_key);
      }
    }

    self
      .state
      .get_glyph_renderer_mut()
      .bulk_remove_models(&text_id.glyph_ids);
  }
}
