use super::{StackChild, StatelessWidget, Widget};
use crate::{
  models::{HorizontalAlign, Origin},
  widgets::Stack,
};
use optarg2chain::optarg_impl;
use rayon::prelude::*;
use skia_safe::Rect;
use std::{mem, sync::OnceLock};

#[derive(Default)]
pub struct Column<'a> {
  align: HorizontalAlign,
  children: Vec<Widget<'a>>,
  size: OnceLock<(f32, f32)>,
}

#[optarg_impl]
impl<'a> Column<'a> {
  #[optarg_method(ColumnNewBuilder, call)]
  pub fn new(
    #[optarg_default] align: HorizontalAlign,
    #[optarg_default] children: Vec<Widget<'a>>,
  ) -> Self {
    Self {
      align,
      children,
      size: OnceLock::new(),
    }
  }
}

impl<'a> StatelessWidget<'a> for Column<'a> {
  fn get_size(&self) -> (f32, f32) {
    *self.size.get_or_init(|| {
      self
        .children
        .iter()
        .try_fold((0.0, 0.0), |acc, child| {
          let child_size = child.get_size();

          if child_size.1 < 0.0 {
            return Err(());
          }

          Ok((child_size.0.max(acc.0), child_size.1 + acc.1))
        })
        .unwrap_or((-1.0, -1.0))
    })
  }

  fn build(&mut self, constraint: Rect) -> Widget<'a> {
    let mut children = mem::take(&mut self.children)
      .into_par_iter()
      .map(|child| StackChild {
        position: (0.0, 0.0),
        size: (0.0, 0.0),
        origin: Origin::TopLeft,
        child: Some(child),
      })
      .collect::<Vec<_>>();

    let mut y = constraint.y();
    let mut child_index = 0;
    let mut unknown_height_child_index = None;

    while child_index < children.len() {
      let stack_child = &mut children[child_index];
      let child_size = stack_child.child.as_ref().unwrap().get_size();

      let width = if child_size.0 >= 0.0 {
        child_size.0
      } else {
        constraint.width()
      };

      let x = constraint.x()
        + (constraint.width() - width)
          * match self.align {
            HorizontalAlign::Left => 0.0,
            HorizontalAlign::Center => 0.5,
            HorizontalAlign::Right => 1.0,
          };

      if let Some(unknown_height_child_index) = unknown_height_child_index {
        if child_index == unknown_height_child_index {
          // Last child to process where it's height is unknown. Can calculate this child height by filling the
          // remaining space
          let height = y - stack_child.get_position().1;
          stack_child.size = (width, height);
          break;
        }
      }

      stack_child.position = (x, y);

      if child_size.1 >= 0.0 {
        // child height is known and fixed
        let height = child_size.1;
        stack_child.size = (width, height);

        // Ensure no overlapping between children
        if unknown_height_child_index.is_none() {
          y += height;
          child_index += 1;
        } else {
          y -= height;
          child_index -= 1;
          stack_child.position.1 = y;
        }
      } else if unknown_height_child_index.is_none() {
        // child height is unknown
        //
        // Loop through children in reverse order to place the remaining fixed sized children first,
        // so that we can determine the final height for this child
        unknown_height_child_index = Some(child_index);
        y = constraint.y() + constraint.height();
        child_index = children.len() - 1;
      } else {
        // child height is unknown
        //
        // There are multiple children where their height is unknown. Throw error since it is not allowed and
        // we can't determine the final size for these children
        panic!("Multiple variable height children are not allowed");
      }
    }

    Stack { children }.into()
  }
}
