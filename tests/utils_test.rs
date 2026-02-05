use flut::models::range::Range;
use flut::utils::coalesce_ranges;

fn assert_range_eq(actual: Range, start: u32, end: u32) {
  assert!(
    actual.start == start && actual.end == end,
    "Expected Range {{ start: {}, end: {} }}, but got Range {{ start: {}, end: {} }}",
    start,
    end,
    actual.start,
    actual.end
  );
}

#[test]
fn test_coalesce_ranges_empty() {
  let mut ranges: Vec<Range> = Vec::new();
  coalesce_ranges(&mut ranges);
  assert!(ranges.is_empty());
}

#[test]
fn test_coalesce_ranges_single() {
  let mut ranges = vec![Range { start: 1, end: 10 }];
  coalesce_ranges(&mut ranges);
  assert_eq!(ranges.len(), 1);
  assert_eq!(ranges[0].start, 1);
  assert_eq!(ranges[0].end, 10);
}

#[test]
fn test_coalesce_ranges_non_overlapping() {
  let mut ranges = vec![
    Range { start: 1, end: 5 },
    Range { start: 10, end: 15 },
    Range { start: 20, end: 25 },
  ];
  coalesce_ranges(&mut ranges);
  assert_eq!(ranges.len(), 3);
  assert_range_eq(ranges[0], 1, 5);
  assert_range_eq(ranges[1], 10, 15);
  assert_range_eq(ranges[2], 20, 25);
}

#[test]
fn test_coalesce_ranges_overlapping() {
  let mut ranges = vec![
    Range { start: 1, end: 10 },
    Range { start: 5, end: 15 },
    Range { start: 12, end: 20 },
  ];
  coalesce_ranges(&mut ranges);
  assert_eq!(ranges.len(), 1);
  assert_range_eq(ranges[0], 1, 20);
}

#[test]
fn test_coalesce_ranges_adjacent() {
  let mut ranges = vec![Range { start: 1, end: 5 }, Range { start: 5, end: 10 }];
  coalesce_ranges(&mut ranges);
  assert_eq!(ranges.len(), 1);
  assert_range_eq(ranges[0], 1, 10);
}

#[test]
fn test_coalesce_ranges_unsorted() {
  let mut ranges = vec![
    Range { start: 20, end: 25 },
    Range { start: 1, end: 10 },
    Range { start: 10, end: 15 },
  ];
  coalesce_ranges(&mut ranges);
  assert_eq!(ranges.len(), 2);
  assert_range_eq(ranges[0], 1, 15);
  assert_range_eq(ranges[1], 20, 25);
}

#[test]
fn test_coalesce_ranges_full_coverage() {
  let mut ranges = vec![
    Range { start: 1, end: 100 },
    Range { start: 10, end: 20 },
    Range { start: 30, end: 40 },
    Range { start: 50, end: 60 },
  ];
  coalesce_ranges(&mut ranges);
  assert_eq!(ranges.len(), 1);
  assert_range_eq(ranges[0], 1, 100);
}

#[test]
fn test_coalesce_ranges_multiple_groups() {
  let mut ranges = vec![
    Range { start: 1, end: 5 },
    Range { start: 2, end: 6 },
    Range { start: 10, end: 15 },
    Range { start: 14, end: 20 },
    Range { start: 30, end: 35 },
  ];
  coalesce_ranges(&mut ranges);
  assert_eq!(ranges.len(), 3);
  assert_range_eq(ranges[0], 1, 6);
  assert_range_eq(ranges[1], 10, 20);
  assert_range_eq(ranges[2], 30, 35);
}

#[test]
fn test_coalesce_ranges_identical() {
  let mut ranges = vec![
    Range { start: 5, end: 10 },
    Range { start: 5, end: 10 },
    Range { start: 5, end: 10 },
  ];
  coalesce_ranges(&mut ranges);
  assert_eq!(ranges.len(), 1);
  assert_range_eq(ranges[0], 5, 10);
}

#[test]
fn test_coalesce_ranges_edge_cases() {
  // Max values
  let mut ranges = vec![
    Range {
      start: u32::MAX - 10,
      end: u32::MAX,
    },
    Range {
      start: u32::MAX - 5,
      end: u32::MAX,
    },
  ];
  coalesce_ranges(&mut ranges);
  assert_eq!(ranges.len(), 1);
  assert_range_eq(ranges[0], u32::MAX - 10, u32::MAX);

  // Zero-length ranges (if allowed by logic)
  let mut ranges = vec![Range { start: 10, end: 10 }, Range { start: 10, end: 15 }];
  coalesce_ranges(&mut ranges);
  assert_eq!(ranges.len(), 1);
  assert_range_eq(ranges[0], 10, 15);
}
