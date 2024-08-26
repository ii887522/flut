use super::{StackChild, StatelessWidget, Widget};
use crate::widgets::Stack;
use skia_safe::Rect;
use std::mem;

#[derive(Debug, Default)]
pub struct Row<'a> {
  pub children: Vec<Widget<'a>>,
}

impl<'a> StatelessWidget<'a> for Row<'a> {
  fn build(&mut self, constraint: Rect) -> Widget<'a> {
    let mut src_children = mem::take(&mut self.children).into_iter();
    let mut dst_children = Vec::with_capacity(self.children.len());
    let mut x = constraint.x();
    let mut old_x = 0.0;
    let mut unknown_width_child = None;

    while let Some(child) = if unknown_width_child.is_none() {
      src_children.next()
    } else {
      src_children.next_back()
    } {
      let child_size = child.get_size();

      if child_size.0 > 0.0 {
        // child width is known and fixed
        let width = child_size.0;

        let height = if child_size.1 == 0.0 {
          constraint.height()
        } else {
          child_size.1
        };

        dst_children.push(StackChild {
          position: (x, constraint.y()),
          size: (width, height),
          child: Some(child),
        });

        // Ensure no overlapping between children
        x += width;
      } else if unknown_width_child.is_none() {
        // child width is unknown
        //
        // Put it somewhere so that after all fixed sized children are placed, we can get it back and place it in
        // the remaining space
        //
        // Also loop through src_children in reverse order to place the remaining fixed sized children first,
        // so that we can determine the final width for this child
        old_x = x;
        x = constraint.x() + constraint.width();
        unknown_width_child = Some(child);
      } else {
        // child width is unknown
        //
        // There are multiple children where their width is unknown. Throw error since it is not allowed and
        // we can't determine the final size for these children
        panic!("Multiple variable width children are not allowed");
      }
    }

    if let Some(child) = unknown_width_child {
      let child_size = child.get_size();

      let height = if child_size.1 == 0.0 {
        constraint.height()
      } else {
        child_size.1
      };

      dst_children.push(StackChild {
        position: (old_x, constraint.y()),
        size: (x - old_x, height),
        child: Some(child),
      });
    }

    Stack {
      children: dst_children,
    }
    .into()
  }
}
