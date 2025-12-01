#[derive(Clone, Copy)]
pub(crate) enum DirectedEdges {
  Neutral,
  Up,
  Right,
  Down,
  Left,
  UpRight,
  RightDown,
  DownLeft,
  LeftUp,
}

impl DirectedEdges {
  pub(crate) fn get_candidates(up: bool, right: bool, down: bool, left: bool) -> Box<[Self]> {
    let mut candidates = Vec::with_capacity(9);

    if up && right {
      candidates.push(DirectedEdges::UpRight);
    }

    if right && down {
      candidates.push(DirectedEdges::RightDown);
    }

    if down && left {
      candidates.push(DirectedEdges::DownLeft);
    }

    if left && up {
      candidates.push(DirectedEdges::LeftUp);
    }

    if !candidates.is_empty() {
      return candidates.into_boxed_slice();
    }

    if up {
      candidates.push(DirectedEdges::Up);
    }

    if right {
      candidates.push(DirectedEdges::Right);
    }

    if down {
      candidates.push(DirectedEdges::Down);
    }

    if left {
      candidates.push(DirectedEdges::Left);
    }

    if candidates.is_empty() {
      candidates.push(DirectedEdges::Neutral);
    }

    candidates.into_boxed_slice()
  }
}
