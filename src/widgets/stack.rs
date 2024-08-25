use super::StackChild;

#[derive(Debug, Default)]
pub struct Stack<'a> {
  pub children: Vec<StackChild<'a>>,
}
