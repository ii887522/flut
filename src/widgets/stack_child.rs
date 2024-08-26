use super::Widget;

#[derive(Debug)]
pub struct StackChild<'a> {
  pub position: (f32, f32),
  pub size: (f32, f32),
  pub child: Option<Widget<'a>>,
}

impl StackChild<'_> {
  pub(super) fn get_size(&self) -> (f32, f32) {
    self.size
  }
}
