use crate::models::range::Range;
use voracious_radix_sort::RadixSort as _;

pub fn coalesce_ranges(ranges: &mut Vec<Range>) {
  ranges.voracious_sort();

  let mut last_coalesced_index = 0;
  let mut index = 1;

  while index < ranges.len() {
    let range = ranges[index];
    let last_coalesced_range = &mut ranges[last_coalesced_index];

    if range.start <= last_coalesced_range.end {
      last_coalesced_range.end = last_coalesced_range.end.max(range.end);
    } else {
      last_coalesced_index += 1;
      ranges[last_coalesced_index] = range;
    }

    index += 1;
  }

  ranges.truncate(last_coalesced_index + 1);
}
