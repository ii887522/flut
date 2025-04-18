use crate::{images::StaticImage, models::GlyphMetrics};
use ash::{
  Device,
  vk::{
    BufferImageCopy, Extent3D, Format, ImageAspectFlags, ImageSubresourceLayers, ImageUsageFlags,
    Offset3D,
  },
};
use gpu_allocator::vulkan::Allocator;
use rayon::prelude::*;
use sdl2::{pixels::Color, ttf};
use std::{cell::RefCell, collections::HashMap, ops::RangeInclusive, rc::Rc};

pub(super) struct FontAtlas {
  pub(super) image: StaticImage,
  pub(super) buffer_image_copies: Vec<BufferImageCopy>,
  char_to_glyph_metrics: HashMap<char, GlyphMetrics>,
}

impl FontAtlas {
  pub(super) fn new(
    device: Rc<Device>,
    memory_allocator: Rc<RefCell<Allocator>>,
    file_path: &str,
    font_size: u16,
    chars: RangeInclusive<char>,
    atlas_size: (u32, u32),
  ) -> Self {
    let ttf = ttf::init().unwrap();
    let font = ttf.load_font(file_path, font_size).unwrap();
    let mut glyph_position = (0, 0);
    let mut max_glyph_height = 0;
    let mut buffer_offset = 0;

    let (buffer_image_copies, (char_to_glyph_metrics, pixels)): (Vec<_>, (Vec<_>, Vec<_>)) = chars
      .map(|char| {
        let font_surface = font
          .render_char(char)
          .shaded(Color::WHITE, Color::BLACK)
          .unwrap();

        if glyph_position.0 + font_surface.width() > atlas_size.0 {
          glyph_position = (0, glyph_position.1 + max_glyph_height);
        }

        let buffer_image_copy = BufferImageCopy {
          buffer_offset,
          buffer_row_length: font_surface.pitch(),
          buffer_image_height: font_surface.height(),
          image_subresource: ImageSubresourceLayers {
            aspect_mask: ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
          },
          image_offset: Offset3D {
            x: glyph_position.0 as _,
            y: glyph_position.1 as _,
            z: 0,
          },
          image_extent: Extent3D {
            width: font_surface.width(),
            height: font_surface.height(),
            depth: 1,
          },
        };

        let glyph_metrics = font.find_glyph_metrics(char).unwrap();

        let glyph_metrics = GlyphMetrics {
          position: (glyph_position.0 as _, glyph_position.1 as _),
          size: (font_surface.width() as _, font_surface.height() as _),
          advance: glyph_metrics.advance,
        };

        glyph_position.0 += font_surface.width();
        max_glyph_height = max_glyph_height.max(font_surface.height());
        buffer_offset += (font_surface.pitch() * font_surface.height()) as u64;

        let pixels = font_surface.without_lock().unwrap();
        (buffer_image_copy, ((char, glyph_metrics), pixels.to_vec()))
      })
      .unzip();

    let pixels = pixels.into_par_iter().flatten().collect::<Vec<_>>();

    let image = StaticImage::new(
      device,
      memory_allocator,
      "font_atlas",
      Format::R8_UNORM,
      atlas_size,
      ImageUsageFlags::SAMPLED,
      &pixels,
    );

    let char_to_glyph_metrics = HashMap::from_par_iter(char_to_glyph_metrics);

    Self {
      image,
      buffer_image_copies,
      char_to_glyph_metrics,
    }
  }

  pub(super) fn get_glyph_metrics(&self, char: char) -> Option<&GlyphMetrics> {
    self.char_to_glyph_metrics.get(&char)
  }
}
