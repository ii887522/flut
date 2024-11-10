use super::{BuilderWidget, Stack, Widget};
use crate::{models::HorizontalAlign, widgets::StackChild};
use rayon::prelude::*;
use skia_safe::Rect;
use std::mem;

struct RemainingChild<'a> {
  top_y: f32,
  bottom_y: f32,
  child: Widget<'a>,
}

#[derive(Default)]
pub struct Column<'a> {
  pub align: HorizontalAlign,
  pub children: Vec<Widget<'a>>,
}

impl Column<'_> {
  fn calc_child_constraint(
    parent_y: &mut f32,
    child: &Widget<'_>,
    parent_constraint: Rect,
    align: HorizontalAlign,
    is_scan_forward: bool,
  ) -> Option<Rect> {
    let child_size = child.get_size();

    if child_size.1 < 0.0 {
      // The height of this child will be determined by the remaining height gap in this Column after scan forward
      // and backward
      return None;
    }

    let child_width = if child_size.0 < 0.0 {
      parent_constraint.width()
    } else {
      child_size.0
    };

    let child_height = child_size.1;

    let child_x = match align {
      HorizontalAlign::Left => parent_constraint.x(),
      HorizontalAlign::Center => {
        parent_constraint.x() + (parent_constraint.width() - child_width) * 0.5
      }
      HorizontalAlign::Right => parent_constraint.x() + parent_constraint.width() - child_width,
    };

    if is_scan_forward {
      let rect = Rect::from_xywh(child_x, *parent_y, child_width, child_height);
      *parent_y += child_height;
      Some(rect)
    } else {
      *parent_y -= child_height;
      let rect = Rect::from_xywh(child_x, *parent_y, child_width, child_height);
      Some(rect)
    }
  }
}

impl<'a> BuilderWidget<'a> for Column<'a> {
  fn build(&mut self, constraint: Rect) -> Widget<'a> {
    let child_count = self.children.len();
    let mut children = mem::take(&mut self.children).into_iter();
    let mut remaining_child = None;
    let mut stack_children = Vec::with_capacity(child_count);

    let top_children = children
      .by_ref()
      .scan(constraint.y(), |y, child| {
        let Some(child_constraint) =
          Self::calc_child_constraint(y, &child, constraint, self.align, true)
        else {
          remaining_child = Some(RemainingChild {
            top_y: *y,
            bottom_y: constraint.y() + constraint.height(),
            child,
          });

          return None;
        };

        Some(StackChild::new(child_constraint, child))
      })
      .collect::<Vec<_>>();

    stack_children.par_extend(top_children);

    let Some(mut remaining_child) = remaining_child else {
      return Stack {
        children: stack_children,
      }
      .into();
    };

    let bottom_children = children
      .rev()
      .scan(constraint.y() + constraint.height(), |y, child| {
        let stack_child = Self::calc_child_constraint(y, &child, constraint, self.align, false)
          .map(|child_constraint| StackChild::new(child_constraint, child));

        remaining_child.bottom_y = *y;
        stack_child
      })
      .collect::<Vec<_>>();

    let remaining_child_size = remaining_child.child.get_size();

    let remaining_child_width = if remaining_child_size.0 < 0.0 {
      constraint.width()
    } else {
      remaining_child_size.0
    };

    let remaining_child_x = match self.align {
      HorizontalAlign::Left => constraint.x(),
      HorizontalAlign::Center => {
        constraint.x() + (constraint.width() - remaining_child_width) * 0.5
      }
      HorizontalAlign::Right => constraint.x() + constraint.width() - remaining_child_width,
    };

    let remaining_stack_child = StackChild {
      position: (remaining_child_x, remaining_child.top_y),
      size: (
        remaining_child_width,
        remaining_child.bottom_y - remaining_child.top_y,
      ),
      child: remaining_child.child,
    };

    stack_children.push(remaining_stack_child);
    stack_children.par_extend(bottom_children);

    Stack {
      children: stack_children,
    }
    .into()
  }
}
