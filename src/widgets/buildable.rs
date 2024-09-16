use super::{stateful_widget::State, StatelessWidget};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub(crate) enum Buildable<'a> {
  Stateless(Arc<Mutex<dyn StatelessWidget<'a> + 'a>>),
  Stateful(Box<dyn State<'a> + 'a>),
}
