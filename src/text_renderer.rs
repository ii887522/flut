use crate::{
  collections::lru_cache::LruCache,
  consts,
  model_sync::ModelSync,
  models::{align::Align, glyph::Glyph, text::Text},
  sampled_image::SampledImage,
  storage_buffer::StorageBuffer,
};
use ash::vk;
use etagere::{AllocId, BucketedAtlasAllocator, Size};
use font_kit::{
  canvas::{Canvas, Format, RasterizationOptions},
  family_name::FamilyName,
  font::Font,
  hinting::HintingOptions,
  properties::{Properties, Stretch, Weight},
  source::SystemSource,
};
use pathfinder_geometry::{
  transform2d::Transform2F,
  vector::{Vector2F, Vector2I},
};
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::{
  borrow::Cow,
  collections::VecDeque,
  hash::{Hash, Hasher},
};

// Settings
const GLYPH_MARGIN: i32 = 1;
const RESOLUTION_SCALE: f32 = 2.0;

#[derive(Clone, PartialEq)]
struct FontKey {
  font_family: Cow<'static, [FamilyName]>,
  font_props: Properties,
}

impl Hash for FontKey {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    let Self {
      ref font_family,
      font_props,
    } = *self;
    let Properties {
      style,
      weight: Weight(weight),
      stretch: Stretch(stretch),
    } = font_props;

    font_family.hash(state);
    style.hash(state);
    weight.to_bits().hash(state);
    stretch.to_bits().hash(state);
  }
}

impl Eq for FontKey {}

#[derive(Clone, PartialEq)]
struct GlyphKey {
  font_key: FontKey,
  ch: char,
  font_size: f32,
}

impl Hash for GlyphKey {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    let Self {
      ref font_key,
      ch,
      font_size,
    } = *self;

    font_key.hash(state);
    ch.hash(state);
    font_size.to_bits().hash(state);
  }
}

impl Eq for GlyphKey {}

#[derive(Clone, Copy)]
enum GlyphMetrics {
  Visible {
    position: (i32, i32),
    size: (u32, u32),
    bearing: (f32, f32),
    advance: (f32, f32),
    alloc_id: AllocId,
    ref_count: u32,
  },
  Invisible {
    advance: (f32, f32),
    ref_count: u32,
  },
}

pub struct TextId {
  glyph_ids: Box<[u32]>,
  glyph_keys: Box<[GlyphKey]>,
  clipped: bool,
}

pub struct TextRenderer {
  glyph_atlas: SampledImage,
  glyph_sync: ModelSync<Glyph>,
  clipped_glyph_sync: ModelSync<Glyph>,
  font_source: SystemSource,
  font_cache: FxHashMap<FontKey, Font>,
  glyph_allocator: BucketedAtlasAllocator,
  glyph_metrics_cache: FxHashMap<GlyphKey, GlyphMetrics>,
  unused_glyph_metrics_cache: LruCache<GlyphKey, GlyphMetrics>,
  changeset_queue: VecDeque<Vec<GlyphKey>>,
  window_scale_factor: f32,
}

impl TextRenderer {
  #[inline]
  pub(super) fn new(
    vk_device: &ash::Device,
    vk_allocator: &vk_mem::Allocator,
    graphics_queue_family_index: u32,
    transfer_queue_family_index: u32,
    window_scale_factor: f32,
    glyph_capacity: usize,
    clipped_glyph_capacity: usize,
    glyph_atlas_size: (u16, u16),
  ) -> (Self, vk::CommandBuffer) {
    let (glyph_atlas_width, glyph_atlas_height) = glyph_atlas_size;
    let glyph_metrics_cache_capacity = (((glyph_atlas_width as usize * glyph_atlas_height as usize)
      >> 10_usize) as f32
      / RESOLUTION_SCALE
      / RESOLUTION_SCALE) as usize;

    let (glyph_atlas, transfer_command_buffer) = SampledImage::new(
      vk_device,
      vk_allocator,
      graphics_queue_family_index,
      transfer_queue_family_index,
      glyph_atlas_width as usize * glyph_atlas_height as usize,
      vk::Extent2D {
        width: u32::from(glyph_atlas_width),
        height: u32::from(glyph_atlas_height),
      },
    );

    (
      Self {
        glyph_atlas,
        glyph_sync: ModelSync::new(glyph_capacity),
        clipped_glyph_sync: ModelSync::new(clipped_glyph_capacity),
        font_source: SystemSource::new(),
        font_cache: FxHashMap::default(),
        glyph_allocator: BucketedAtlasAllocator::new(Size::new(
          glyph_atlas_width.into(),
          glyph_atlas_height.into(),
        )),
        glyph_metrics_cache: FxHashMap::with_capacity_and_hasher(
          glyph_metrics_cache_capacity,
          FxBuildHasher,
        ),
        unused_glyph_metrics_cache: LruCache::with_capacity(glyph_metrics_cache_capacity),
        changeset_queue: VecDeque::from_iter([vec![]]),
        window_scale_factor,
      },
      transfer_command_buffer,
    )
  }

  #[inline]
  pub(super) const fn get_glyph_count(&self) -> usize {
    self.glyph_sync.get_model_count()
  }

  #[inline]
  pub(super) const fn get_clipped_glyph_count(&self) -> usize {
    self.clipped_glyph_sync.get_model_count()
  }

  #[inline]
  const fn get_glyph_sync(&mut self, clipped: bool) -> &mut ModelSync<Glyph> {
    if clipped {
      &mut self.clipped_glyph_sync
    } else {
      &mut self.glyph_sync
    }
  }

  #[inline]
  pub(super) const fn get_glyph_atlas(&mut self) -> &mut SampledImage {
    &mut self.glyph_atlas
  }

  pub(super) fn sync_to(
    &mut self,
    model_buffer: &StorageBuffer,
    vk_device: &ash::Device,
    model_buffer_offset: usize,
    clipped_model_buffer_offset: usize,
    graphics_queue_family_index: u32,
    transfer_queue_family_index: u32,
  ) -> Box<[vk::CommandBuffer]> {
    let all_changeset = self
      .changeset_queue
      .iter()
      .flatten()
      .cloned()
      .collect::<Vec<_>>();

    let mut transfer_command_buffers = vec![];

    if !all_changeset.is_empty() {
      let mut regions = Vec::with_capacity(all_changeset.len());
      let mut pixels = Vec::with_capacity(
        ((all_changeset.len() << 10) as f32 * RESOLUTION_SCALE * RESOLUTION_SCALE) as usize,
      );

      for glyph_key in all_changeset {
        let GlyphKey {
          ref font_key,
          ch,
          font_size,
        } = glyph_key;

        let font = &self.font_cache[font_key];

        let glyph_id = font.glyph_for_char(ch).unwrap_or_else(|| {
          eprintln!("Failed to find glyph for char '{ch}'");
          font.glyph_for_char('?').unwrap()
        });

        let GlyphMetrics::Visible {
          position: (glyph_x, glyph_y),
          size: (glyph_width, glyph_height),
          bearing: (bearing_x, bearing_y),
          ..
        } = self.glyph_metrics_cache[&glyph_key]
        else {
          unreachable!("Rasterizing invisible glyph is not allowed");
        };

        let mut canvas = Canvas::new(
          Vector2I::new(
            glyph_width.cast_signed() + (GLYPH_MARGIN << 1),
            glyph_height.cast_signed() + (GLYPH_MARGIN << 1),
          ),
          Format::A8,
        );

        font
          .rasterize_glyph(
            &mut canvas,
            glyph_id,
            font_size * self.window_scale_factor * RESOLUTION_SCALE,
            Transform2F::from_translation(Vector2F::new(
              (bearing_x * self.window_scale_factor)
                .mul_add(-RESOLUTION_SCALE, GLYPH_MARGIN as f32),
              (bearing_y * self.window_scale_factor)
                .mul_add(-RESOLUTION_SCALE, GLYPH_MARGIN as f32),
            )),
            HintingOptions::Vertical(font_size * self.window_scale_factor),
            RasterizationOptions::GrayscaleAa,
          )
          .unwrap();

        regions.push(vk::BufferImageCopy2 {
          buffer_offset: pixels.len() as u64,
          buffer_row_length: canvas.stride as u32,
          buffer_image_height: canvas.size.y() as u32,
          image_subresource: vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
          },
          image_offset: vk::Offset3D {
            x: glyph_x - GLYPH_MARGIN,
            y: glyph_y - GLYPH_MARGIN,
            z: 0,
          },
          image_extent: vk::Extent3D {
            width: glyph_width + (GLYPH_MARGIN << 1) as u32,
            height: glyph_height + (GLYPH_MARGIN << 1) as u32,
            depth: 1,
          },
          ..Default::default()
        });

        pixels.extend(canvas.pixels);
      }

      let transfer_command_buffer = self.glyph_atlas.write(
        vk_device,
        graphics_queue_family_index,
        transfer_queue_family_index,
        &pixels,
        &regions,
      );

      transfer_command_buffers.push(transfer_command_buffer);
    }

    if let Some(transfer_command_buffer) =
      self
        .glyph_sync
        .sync_to(model_buffer, vk_device, model_buffer_offset, false)
    {
      transfer_command_buffers.push(transfer_command_buffer);
    }

    if let Some(transfer_command_buffer) =
      self
        .clipped_glyph_sync
        .sync_to(model_buffer, vk_device, clipped_model_buffer_offset, true)
    {
      transfer_command_buffers.push(transfer_command_buffer);
    }

    if self.changeset_queue.len() >= consts::MAX_IN_FLIGHT_FRAME_COUNT {
      self.changeset_queue.pop_front();
    }

    self.changeset_queue.push_back(vec![]);
    transfer_command_buffers.into_boxed_slice()
  }

  pub(super) fn add_text(&mut self, text: Text, clipped: bool) -> TextId {
    let font_key = FontKey {
      font_family: text.font_family,
      font_props: text.font_props,
    };

    let font = self.font_cache.entry(font_key.clone()).or_insert_with_key(
      |&FontKey {
         ref font_family,
         font_props,
       }| {
        self
          .font_source
          .select_best_match(font_family, &font_props)
          .unwrap()
          .load()
          .unwrap()
      },
    );

    let changeset = self.changeset_queue.back_mut().unwrap();
    let mut glyph_position = text.position;

    let (glyph_keys, glyphs): (Vec<_>, Vec<_>) = text
      .text
      .chars()
      .map(|ch| {
        let glyph_key = GlyphKey {
          font_key: font_key.clone(),
          ch,
          font_size: text.font_size,
        };

        if !self.glyph_metrics_cache.contains_key(&glyph_key) {
          let glyph_id = font.glyph_for_char(ch).unwrap_or_else(|| {
            eprintln!("Failed to find glyph for char '{ch}'");
            font.glyph_for_char('?').unwrap()
          });

          let advance = font.advance(glyph_id).unwrap();

          let advance = (
            advance.x() / 2048.0 * text.font_size,
            advance.y() / 2048.0 * text.font_size,
          );

          let glyph_bounds = font
            .raster_bounds(
              glyph_id,
              text.font_size * self.window_scale_factor * RESOLUTION_SCALE,
              Transform2F::default(),
              HintingOptions::Vertical(text.font_size * self.window_scale_factor),
              RasterizationOptions::GrayscaleAa,
            )
            .unwrap();

          let glyph_metrics = if glyph_bounds.width() > 0_i32 && glyph_bounds.height() > 0_i32 {
            let mut unused_glyph_evict_count = 1_usize;

            let glyph_alloc = loop {
              if let Some(glyph_alloc) = self.glyph_allocator.allocate(Size::new(
                glyph_bounds.width() + (GLYPH_MARGIN << 1_i32),
                glyph_bounds.height() + (GLYPH_MARGIN << 1_i32),
              )) {
                break glyph_alloc;
              }

              for i in 0..unused_glyph_evict_count {
                let Some((unused_glyph_key, glyph_metrics)) =
                  self.unused_glyph_metrics_cache.evict_one()
                else {
                  assert!(i > 0, "Failed to allocate from glyph atlas");
                  break;
                };

                self.glyph_metrics_cache.remove(&unused_glyph_key);

                if let GlyphMetrics::Visible {
                  alloc_id: unused_alloc_id,
                  ..
                } = glyph_metrics
                {
                  self.glyph_allocator.deallocate(unused_alloc_id);
                }
              }

              unused_glyph_evict_count <<= 1_usize;
            };

            changeset.push(glyph_key.clone());

            GlyphMetrics::Visible {
              position: (
                glyph_alloc.rectangle.min.x + GLYPH_MARGIN,
                glyph_alloc.rectangle.min.y + GLYPH_MARGIN,
              ),
              size: (
                (glyph_alloc.rectangle.width() - (GLYPH_MARGIN << 1_i32)) as u32,
                (glyph_alloc.rectangle.height() - (GLYPH_MARGIN << 1_i32)) as u32,
              ),
              bearing: (
                glyph_bounds.min_x() as f32 / self.window_scale_factor / RESOLUTION_SCALE,
                glyph_bounds.min_y() as f32 / self.window_scale_factor / RESOLUTION_SCALE,
              ),
              advance,
              alloc_id: glyph_alloc.id,
              ref_count: 0,
            }
          } else {
            GlyphMetrics::Invisible {
              advance,
              ref_count: 0,
            }
          };

          self
            .glyph_metrics_cache
            .insert(glyph_key.clone(), glyph_metrics);
        }

        let (glyph, ref_count, (advance_x, advance_y)) =
          match *self.glyph_metrics_cache.get_mut(&glyph_key).unwrap() {
            GlyphMetrics::Visible {
              position: (glyph_x, glyph_y),
              size: (glyph_width, glyph_height),
              bearing: (bearing_x, bearing_y),
              advance,
              alloc_id: _,
              ref mut ref_count,
            } => (
              Some(Glyph {
                position: (
                  glyph_position.0 + bearing_x,
                  glyph_position.1 + bearing_y,
                  glyph_position.2,
                ),
                color: text.color,
                size: (
                  glyph_width as f32 / self.window_scale_factor / RESOLUTION_SCALE,
                  glyph_height as f32 / self.window_scale_factor / RESOLUTION_SCALE,
                ),
                atlas_position: (glyph_x as f32, glyph_y as f32),
              }),
              ref_count,
              advance,
            ),
            GlyphMetrics::Invisible {
              advance,
              ref mut ref_count,
            } => (None, ref_count, advance),
          };

        if *ref_count == 0 {
          self.unused_glyph_metrics_cache.remove(&glyph_key);
        }

        *ref_count += 1;
        glyph_position.0 += advance_x;
        glyph_position.1 += advance_y;
        (glyph_key, glyph)
      })
      .unzip();

    let text_width = glyph_position.0 - text.position.0;

    let glyph_offset_x = match text.align {
      Align::Left => 0.0,
      Align::Center => -text_width * 0.5,
      Align::Right => -text_width,
    };

    let glyphs = glyphs
      .into_iter()
      .filter_map(|glyph| {
        glyph.map(|glyph| {
          let (glyph_x, glyph_y, glyph_z) = glyph.position;

          Glyph {
            position: (glyph_x + glyph_offset_x, glyph_y, glyph_z),
            ..glyph
          }
        })
      })
      .collect::<Box<_>>();

    let glyph_ids = self.get_glyph_sync(clipped).bulk_add_models(glyphs);

    TextId {
      glyph_ids,
      glyph_keys: glyph_keys.into_boxed_slice(),
      clipped,
    }
  }

  pub(super) fn remove_text(&mut self, text_id: TextId) {
    let TextId {
      glyph_ids,
      glyph_keys,
      clipped,
    } = text_id;

    self.get_glyph_sync(clipped).bulk_remove_models(&glyph_ids);

    for glyph_key in glyph_keys {
      let glyph_metrics = self.glyph_metrics_cache.get_mut(&glyph_key).unwrap();

      let ref_count = match *glyph_metrics {
        GlyphMetrics::Visible {
          ref mut ref_count, ..
        }
        | GlyphMetrics::Invisible {
          ref mut ref_count, ..
        } => ref_count,
      };

      *ref_count -= 1;

      if *ref_count > 0 {
        continue;
      }

      self
        .unused_glyph_metrics_cache
        .insert(glyph_key, *glyph_metrics);
    }
  }

  #[inline]
  pub(super) fn drop(self, vk_device: &ash::Device, vk_allocator: &vk_mem::Allocator) {
    self.glyph_atlas.drop(vk_device, vk_allocator);
  }
}
