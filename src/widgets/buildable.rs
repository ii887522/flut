use super::{stateful_widget::State, StatelessWidget};

#[derive(Debug)]
pub(crate) enum Buildable<'a> {
  Stateless(Box<dyn StatelessWidget<'a> + 'a>),
  Stateful(Box<dyn State<'a> + 'a>),
}
