use super::StackChild;

#[derive(Default)]
pub struct Stack<'a> {
  pub children: Vec<StackChild<'a>>,
}
