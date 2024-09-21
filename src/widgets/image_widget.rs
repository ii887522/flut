use super::PainterWidget;
use crate::boot::context;
use optarg2chain::optarg_impl;
use skia_safe::{Canvas, Data, Image, Paint, Rect};

#[derive(Debug)]
pub struct ImageWidget {
  file_path: &'static str,
  size: (f32, f32),
}

#[optarg_impl]
impl ImageWidget {
  #[optarg_method(ImageWidgetNewBuilder, call)]
  pub fn new(file_path: &'static str, #[optarg((-1.0, -1.0))] size: (f32, f32)) -> Self {
    context::IMAGES.with_borrow_mut(|images| {
      images.entry(file_path).or_insert_with(|| {
        let image_data = Data::from_filename(file_path).unwrap();
        Image::from_encoded(image_data).unwrap()
      });
    });

    Self { file_path, size }
  }
}

impl PainterWidget for ImageWidget {
  fn get_size(&self) -> (f32, f32) {
    self.size
  }

  fn draw(&self, canvas: &Canvas, constraint: Rect) {
    context::IMAGES.with_borrow(|images| {
      canvas.draw_image_rect(
        &images[self.file_path],
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
        Paint::default().set_anti_alias(true),
      );
    });
  }
}
