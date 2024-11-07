use super::StackChild;

pub struct Stack<'a> {
  pub children: Vec<StackChild<'a>>,
}
