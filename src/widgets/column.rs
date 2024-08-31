use super::{StackChild, StatelessWidget, Widget};
use crate::{models::HorizontalAlign, widgets::Stack};
use skia_safe::Rect;
use std::mem;

#[derive(Debug, Default)]
pub struct Column<'a> {
  pub align: HorizontalAlign,
  pub children: Vec<Widget<'a>>,
}

impl<'a> StatelessWidget<'a> for Column<'a> {
  fn build(&mut self, constraint: Rect) -> Widget<'a> {
    let mut src_children = mem::take(&mut self.children).into_iter();
    let mut dst_children = Vec::with_capacity(self.children.len());
    let mut y = constraint.y();
    let mut old_y = 0.0;
    let mut unknown_height_child = None;

    while let Some(child) = if unknown_height_child.is_none() {
      src_children.next()
    } else {
      src_children.next_back()
    } {
      let child_size = child.get_size();

      if child_size.1 > 0.0 {
        // child height is known and fixed
        let width = if child_size.0 == 0.0 {
          constraint.width()
        } else {
          child_size.0
        };

        let height = child_size.1;

        let x = constraint.x()
          + (constraint.width() - width)
            * match self.align {
              HorizontalAlign::Left => 0.0,
              HorizontalAlign::Center => 0.5,
              HorizontalAlign::Right => 1.0,
            };

        // Ensure no overlapping between children
        if unknown_height_child.is_some() {
          y -= height;
        }

        dst_children.push(StackChild {
          position: (x, y),
          size: (width, height),
          child: Some(child),
        });

        // Ensure no overlapping between children
        if unknown_height_child.is_none() {
          y += height;
        }
      } else if unknown_height_child.is_none() {
        // child height is unknown
        //
        // Put it somewhere so that after all fixed sized children are placed, we can get it back and place it in
        // the remaining space
        //
        // Also loop through src_children in reverse order to place the remaining fixed sized children first,
        // so that we can determine the final height for this child
        old_y = y;
        y = constraint.y() + constraint.height();
        unknown_height_child = Some(child);
      } else {
        // child height is unknown
        //
        // There are multiple children where their height is unknown. Throw error since it is not allowed and
        // we can't determine the final size for these children
        panic!("Multiple variable height children are not allowed");
      }
    }

    if let Some(child) = unknown_height_child {
      let child_size = child.get_size();

      let width = if child_size.0 == 0.0 {
        constraint.width()
      } else {
        child_size.0
      };

      let x = constraint.x()
        + (constraint.width() - width)
          * match self.align {
            HorizontalAlign::Left => 0.0,
            HorizontalAlign::Center => 0.5,
            HorizontalAlign::Right => 1.0,
          };

      dst_children.push(StackChild {
        position: (x, old_y),
        size: (width, y - old_y),
        child: Some(child),
      });
    }

    Stack {
      children: dst_children,
    }
    .into()
  }
}
