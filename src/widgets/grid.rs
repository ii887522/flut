use super::{Stack, StackChild, StatelessWidget, Widget};
use crate::models::Origin;
use rayon::prelude::*;
use skia_safe::Rect;
use std::fmt::{self, Debug, Formatter};

pub struct Grid<'a> {
  pub col_count: u16,
  pub row_count: u16,
  pub gap: f32,
  pub builder: Box<dyn Fn(u32) -> Option<Widget<'a>> + 'a + Send + Sync>,
}

impl Debug for Grid<'_> {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
    fmt
      .debug_struct("Grid")
      .field("col_count", &self.col_count)
      .field("row_count", &self.row_count)
      .field("gap", &self.gap)
      .finish_non_exhaustive()
  }
}

impl<'a> StatelessWidget<'a> for Grid<'a> {
  fn build(&mut self, constraint: Rect) -> Widget<'a> {
    let child_size = (
      (constraint.width() - (self.col_count - 1) as f32 * self.gap) / self.col_count as f32,
      (constraint.height() - (self.row_count - 1) as f32 * self.gap) / self.row_count as f32,
    );

    Stack {
      children: (0..self.col_count as u32 * self.row_count as u32)
        .into_par_iter()
        .map(|i| StackChild {
          position: (
            (i % self.col_count as u32) as f32 * (child_size.0 + self.gap) + constraint.x(),
            (i / self.col_count as u32) as f32 * (child_size.1 + self.gap) + constraint.y(),
          ),
          size: child_size,
          origin: Origin::TopLeft,
          child: (self.builder)(i),
        })
        .collect(),
    }
    .into()
  }
}
