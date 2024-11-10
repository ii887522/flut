use super::{BuilderWidget, Stack, Widget};
use crate::{models::VerticalAlign, widgets::StackChild};
use rayon::prelude::*;
use skia_safe::Rect;
use std::mem;

struct RemainingChild<'a> {
  left_x: f32,
  right_x: f32,
  child: Widget<'a>,
}

#[derive(Default)]
pub struct Row<'a> {
  pub align: VerticalAlign,
  pub children: Vec<Widget<'a>>,
}

impl Row<'_> {
  fn calc_child_constraint(
    parent_x: &mut f32,
    child: &Widget<'_>,
    parent_constraint: Rect,
    align: VerticalAlign,
    is_scan_forward: bool,
  ) -> Option<Rect> {
    let child_size = child.get_size();

    if child_size.0 < 0.0 {
      // The width of this child will be determined by the remaining width gap in this Row after scan forward
      // and backward
      return None;
    }

    let child_height = if child_size.1 < 0.0 {
      parent_constraint.height()
    } else {
      child_size.1
    };

    let child_width = child_size.0;

    let child_y = match align {
      VerticalAlign::Top => parent_constraint.y(),
      VerticalAlign::Center => {
        parent_constraint.y() + (parent_constraint.height() - child_height) * 0.5
      }
      VerticalAlign::Bottom => parent_constraint.y() + parent_constraint.height() - child_height,
    };

    if is_scan_forward {
      let rect = Rect::from_xywh(*parent_x, child_y, child_width, child_height);
      *parent_x += child_width;
      Some(rect)
    } else {
      *parent_x -= child_width;
      let rect = Rect::from_xywh(*parent_x, child_y, child_width, child_height);
      Some(rect)
    }
  }
}

impl<'a> BuilderWidget<'a> for Row<'a> {
  fn build(&mut self, constraint: Rect) -> Widget<'a> {
    let child_count = self.children.len();
    let mut children = mem::take(&mut self.children).into_iter();
    let mut remaining_child = None;
    let mut stack_children = Vec::with_capacity(child_count);

    let left_children = children
      .by_ref()
      .scan(constraint.x(), |x, child| {
        let Some(child_constraint) =
          Self::calc_child_constraint(x, &child, constraint, self.align, true)
        else {
          remaining_child = Some(RemainingChild {
            left_x: *x,
            right_x: constraint.x() + constraint.width(),
            child,
          });

          return None;
        };

        Some(StackChild::new(child_constraint, child))
      })
      .collect::<Vec<_>>();

    stack_children.par_extend(left_children);

    let Some(mut remaining_child) = remaining_child else {
      return Stack {
        children: stack_children,
      }
      .into();
    };

    let right_children = children
      .rev()
      .scan(constraint.x() + constraint.width(), |x, child| {
        let stack_child = Self::calc_child_constraint(x, &child, constraint, self.align, false)
          .map(|child_constraint| StackChild::new(child_constraint, child));

        remaining_child.right_x = *x;
        stack_child
      })
      .collect::<Vec<_>>();

    let remaining_child_size = remaining_child.child.get_size();

    let remaining_child_height = if remaining_child_size.1 < 0.0 {
      constraint.height()
    } else {
      remaining_child_size.1
    };

    let remaining_child_y = match self.align {
      VerticalAlign::Top => constraint.y(),
      VerticalAlign::Center => {
        constraint.y() + (constraint.height() - remaining_child_height) * 0.5
      }
      VerticalAlign::Bottom => constraint.y() + constraint.height() - remaining_child_height,
    };

    let remaining_stack_child = StackChild {
      position: (remaining_child.left_x, remaining_child_y),
      size: (
        remaining_child.right_x - remaining_child.left_x,
        remaining_child_height,
      ),
      child: remaining_child.child,
    };

    stack_children.push(remaining_stack_child);
    stack_children.par_extend(right_children);

    Stack {
      children: stack_children,
    }
    .into()
  }
}
