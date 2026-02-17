use flut::collections::sparse_set::SparseSet;

// ============================================================
// Constructor tests
// ============================================================

#[test]
fn test_new() {
  let set: SparseSet<i32> = SparseSet::new();
  assert!(set.is_empty());
  assert_eq!(set.len(), 0);
  assert!(set.get_items().is_empty());
}

#[test]
fn test_default() {
  let set: SparseSet<i32> = SparseSet::default();
  assert!(set.is_empty());
  assert_eq!(set.len(), 0);
}

#[test]
fn test_with_capacity() {
  let set: SparseSet<i32> = SparseSet::with_capacity(10);
  assert!(set.is_empty());
  assert_eq!(set.len(), 0);
}

// ============================================================
// is_empty / len / get_items
// ============================================================

#[test]
fn test_is_empty_after_add() {
  let mut set = SparseSet::new();
  set.add(1);
  assert!(!set.is_empty());
}

#[test]
fn test_is_empty_after_add_and_remove_all() {
  let mut set = SparseSet::new();
  let id = set.add(1).id;
  set.remove(id);
  assert!(set.is_empty());
}

#[test]
fn test_len_after_multiple_adds() {
  let mut set = SparseSet::new();
  set.add(10);
  set.add(20);
  set.add(30);
  assert_eq!(set.len(), 3);
}

#[test]
fn test_len_after_add_and_remove() {
  let mut set = SparseSet::new();
  let id0 = set.add(10).id;
  set.add(20);
  set.remove(id0);
  assert_eq!(set.len(), 1);
}

#[test]
fn test_get_items_reflects_contents() {
  let mut set = SparseSet::new();
  set.add(10);
  set.add(20);
  set.add(30);
  let items = set.get_items();
  assert_eq!(items.len(), 3);
  assert!(items.contains(&10));
  assert!(items.contains(&20));
  assert!(items.contains(&30));
}

#[test]
fn test_get_items_after_update() {
  let mut set = SparseSet::new();
  let id = set.add(10).id;
  set.update(id, 99);
  assert_eq!(set.get_items(), &[99]);
}

// ============================================================
// add
// ============================================================

#[test]
fn test_add_single() {
  let mut set = SparseSet::new();
  let resp = set.add(42);
  assert_eq!(resp.id, 0);
  assert_eq!(set.len(), 1);
  assert_eq!(set.get_items(), &[42]);
}

#[test]
fn test_add_multiple_sequential_ids() {
  let mut set = SparseSet::new();
  assert_eq!(set.add("a").id, 0);
  assert_eq!(set.add("b").id, 1);
  assert_eq!(set.add("c").id, 2);
}

#[test]
fn test_add_reuses_freed_id() {
  let mut set = SparseSet::new();
  let id0 = set.add("a").id;
  let _id1 = set.add("b").id;
  set.remove(id0);
  // Free list has id0; next add should reuse it
  let id2 = set.add("c").id;
  assert_eq!(id2, id0);
}

#[test]
fn test_add_reuses_lifo_order() {
  let mut set = SparseSet::new();
  let id0 = set.add(0).id;
  let id1 = set.add(1).id;
  let _id2 = set.add(2).id;

  set.remove(id0); // free: [0]
  set.remove(id1); // free: [0, 1]

  // LIFO: should get id1 first, then id0
  assert_eq!(set.add(10).id, id1);
  assert_eq!(set.add(20).id, id0);
}

// ============================================================
// update
// ============================================================

#[test]
fn test_update_returns_correct_index() {
  let mut set = SparseSet::new();
  let id0 = set.add(10).id;
  let id1 = set.add(20).id;
  assert_eq!(set.update(id0, 100).index, 0);
  assert_eq!(set.update(id1, 200).index, 1);
}

#[test]
fn test_update_modifies_item() {
  let mut set = SparseSet::new();
  let id = set.add(10).id;
  set.update(id, 99);
  assert_eq!(set.remove(id).item, 99);
}

#[test]
fn test_update_after_swap_remove() {
  let mut set = SparseSet::new();
  let id0 = set.add(0).id;
  let _id1 = set.add(1).id;
  let id2 = set.add(2).id;

  // Removing id0 (index 0): last element (id2) is swapped into index 0
  set.remove(id0);

  // id2 should now be at index 0
  let resp = set.update(id2, 22);
  assert_eq!(resp.index, 0);
  assert_eq!(set.get_items()[0], 22);
}

// ============================================================
// update – negative
// ============================================================

#[test]
#[should_panic]
fn test_update_panics_on_empty_set() {
  let mut set: SparseSet<i32> = SparseSet::new();
  set.update(0, 10);
}

#[test]
#[should_panic]
fn test_update_panics_on_out_of_bounds_id() {
  let mut set = SparseSet::new();
  set.add(1);
  set.update(5, 10); // id 5 was never allocated
}

// ============================================================
// remove
// ============================================================

#[test]
fn test_remove_returns_item() {
  let mut set = SparseSet::new();
  let id = set.add("hello").id;
  let resp = set.remove(id);
  assert_eq!(resp.item, "hello");
}

#[test]
fn test_remove_last_element_index_is_none() {
  let mut set = SparseSet::new();
  let id = set.add(42).id;
  let resp = set.remove(id);
  // Removing the only element: no swap needed, index should be None
  assert_eq!(resp.index, None);
}

#[test]
fn test_remove_middle_element_returns_swap_index() {
  let mut set = SparseSet::new();
  let id0 = set.add(0).id;
  let _id1 = set.add(1).id;
  let _id2 = set.add(2).id;

  // Remove id0 at index 0. Last element (at index 2) is swapped into index 0.
  let resp = set.remove(id0);
  assert_eq!(resp.item, 0);
  assert_eq!(resp.index, Some(0));
}

#[test]
fn test_remove_last_position_element_index_is_none() {
  let mut set = SparseSet::new();
  let _id0 = set.add(0).id;
  let _id1 = set.add(1).id;
  let id2 = set.add(2).id;

  // Remove the last physical element (index 2). swap_remove on last = just pop.
  let resp = set.remove(id2);
  assert_eq!(resp.item, 2);
  assert_eq!(resp.index, None);
}

#[test]
fn test_remove_swap_updates_index_mapping() {
  let mut set = SparseSet::new();
  let id0 = set.add(0).id;
  let id1 = set.add(1).id;
  let id2 = set.add(2).id;

  // Remove id0 at index 0. id2 (was at index 2) is moved to index 0.
  set.remove(id0);

  // Verify id2 is now at index 0
  assert_eq!(set.update(id2, 22).index, 0);
  // id1 is still at index 1
  assert_eq!(set.update(id1, 11).index, 1);
}

#[test]
fn test_remove_all_then_re_add() {
  let mut set = SparseSet::new();
  let id0 = set.add(0).id;
  let id1 = set.add(1).id;
  set.remove(id0);
  set.remove(id1);

  assert!(set.is_empty());
  assert_eq!(set.len(), 0);

  // Re-add should reuse freed IDs
  let new_id = set.add(99).id;
  assert!(new_id == id0 || new_id == id1);
  assert_eq!(set.len(), 1);
}

// ============================================================
// remove – negative
// ============================================================

#[test]
#[should_panic]
fn test_remove_panics_on_empty_set() {
  let mut set: SparseSet<i32> = SparseSet::new();
  set.remove(0);
}

#[test]
#[should_panic]
fn test_remove_panics_on_out_of_bounds_id() {
  let mut set = SparseSet::new();
  set.add(1);
  set.remove(5); // id 5 was never allocated
}

// ============================================================
// bulk_add
// ============================================================

#[test]
fn test_bulk_add_basic() {
  let mut set = SparseSet::new();
  let resp = set.bulk_add(vec![10, 20, 30].into_boxed_slice());
  assert_eq!(resp.ids.len(), 3);
  assert_eq!(resp.ids[0], 0);
  assert_eq!(resp.ids[1], 1);
  assert_eq!(resp.ids[2], 2);
  assert_eq!(set.len(), 3);
}

#[test]
fn test_bulk_add_items_are_stored() {
  let mut set = SparseSet::new();
  set.bulk_add(vec![10, 20, 30].into_boxed_slice());
  let items = set.get_items();
  assert!(items.contains(&10));
  assert!(items.contains(&20));
  assert!(items.contains(&30));
}

#[test]
fn test_bulk_add_with_full_reuse() {
  let mut set = SparseSet::new();
  let id0 = set.add(0).id;
  let id1 = set.add(1).id;
  set.remove(id0);
  set.remove(id1);

  // Both freed IDs should be reused
  let resp = set.bulk_add(vec![10, 20].into_boxed_slice());
  assert_eq!(resp.ids.len(), 2);
  assert!(resp.ids.contains(&id0));
  assert!(resp.ids.contains(&id1));
}

#[test]
fn test_bulk_add_with_partial_reuse() {
  let mut set = SparseSet::new();
  let id0 = set.add(0).id;
  let _id1 = set.add(1).id;
  let _id2 = set.add(2).id;

  set.remove(id0); // free: [0]

  // Bulk add 3 items: 1 reused (id0), 2 new (id 3, 4)
  let resp = set.bulk_add(vec![10, 20, 30].into_boxed_slice());
  assert_eq!(resp.ids.len(), 3);
  assert!(resp.ids.contains(&0)); // reused
  assert!(resp.ids.contains(&3)); // new
  assert!(resp.ids.contains(&4)); // new
}

#[test]
fn test_bulk_add_with_no_reuse() {
  let mut set = SparseSet::new();
  set.add(0);
  set.add(1);

  // No free IDs, all new
  let resp = set.bulk_add(vec![10, 20].into_boxed_slice());
  assert_eq!(resp.ids.len(), 2);
  assert_eq!(resp.ids[0], 2);
  assert_eq!(resp.ids[1], 3);
}

#[test]
fn test_bulk_add_more_free_ids_than_items() {
  let mut set = SparseSet::new();
  let id0 = set.add(0).id;
  let id1 = set.add(1).id;
  let id2 = set.add(2).id;
  set.remove(id0);
  set.remove(id1);
  set.remove(id2);

  // 3 free IDs, adding only 1 item
  let resp = set.bulk_add(vec![99].into_boxed_slice());
  assert_eq!(resp.ids.len(), 1);
  // Should reuse one of the freed IDs
  assert!(resp.ids[0] == id0 || resp.ids[0] == id1 || resp.ids[0] == id2);
}

// ============================================================
// bulk_update
// ============================================================

#[test]
fn test_bulk_update_basic() {
  let mut set = SparseSet::new();
  let ids = set.bulk_add(vec![1, 2, 3].into_boxed_slice()).ids;

  let resp = set.bulk_update(&ids, vec![10, 20, 30].into_boxed_slice());
  assert_eq!(resp.indices.len(), 3);
  assert_eq!(resp.indices[0], 0);
  assert_eq!(resp.indices[1], 1);
  assert_eq!(resp.indices[2], 2);

  let items = set.get_items();
  assert!(items.contains(&10));
  assert!(items.contains(&20));
  assert!(items.contains(&30));
}

#[test]
fn test_bulk_update_partial() {
  let mut set = SparseSet::new();
  let ids = set.bulk_add(vec![1, 2, 3].into_boxed_slice()).ids;

  // Update only the first and last
  let partial_ids = vec![ids[0], ids[2]];
  let resp = set.bulk_update(&partial_ids, vec![10, 30].into_boxed_slice());
  assert_eq!(resp.indices.len(), 2);

  let items = set.get_items();
  assert!(items.contains(&10)); // updated
  assert!(items.contains(&2)); // unchanged
  assert!(items.contains(&30)); // updated
}

#[test]
fn test_bulk_update_single() {
  let mut set = SparseSet::new();
  let id = set.add(1).id;

  let resp = set.bulk_update(&[id], vec![100].into_boxed_slice());
  assert_eq!(resp.indices.len(), 1);
  assert_eq!(resp.indices[0], 0);
  assert_eq!(set.get_items(), &[100]);
}

// ============================================================
// bulk_update – negative
// ============================================================

#[test]
#[should_panic]
fn test_bulk_update_panics_on_invalid_id() {
  let mut set = SparseSet::new();
  set.add(1);
  set.bulk_update(&[5], vec![10].into_boxed_slice());
}

// ============================================================
// bulk_remove
// ============================================================

#[test]
fn test_bulk_remove_all() {
  let mut set = SparseSet::new();
  let ids = set.bulk_add(vec![1, 2, 3].into_boxed_slice()).ids;

  let resp = set.bulk_remove(&ids);
  assert_eq!(resp.items.len(), 3);
  assert_eq!(resp.items[0], 1);
  assert_eq!(resp.items[1], 2);
  assert_eq!(resp.items[2], 3);
  assert!(set.is_empty());
}

#[test]
fn test_bulk_remove_partial() {
  let mut set = SparseSet::new();
  let ids = set.bulk_add(vec![1, 2, 3, 4].into_boxed_slice()).ids;

  // Remove first two
  let to_remove = vec![ids[0], ids[1]];
  let resp = set.bulk_remove(&to_remove);
  assert_eq!(resp.items.len(), 2);
  assert_eq!(resp.items[0], 1);
  assert_eq!(resp.items[1], 2);
  assert_eq!(set.len(), 2);
}

#[test]
fn test_bulk_remove_returns_correct_items() {
  let mut set = SparseSet::new();
  let ids = set.bulk_add(vec![10, 20, 30].into_boxed_slice()).ids;

  // Remove in different order: middle, then first
  let to_remove = vec![ids[1], ids[0]];
  let resp = set.bulk_remove(&to_remove);
  // Items returned correspond to the order of input IDs
  assert_eq!(resp.items[0], 20);
  assert_eq!(resp.items[1], 10);
}

#[test]
fn test_bulk_remove_swap_logic() {
  let mut set = SparseSet::new();
  let ids = set.bulk_add(vec![0, 1, 2, 3, 4].into_boxed_slice()).ids;

  // Remove items at the front; items from end should swap in
  let to_remove = vec![ids[0], ids[1]];
  let resp = set.bulk_remove(&to_remove);

  assert_eq!(resp.items.len(), 2);
  assert_eq!(resp.items[0], 0);
  assert_eq!(resp.items[1], 1);

  // Check remaining items are still accessible
  assert_eq!(set.len(), 3);
  let items = set.get_items();
  assert!(items.contains(&2));
  assert!(items.contains(&3));
  assert!(items.contains(&4));
}

#[test]
fn test_bulk_remove_then_bulk_add_reuses_ids() {
  let mut set = SparseSet::new();
  let ids = set.bulk_add(vec![1, 2, 3].into_boxed_slice()).ids;

  set.bulk_remove(&ids);
  assert!(set.is_empty());

  let new_ids = set.bulk_add(vec![10, 20, 30].into_boxed_slice()).ids;
  // All three old IDs should be reused
  for &new_id in new_ids.iter() {
    assert!(new_id <= 2, "Expected reused ID <= 2, got {new_id}");
  }
}

#[test]
fn test_bulk_remove_single() {
  let mut set = SparseSet::new();
  let ids = set.bulk_add(vec![1, 2, 3].into_boxed_slice()).ids;

  let resp = set.bulk_remove(&[ids[1]]);
  assert_eq!(resp.items.len(), 1);
  assert_eq!(resp.items[0], 2);
  assert_eq!(set.len(), 2);
}

#[test]
fn test_bulk_remove_from_end_no_swap() {
  let mut set = SparseSet::new();
  let ids = set.bulk_add(vec![1, 2, 3].into_boxed_slice()).ids;

  // Remove only the last element; no swap needed since it's already at the end
  let resp = set.bulk_remove(&[ids[2]]);
  assert_eq!(resp.items.len(), 1);
  assert_eq!(resp.items[0], 3);
  // indices (indices_to_replace) should be empty since nothing needs to be swapped
  assert!(resp.indices.is_empty());
  assert_eq!(set.len(), 2);
}

// ============================================================
// Mixed single + bulk operations
// ============================================================

#[test]
fn test_mixed_add_then_bulk_remove() {
  let mut set = SparseSet::new();
  let id0 = set.add(1).id;
  let id1 = set.add(2).id;
  let id2 = set.add(3).id;

  let resp = set.bulk_remove(&[id0, id2]);
  assert_eq!(resp.items.len(), 2);
  assert_eq!(resp.items[0], 1);
  assert_eq!(resp.items[1], 3);

  assert_eq!(set.len(), 1);
  assert_eq!(set.remove(id1).item, 2);
}

#[test]
fn test_mixed_bulk_add_then_single_ops() {
  let mut set = SparseSet::new();
  let ids = set.bulk_add(vec![1, 2, 3, 4].into_boxed_slice()).ids;

  // Single update
  set.update(ids[1], 20);

  // Single remove
  set.remove(ids[0]);

  assert_eq!(set.len(), 3);

  // Verify updated item survives
  let items = set.get_items();
  assert!(items.contains(&20));
  assert!(items.contains(&3));
  assert!(items.contains(&4));
}

#[test]
fn test_mixed_bulk_add_batches() {
  let mut set = SparseSet::new();
  let ids_batch_1 = set.bulk_add(vec![1, 2].into_boxed_slice()).ids;
  let ids_batch_2 = set.bulk_add(vec![3, 4].into_boxed_slice()).ids;

  assert_eq!(set.len(), 4);

  // Remove one from batch 1
  set.remove(ids_batch_1[0]);

  // Bulk remove from batch 2
  let resp = set.bulk_remove(&[ids_batch_2[0], ids_batch_2[1]]);
  assert_eq!(resp.items.len(), 2);
  assert_eq!(resp.items[0], 3);
  assert_eq!(resp.items[1], 4);

  assert_eq!(set.len(), 1);
  assert_eq!(set.remove(ids_batch_1[1]).item, 2);
}

// ============================================================
// sort (requires T: Radixable<f32, Key = f32>)
// ============================================================

#[test]
fn test_sort_already_sorted() {
  let mut set = SparseSet::new();
  set.add(1.0_f32);
  set.add(2.0_f32);
  set.add(3.0_f32);
  let resp = set.sort();
  assert_eq!(set.get_items(), &[1.0_f32, 2.0_f32, 3.0_f32]);
  assert!(resp.indices.is_empty());
}

#[test]
fn test_sort_reverse_order() {
  let mut set = SparseSet::new();
  set.add(3.0_f32);
  set.add(2.0_f32);
  set.add(1.0_f32);
  let resp = set.sort();
  assert_eq!(set.get_items(), &[1.0_f32, 2.0_f32, 3.0_f32]);
  // 3.0 (at 0) -> moves to 2
  // 2.0 (at 1) -> stays at 1
  // 1.0 (at 2) -> moves to 0
  assert_eq!(resp.indices.as_ref(), &[0, 2]);
}

#[test]
fn test_sort_mixed_order() {
  let mut set = SparseSet::new();
  set.add(5.0_f32);
  set.add(1.0_f32);
  set.add(3.0_f32);
  set.add(2.0_f32);
  set.add(4.0_f32);
  let resp = set.sort();
  assert_eq!(
    set.get_items(),
    &[1.0_f32, 2.0_f32, 3.0_f32, 4.0_f32, 5.0_f32]
  );
  // Indices mapping:
  // 1.0: 1 -> 0 (moved)
  // 2.0: 3 -> 1 (moved)
  // 3.0: 2 -> 2 (stayed)
  // 4.0: 4 -> 3 (moved)
  // 5.0: 0 -> 4 (moved)
  assert_eq!(resp.indices.as_ref(), &[0, 1, 3, 4]);
}

#[test]
fn test_sort_single_element() {
  let mut set = SparseSet::new();
  set.add(42.0_f32);
  let resp = set.sort();
  assert_eq!(set.get_items(), &[42.0_f32]);
  assert!(resp.indices.is_empty());
}

#[test]
fn test_sort_empty() {
  let mut set: SparseSet<f32> = SparseSet::new();
  let resp = set.sort();
  assert!(resp.indices.is_empty());
  assert!(set.is_empty());
}

#[test]
fn test_sort_updates_id_mapping() {
  let mut set = SparseSet::new();
  let id0 = set.add(3.0_f32).id;
  let id1 = set.add(1.0_f32).id;
  let id2 = set.add(2.0_f32).id;
  let resp = set.sort();

  // Mapping check:
  // id1: 1 -> 0 result in index 0 in resp.indices
  // id2: 2 -> 1 result in index 1 in resp.indices
  // id0: 0 -> 2 result in index 2 in resp.indices
  assert_eq!(resp.indices.as_ref(), &[0, 1, 2]);

  assert_eq!(set.update(id1, 1.0).index, 0);
  assert_eq!(set.update(id2, 2.0).index, 1);
  assert_eq!(set.update(id0, 3.0).index, 2);
}

#[test]
fn test_sort_preserves_len() {
  let mut set = SparseSet::new();
  set.add(5.0_f32);
  set.add(3.0_f32);
  set.add(1.0_f32);
  let len_before = set.len();
  let _ = set.sort();
  assert_eq!(set.len(), len_before);
}

#[test]
fn test_sort_stable_by_insertion_order() {
  let mut set = SparseSet::new();
  let id0 = set.add(1.0_f32).id;
  let id1 = set.add(1.0_f32).id;
  let id2 = set.add(1.0_f32).id;
  let resp = set.sort();

  assert!(resp.indices.is_empty());
  assert_eq!(set.update(id0, 1.0).index, 0);
  assert_eq!(set.update(id1, 1.0).index, 1);
  assert_eq!(set.update(id2, 1.0).index, 2);
}

#[test]
fn test_sort_then_add() {
  let mut set = SparseSet::new();
  set.add(3.0_f32);
  set.add(1.0_f32);
  let resp = set.sort();
  assert_eq!(resp.indices.as_ref(), &[0, 1]);

  let id = set.add(2.0_f32).id;
  assert_eq!(set.len(), 3);
  assert_eq!(set.update(id, 2.0).index, 2);
}

#[test]
fn test_sort_then_remove() {
  let mut set = SparseSet::new();
  let _id0 = set.add(3.0_f32).id;
  let id1 = set.add(1.0_f32).id;
  let _id2 = set.add(2.0_f32).id;
  let resp = set.sort();
  assert_eq!(resp.indices.as_ref(), &[0, 1, 2]);

  let resp_remove = set.remove(id1);
  assert_eq!(resp_remove.item, 1.0);
  assert_eq!(set.len(), 2);
}

#[test]
fn test_sort_with_negative_values() {
  let mut set = SparseSet::new();
  set.add(1.0_f32);
  set.add(-3.0_f32);
  set.add(0.0_f32);
  set.add(-1.0_f32);
  set.add(2.0_f32);
  let resp = set.sort();
  assert_eq!(
    set.get_items(),
    &[-3.0_f32, -1.0_f32, 0.0_f32, 1.0_f32, 2.0_f32]
  );
  // Indices changes:
  // -3 (at 1) -> 0. (indices += 0)
  // -1 (at 3) -> 1. (indices += 1)
  // 0 (at 2) -> 2. (stayed)
  // 1 (at 0) -> 3. (indices += 3)
  // 2 (at 4) -> 4. (stayed)
  assert_eq!(resp.indices.as_ref(), &[0, 1, 3]);
}

#[test]
fn test_sort_partial_move() {
  let mut set = SparseSet::new();
  set.add(1.0_f32); // at 0
  set.add(3.0_f32); // at 1
  set.add(2.0_f32); // at 2
  let resp = set.sort();
  // Result: [1.0, 2.0, 3.0]
  // 1.0: stayed at 0
  // 2.0: moved 2 -> 1 (index 1 is new)
  // 3.0: moved 1 -> 2 (index 2 is new)
  assert_eq!(resp.indices.as_ref(), &[1, 2]);
}

// ============================================================
// with_capacity – verify it works correctly with operations
// ============================================================

#[test]
fn test_with_capacity_add_within_capacity() {
  let mut set: SparseSet<i32> = SparseSet::with_capacity(10);
  for i in 0..10 {
    assert_eq!(set.add(i).id, i as u32);
  }
  assert_eq!(set.len(), 10);
}

#[test]
fn test_with_capacity_add_beyond_capacity() {
  let mut set: SparseSet<i32> = SparseSet::with_capacity(2);
  for i in 0..5 {
    assert_eq!(set.add(i).id, i as u32);
  }
  assert_eq!(set.len(), 5);
}

// ============================================================
// Complex integration scenarios
// ============================================================

#[test]
fn test_add_remove_add_cycle() {
  let mut set = SparseSet::new();
  let id0 = set.add(0).id;
  set.remove(id0);
  let id1 = set.add(1).id;
  assert_eq!(id1, 0); // reused
  set.remove(id1);
  let id2 = set.add(2).id;
  assert_eq!(id2, 0); // reused again
  assert_eq!(set.get_items(), &[2]);
}

#[test]
fn test_bulk_add_after_many_single_removes() {
  let mut set = SparseSet::new();
  let id0 = set.add(0).id;
  let id1 = set.add(1).id;
  let id2 = set.add(2).id;
  let id3 = set.add(3).id;

  set.remove(id0);
  set.remove(id2);

  // 2 free IDs, bulk_add 4 items: 2 reuse + 2 new
  let resp = set.bulk_add(vec![10, 20, 30, 40].into_boxed_slice());
  assert_eq!(resp.ids.len(), 4);
  assert_eq!(set.len(), 6); // 2 remaining + 4 new

  // id1 and id3 are still valid
  assert_eq!(set.update(id1, 100).index, set.update(id1, 100).index); // just verify no panic
  assert_eq!(set.update(id3, 300).index, set.update(id3, 300).index);
}

#[test]
fn test_bulk_remove_front_items_swap_from_back() {
  let mut set = SparseSet::new();
  let ids = set.bulk_add(vec![0, 1, 2, 3, 4].into_boxed_slice()).ids;

  // Remove front two: items at indices 3, 4 should swap into indices 0, 1
  let resp = set.bulk_remove(&[ids[0], ids[1]]);
  assert_eq!(resp.items[0], 0);
  assert_eq!(resp.items[1], 1);

  // Remaining items should be accessible
  let items = set.get_items();
  assert_eq!(items.len(), 3);
  assert!(items.contains(&2));
  assert!(items.contains(&3));
  assert!(items.contains(&4));
}

#[test]
fn test_interleaved_single_and_bulk_operations() {
  let mut set = SparseSet::new();

  // Single adds
  let id0 = set.add(0).id;
  let id1 = set.add(1).id;

  // Bulk add
  let bulk_ids = set.bulk_add(vec![2, 3, 4].into_boxed_slice()).ids;

  // Single remove
  set.remove(id0);

  // Bulk update
  set.bulk_update(&[id1, bulk_ids[0]], vec![10, 20].into_boxed_slice());

  // Bulk remove
  let resp = set.bulk_remove(&[bulk_ids[1], bulk_ids[2]]);
  assert_eq!(resp.items[0], 3);
  assert_eq!(resp.items[1], 4);

  // Only id1 and bulk_ids[0] should remain
  assert_eq!(set.len(), 2);
  let items = set.get_items();
  assert!(items.contains(&10));
  assert!(items.contains(&20));
}
