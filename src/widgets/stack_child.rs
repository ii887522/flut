use super::Widget;

#[derive(Debug)]
pub struct StackChild<'a> {
  pub position: (f32, f32),
  pub size: (f32, f32),
  pub child: Option<Widget<'a>>,
}
