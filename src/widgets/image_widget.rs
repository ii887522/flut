use super::PainterWidget;
use crate::{
  boot::context::{self, ASSET_TX},
  models::AssetReq,
};
use optarg2chain::optarg_impl;
use skia_safe::{Canvas, Color, Paint, Rect};
use std::sync::mpsc::Sender;

pub struct ImageWidget {
  file_path: &'static str,
  size: (f32, f32),
  tint: Color,
}

#[optarg_impl]
impl ImageWidget {
  #[optarg_method(ImageWidgetNewBuilder, call)]
  pub fn new(
    file_path: &'static str,
    #[optarg((-1.0, -1.0))] size: (f32, f32),
    #[optarg(Color::WHITE)] tint: Color,
  ) -> Self {
    ASSET_TX.with(|asset_tx| {
      let asset_tx = asset_tx.get_or_init(|| Sender::clone(context::MAIN_ASSET_TX.get().unwrap()));
      asset_tx.send(AssetReq::LoadImage(file_path)).unwrap();
    });

    Self {
      file_path,
      size,
      tint,
    }
  }
}

impl PainterWidget for ImageWidget {
  fn get_size(&self) -> (f32, f32) {
    self.size
  }

  fn draw(&self, canvas: &Canvas, constraint: Rect) {
    let Some(image) = context::IMAGES.get(self.file_path) else {
      return;
    };

    canvas.draw_image_rect(
      &*image,
      None,
      Rect::from_xywh(
        constraint.x(),
        constraint.y(),
        if self.size.0 < 0.0 {
          constraint.width()
        } else {
          self.size.0
        },
        if self.size.1 < 0.0 {
          constraint.height()
        } else {
          self.size.1
        },
      ),
      Paint::default().set_anti_alias(true).set_color(self.tint),
    );
  }
}
