#[cfg(test)]
mod tests {
  use flut::models::Write;

  /// Test helper function to make Write structs more easily
  fn make_write(index: u32, len: u32) -> Write {
    Write { index, len }
  }

  /// Import the coalesce_writes function from the crate
  use flut::coalesce_writes;

  #[test]
  fn test_empty_vector() {
    let mut writes = vec![];
    coalesce_writes(&mut writes);
    assert_eq!(writes.len(), 0);
  }

  #[test]
  fn test_single_write() {
    let mut writes = vec![make_write(5, 10)];
    let expected = vec![make_write(5, 10)];
    coalesce_writes(&mut writes);
    assert_eq!(writes, expected);
  }

  #[test]
  fn test_non_overlapping_writes() {
    let mut writes = vec![make_write(5, 10), make_write(20, 5), make_write(30, 15)];
    let expected = vec![make_write(5, 10), make_write(20, 5), make_write(30, 15)];
    coalesce_writes(&mut writes);
    assert_eq!(writes, expected);
  }

  #[test]
  fn test_adjacent_writes() {
    let mut writes = vec![make_write(5, 10), make_write(15, 5), make_write(20, 10)];
    // Adjacent writes (where one ends and the next begins) should be merged
    // Write 1: 5-15, Write 2: 15-20 -> merged to 5-20
    // Write 3: 20-30 -> merged with previous to 5-30
    let expected = vec![
      make_write(5, 25), // All three merged because they're contiguous
    ];
    coalesce_writes(&mut writes);
    assert_eq!(writes, expected);
  }

  #[test]
  fn test_overlapping_writes() {
    let mut writes = vec![make_write(5, 10), make_write(8, 5), make_write(12, 10)];
    let expected = vec![
      make_write(5, 17), // All three merged
    ];
    coalesce_writes(&mut writes);
    assert_eq!(writes, expected);
  }

  #[test]
  fn test_contained_writes() {
    let mut writes = vec![make_write(5, 20), make_write(10, 5), make_write(15, 3)];
    let expected = vec![
      make_write(5, 20), // First one covers all others
    ];
    coalesce_writes(&mut writes);
    assert_eq!(writes, expected);
  }

  #[test]
  fn test_complex_merge_scenario() {
    let mut writes = vec![
      make_write(1, 5),
      make_write(4, 4),
      make_write(10, 3),
      make_write(12, 5),
      make_write(20, 2),
      make_write(25, 3),
      make_write(26, 2),
    ];
    let expected = vec![
      make_write(1, 7),  // First two merged
      make_write(10, 7), // Middle two merged
      make_write(20, 2), // Stands alone
      make_write(25, 4), // Last two merged
    ];
    coalesce_writes(&mut writes);
    assert_eq!(writes, expected);
  }

  #[test]
  fn test_out_of_order_writes() {
    // These writes are not in order by index, but should be sorted first
    let mut writes = vec![make_write(20, 5), make_write(5, 10), make_write(18, 4)];
    let expected = vec![
      make_write(5, 10),
      make_write(18, 7), // Overlaps with first range after sorting
    ];
    coalesce_writes(&mut writes);
    assert_eq!(writes, expected);
  }

  #[test]
  fn test_zero_length_writes() {
    let mut writes = vec![make_write(5, 0), make_write(5, 10), make_write(15, 0)];
    let expected = vec![
      make_write(5, 10), // Zero-length writes should be absorbed
    ];
    coalesce_writes(&mut writes);
    assert_eq!(writes, expected);
  }
}
