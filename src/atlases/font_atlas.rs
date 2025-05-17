use crate::{consts, images::StaticImage, models::GlyphMetrics};
use ash::{
  Device,
  vk::{
    BufferImageCopy, Extent3D, Format, ImageAspectFlags, ImageSubresourceLayers, ImageUsageFlags,
    Offset3D,
  },
};
use gpu_allocator::vulkan::Allocator;
use rayon::prelude::*;
use sdl2::{pixels::Color, ttf::Sdl2TtfContext};
use std::{cell::RefCell, collections::HashMap, ops::RangeInclusive, rc::Rc, sync::Arc};

pub(crate) struct FontAtlas {
  pub(crate) font_size: u16,
  pub(crate) image: StaticImage,
  pub(crate) buffer_image_copies: Vec<BufferImageCopy>,
  char_to_glyph_metrics: HashMap<char, GlyphMetrics>,
}

impl FontAtlas {
  pub(crate) fn new(
    device: Arc<Device>,
    memory_allocator: Rc<RefCell<Allocator>>,
    ttf: &Sdl2TtfContext,
    file_path: &str,
    font_size: u16,
    chars: RangeInclusive<char>,
    atlas_size: (u32, u32),
  ) -> Self {
    let font = ttf.load_font(file_path, font_size).unwrap();
    let mut glyph_position = (consts::GLYPH_PADDING, consts::GLYPH_PADDING);
    let mut max_glyph_height = 0;
    let mut buffer_offset = 0;

    let (buffer_image_copies, (char_to_glyph_metrics, pixels)): (Vec<_>, (Vec<_>, Vec<_>)) = chars
      .map(|char| {
        let font_surface = font
          .render_char(char)
          .shaded(Color::WHITE, Color::BLACK)
          .unwrap();

        if glyph_position.0 + font_surface.width() + consts::GLYPH_PADDING > atlas_size.0 {
          glyph_position = (
            consts::GLYPH_PADDING,
            glyph_position.1 + max_glyph_height + consts::GLYPH_PADDING,
          );
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

        glyph_position.0 += font_surface.width() + consts::GLYPH_PADDING;
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
      ImageAspectFlags::COLOR,
      &pixels,
    );

    let char_to_glyph_metrics = HashMap::from_par_iter(char_to_glyph_metrics);

    Self {
      font_size,
      image,
      buffer_image_copies,
      char_to_glyph_metrics,
    }
  }

  pub(crate) fn get_glyph_metrics(&self, char: char) -> Option<&GlyphMetrics> {
    self.char_to_glyph_metrics.get(&char)
  }
}
