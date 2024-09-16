use super::Widget;
use crate::models::Origin;

#[derive(Clone, Debug)]
pub struct StackChild<'a> {
  pub position: (f32, f32),
  pub size: (f32, f32),
  pub origin: Origin,
  pub child: Option<Widget<'a>>,
}

impl StackChild<'_> {
  pub(crate) fn get_position(&self) -> (f32, f32) {
    let size = self.get_size();

    match self.origin {
      Origin::TopLeft => self.position,
      Origin::Top => (self.position.0 - size.0 * 0.5, self.position.1),
      Origin::TopRight => (self.position.0 - size.0, self.position.1),
      Origin::Left => (self.position.0, self.position.1 - size.1 * 0.5),
      Origin::Center => (
        self.position.0 - size.0 * 0.5,
        self.position.1 - size.1 * 0.5,
      ),
      Origin::Right => (self.position.0 - size.0, self.position.1 - size.1 * 0.5),
      Origin::BottomLeft => (self.position.0, self.position.1 - size.1),
      Origin::Bottom => (self.position.0 - size.0 * 0.5, self.position.1 - size.1),
      Origin::BottomRight => (self.position.0 - size.0, self.position.1 - size.1),
    }
  }

  pub(crate) fn get_size(&self) -> (f32, f32) {
    if let Some(child) = &self.child {
      let child_size = child.get_size();

      (
        if child_size.0 >= 0.0 {
          child_size.0
        } else {
          self.size.0
        },
        if child_size.1 >= 0.0 {
          child_size.1
        } else {
          self.size.1
        },
      )
    } else {
      self.size
    }
  }
}
