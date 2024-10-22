use super::{stateful_widget::State, StatelessWidget};
use atomic_refcell::AtomicRefCell;
use std::sync::Arc;

pub(crate) enum Buildable<'a> {
  Stateless(Arc<AtomicRefCell<dyn StatelessWidget<'a> + 'a>>),
  Stateful(Box<dyn State<'a> + 'a>),
}
