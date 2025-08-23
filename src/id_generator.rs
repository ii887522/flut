pub(super) struct IdGenerator {
  next_id: u16,
  free_ids: Vec<u16>,
}

impl IdGenerator {
  pub(super) const fn new() -> Self {
    Self {
      next_id: 0,
      free_ids: vec![],
    }
  }

  pub(super) fn generate(&mut self) -> u16 {
    if let Some(id) = self.free_ids.pop() {
      id
    } else {
      let id = self.next_id;
      self.next_id += 1;
      id
    }
  }

  pub(super) fn free(&mut self, id: u16) {
    self.free_ids.push(id);
  }
}
