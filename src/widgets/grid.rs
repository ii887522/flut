use super::{BuilderWidget, Stack, StackChild, Widget};
use rayon::prelude::*;
use skia_safe::Rect;

pub struct Grid<'a> {
  pub col_count: u32,
  pub row_count: u32,
  pub gap: f32,
  pub builder: Box<dyn Fn(u32) -> Widget<'a> + 'a + Send + Sync>,
}

impl<'a> BuilderWidget<'a> for Grid<'a> {
  fn build(&self, constraint: Rect) -> Widget<'a> {
    let child_size = (
      (constraint.width() - self.gap * (self.col_count - 1) as f32) / self.col_count as f32,
      (constraint.height() - self.gap * (self.row_count - 1) as f32) / self.row_count as f32,
    );

    Stack {
      children: (0..self.col_count * self.row_count)
        .into_par_iter()
        .map(|index| StackChild {
          position: (
            constraint.x() + (index % self.col_count) as f32 * (child_size.0 + self.gap),
            constraint.y() + (index / self.col_count) as f32 * (child_size.1 + self.gap),
          ),
          size: child_size,
          child: (self.builder)(index),
        })
        .collect(),
    }
    .into()
  }
}
