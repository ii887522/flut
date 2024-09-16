use super::{StackChild, StatelessWidget, Widget};
use crate::{
  models::{Origin, VerticalAlign},
  widgets::Stack,
};
use optarg2chain::optarg_impl;
use rayon::prelude::*;
use skia_safe::Rect;
use std::{mem, sync::OnceLock};

#[derive(Debug, Default)]
pub struct Row<'a> {
  align: VerticalAlign,
  children: Vec<Widget<'a>>,
  size: OnceLock<(f32, f32)>,
}

#[optarg_impl]
impl<'a> Row<'a> {
  #[optarg_method(RowNewBuilder, call)]
  pub fn new(
    #[optarg_default] align: VerticalAlign,
    #[optarg_default] children: Vec<Widget<'a>>,
  ) -> Self {
    Self {
      align,
      children,
      size: OnceLock::new(),
    }
  }
}

impl<'a> StatelessWidget<'a> for Row<'a> {
  fn get_size(&self) -> (f32, f32) {
    *self.size.get_or_init(|| {
      self
        .children
        .iter()
        .try_fold((0.0, 0.0), |acc, child| {
          let child_size = child.get_size();

          if child_size.0 < 0.0 {
            Err(())
          } else {
            Ok((child_size.0 + acc.0, child_size.1.max(acc.1)))
          }
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

    let mut x = constraint.x();
    let mut child_index = 0;
    let mut unknown_width_child_index = None;

    while child_index < children.len() {
      let stack_child = &mut children[child_index];
      let child_size = stack_child.child.as_ref().unwrap().get_size();

      let height = if child_size.1 >= 0.0 {
        child_size.1
      } else {
        constraint.height()
      };

      let y = constraint.y()
        + (constraint.height() - height)
          * match self.align {
            VerticalAlign::Top => 0.0,
            VerticalAlign::Middle => 0.5,
            VerticalAlign::Bottom => 1.0,
          };

      if let Some(unknown_width_child_index) = unknown_width_child_index {
        if child_index == unknown_width_child_index {
          // Last child to process where it's width is unknown. Can calculate this child width by filling the
          // remaining space
          let width = x - stack_child.get_position().0;
          stack_child.size = (width, height);
          break;
        }
      }

      stack_child.position = (x, y);

      if child_size.0 >= 0.0 {
        // child width is known and fixed
        let width = child_size.0;
        stack_child.size = (width, height);

        // Ensure no overlapping between children
        if unknown_width_child_index.is_none() {
          x += width;
          child_index += 1;
        } else {
          x -= width;
          child_index -= 1;
          stack_child.position.0 = x;
        }
      } else if unknown_width_child_index.is_none() {
        // child width is unknown
        //
        // Loop through children in reverse order to place the remaining fixed sized children first,
        // so that we can determine the final width for this child
        unknown_width_child_index = Some(child_index);
        x = constraint.x() + constraint.width();
        child_index = children.len() - 1;
      } else {
        // child width is unknown
        //
        // There are multiple children where their width is unknown. Throw error since it is not allowed and
        // we can't determine the final size for these children
        panic!("Multiple variable width children are not allowed");
      }
    }

    Stack { children }.into()
  }
}
