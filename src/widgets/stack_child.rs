use super::Widget;

pub struct StackChild<'a> {
  pub position: (f32, f32),
  pub size: (f32, f32),
  pub child: Widget<'a>,
}
