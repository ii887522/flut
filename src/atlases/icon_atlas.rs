use crate::{consts, images::DynamicImage, models::GlyphMetrics};
use ash::{
  Device,
  vk::{
    BufferImageCopy, CommandPool, Extent3D, Format, ImageAspectFlags, ImageSubresourceLayers,
    ImageUsageFlags, Offset3D, Queue,
  },
};
use gpu_allocator::vulkan::Allocator;
use rayon::prelude::*;
use sdl2::{pixels::Color, ttf::Font};
use std::{cell::RefCell, collections::HashMap, mem, rc::Rc, sync::Arc};

pub(crate) struct IconAtlas {
  pub(crate) image: DynamicImage,
  font: Font<'static, 'static>,
  atlas_size: (u32, u32),
  code_point_to_glyph_metrics: HashMap<u16, GlyphMetrics>,
  buffer_image_copies: Vec<BufferImageCopy>,
  pixels: Vec<u8>,
  buffer_offset: u64,
  icon_position: (u32, u32),
  max_icon_height: u32,
}

impl IconAtlas {
  pub(crate) fn new(
    device: Arc<Device>,
    memory_allocator: Rc<RefCell<Allocator>>,
    transfer_command_pool: CommandPool,
    file_path: &str,
    font_size: u16,
    atlas_size: (u32, u32),
  ) -> Self {
    let font = consts::TTF.load_font(file_path, font_size).unwrap();

    let image = DynamicImage::new(
      device,
      memory_allocator,
      transfer_command_pool,
      "icon_atlas",
      Format::R8_UNORM,
      (atlas_size.0 * atlas_size.1) as _,
      atlas_size,
      ImageUsageFlags::SAMPLED,
    );

    let icon_cap = ((atlas_size.0 / font_size as u32) * (atlas_size.1 / font_size as u32)) as _;
    let code_point_to_glyph_metrics = HashMap::with_capacity(icon_cap);
    let buffer_image_copies = Vec::with_capacity(icon_cap);
    let pixels = Vec::with_capacity((atlas_size.0 * atlas_size.1) as _);

    Self {
      image,
      font,
      atlas_size,
      code_point_to_glyph_metrics,
      buffer_image_copies,
      pixels,
      buffer_offset: 0,
      icon_position: (consts::GLYPH_PADDING, consts::GLYPH_PADDING),
      max_icon_height: 0,
    }
  }

  pub(crate) fn get_glyph_metrics(&mut self, code_point: u16) -> Option<GlyphMetrics> {
    if let Some(&glyph_metrics) = self.code_point_to_glyph_metrics.get(&code_point) {
      return Some(glyph_metrics);
    }

    let code_point_char = char::from_u32(code_point as _)?;
    let glyph_metrics = self.font.find_glyph_metrics(code_point_char)?;

    let font_surface = self
      .font
      .render_char(code_point_char)
      .shaded(Color::WHITE, Color::BLACK)
      .unwrap();

    if self.icon_position.0 + font_surface.width() + consts::GLYPH_PADDING > self.atlas_size.0 {
      self.icon_position = (
        consts::GLYPH_PADDING,
        self.icon_position.1 + self.max_icon_height + consts::GLYPH_PADDING,
      );
    }

    let buffer_image_copy = BufferImageCopy {
      buffer_offset: self.buffer_offset,
      buffer_row_length: font_surface.pitch(),
      buffer_image_height: font_surface.height(),
      image_subresource: ImageSubresourceLayers {
        aspect_mask: ImageAspectFlags::COLOR,
        mip_level: 0,
        base_array_layer: 0,
        layer_count: 1,
      },
      image_offset: Offset3D {
        x: self.icon_position.0 as _,
        y: self.icon_position.1 as _,
        z: 0,
      },
      image_extent: Extent3D {
        width: font_surface.width(),
        height: font_surface.height(),
        depth: 1,
      },
    };

    let glyph_metrics = GlyphMetrics {
      position: (self.icon_position.0 as _, self.icon_position.1 as _),
      size: (font_surface.width() as _, font_surface.height() as _),
      advance: glyph_metrics.advance,
    };

    let pixels = font_surface.without_lock().unwrap();
    self.buffer_image_copies.push(buffer_image_copy);

    self
      .code_point_to_glyph_metrics
      .insert(code_point, glyph_metrics);

    self.pixels.par_extend(pixels);
    self.buffer_offset += (font_surface.pitch() * font_surface.height()) as u64;
    self.icon_position.0 += font_surface.width() + consts::GLYPH_PADDING;
    self.max_icon_height = self.max_icon_height.max(font_surface.height());
    Some(glyph_metrics)
  }

  pub(crate) fn draw(
    &mut self,
    transfer_queue: Queue,
    graphics_queue_family_index: u32,
    transfer_queue_family_index: u32,
  ) -> bool {
    if self.pixels.is_empty() {
      return false;
    }

    self.image.draw(
      transfer_queue,
      mem::take(&mut self.pixels),
      mem::take(&mut self.buffer_image_copies),
      graphics_queue_family_index,
      transfer_queue_family_index,
    );

    self.buffer_offset = 0;
    true
  }
}
