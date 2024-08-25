use super::{stateful_widget::State, StatelessWidget};

#[derive(Debug)]
pub(crate) enum Buildable<'a> {
  Stateless(Box<dyn StatelessWidget + 'a>),
  Stateful(Box<dyn State + 'a>),
}
