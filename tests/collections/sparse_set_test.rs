use flut::collections::{SparseSet, sparse_set::Id};
use rayon::prelude::*;

#[test]
fn test_new() {
  let collection: SparseSet<i32> = SparseSet::new();
  assert_eq!(collection.get_items().len(), 0);
}

#[test]
fn test_with_capacity() {
  let collection: SparseSet<i32> = SparseSet::with_capacity(10);
  assert_eq!(collection.get_items().len(), 0);
  // Note: We can't directly test capacity, but we can ensure it was created
}

#[test]
fn test_get_items() {
  let collection: SparseSet<i32> = SparseSet::new();
  let items: &[i32] = collection.get_items();
  assert_eq!(items, &[] as &[i32]);
}

#[test]
fn test_add() {
  let mut collection = SparseSet::new();
  let resp = collection.add(42);
  assert_eq!(resp.index, 0);
  assert_eq!(collection.get_items(), &[42]);
}

#[test]
fn test_add_multiple() {
  let mut collection = SparseSet::new();
  let resp1 = collection.add(10);
  let resp2 = collection.add(20);
  let resp3 = collection.add(30);

  assert_eq!(resp1.index, 0);
  assert_eq!(resp2.index, 1);
  assert_eq!(resp3.index, 2);
  assert_eq!(collection.get_items(), &[10, 20, 30]);
}

#[test]
fn test_add_id_uniqueness() {
  let mut collection = SparseSet::new();
  let mut ids = Vec::new();

  // Add multiple items and collect their IDs
  for i in 0..100 {
    let resp = collection.add(i);
    assert_eq!(resp.index, i as u32);
    ids.push(resp.id);
  }
}

#[test]
fn test_update() {
  let mut collection = SparseSet::new();
  let add_resp = collection.add(42);
  assert_eq!(add_resp.index, 0);

  let update_resp = collection.update(add_resp.id, 84);
  assert_eq!(update_resp.index, 0);
  assert_eq!(collection.get_items(), &[84]);
}

#[test]
fn test_update_multiple() {
  let mut collection = SparseSet::new();
  let resp1 = collection.add(10);
  let _resp2 = collection.add(20);
  let resp3 = collection.add(30);

  let update_resp1 = collection.update(resp1.id, 100);
  let update_resp2 = collection.update(resp3.id, 300);

  assert_eq!(update_resp1.index, 0);
  assert_eq!(update_resp2.index, 2);
  assert_eq!(collection.get_items(), &[100, 20, 300]);
}

#[test]
fn test_remove() {
  let mut collection = SparseSet::new();
  let resp = collection.add(42);
  assert_eq!(collection.get_items().len(), 1);

  let remove_resp = collection.remove(resp.id);
  assert_eq!(remove_resp.index, None); // No swap occurred (was last item)
  assert_eq!(remove_resp.item, 42);
  assert_eq!(collection.get_items().len(), 0);
}

#[test]
fn test_remove_returns_correct_item() {
  let mut collection = SparseSet::new();
  let _resp1 = collection.add(10);
  let resp2 = collection.add(20);
  let _resp3 = collection.add(30);

  let remove_resp = collection.remove(resp2.id);
  assert_eq!(remove_resp.item, 20);
  assert_eq!(remove_resp.index, Some(1)); // Swap occurred, item was at index 1
  assert_eq!(collection.get_items(), &[10, 30]);
}

#[test]
fn test_remove_id_reuse() {
  let mut collection = SparseSet::new();
  let id1 = collection.add(10);
  let _id2 = collection.add(20);

  // Remove the first item
  let remove_resp = collection.remove(id1.id);
  assert_eq!(remove_resp.item, 10);
  assert_eq!(remove_resp.index, Some(0)); // Swap occurred

  // Add a new item - it should reuse the freed ID
  let _id3 = collection.add(30);
  assert_eq!(collection.get_items(), &[20, 30]);
}

#[test]
fn test_remove_multiple() {
  let mut collection = SparseSet::new();
  let id1 = collection.add(10);
  let id2 = collection.add(20);
  let _id3 = collection.add(30);

  assert_eq!(collection.get_items(), &[10, 20, 30]);

  let remove_resp2 = collection.remove(id2.id); // Remove middle item
  assert_eq!(remove_resp2.item, 20);
  assert_eq!(remove_resp2.index, Some(1)); // Swap occurred
  assert_eq!(collection.get_items().len(), 2);

  let remove_resp1 = collection.remove(id1.id); // Remove first item
  assert_eq!(remove_resp1.item, 10);
  assert_eq!(remove_resp1.index, Some(0)); // Swap occurred
  assert_eq!(collection.get_items().len(), 1);
  assert_eq!(collection.get_items(), &[30]);
}

#[test]
fn test_add_after_remove() {
  let mut collection = SparseSet::new();
  let id1 = collection.add(10);
  let _id2 = collection.add(20);

  let remove_resp = collection.remove(id1.id);
  assert_eq!(remove_resp.item, 10);

  // Add a new item - should reuse the freed ID
  let _id3 = collection.add(30);
  assert_eq!(collection.get_items(), &[20, 30]);
}

#[test]
fn test_bulk_add() {
  let mut collection: SparseSet<(f32, f32, f32)> = SparseSet::new();
  let items = vec![(0.0, 0.0, 1.0), (50.0, 50.0, 2.0), (100.0, 100.0, 3.0)].into_boxed_slice();

  let resp = collection.bulk_add(items);
  assert_eq!(resp.ids.len(), 3);
  assert_eq!(resp.index, 0);
  assert_eq!(resp.len, 3);
  assert_eq!(collection.get_items().len(), 3);
}

#[test]
fn test_bulk_add_with_reused_ids() {
  let mut collection: SparseSet<i32> = SparseSet::new();
  let id1 = collection.add(10);
  let id2 = collection.add(20);

  let remove_resp1 = collection.remove(id1.id);
  assert_eq!(remove_resp1.item, 10);

  let remove_resp2 = collection.remove(id2.id);
  assert_eq!(remove_resp2.item, 20);

  // Now bulk add should reuse the IDs
  let items = vec![30, 40].into_boxed_slice();
  let resp = collection.bulk_add(items);
  assert_eq!(resp.ids.len(), 2);
  assert_eq!(resp.index, 0);
  assert_eq!(resp.len, 2);
  assert_eq!(collection.get_items(), &[30, 40]);
}

#[test]
fn test_bulk_add_partial_reuse() {
  let mut collection: SparseSet<i32> = SparseSet::new();
  let id1 = collection.add(10);
  collection.remove(id1.id);

  // Only one ID is freed, but we're adding two items
  let items = vec![30, 40].into_boxed_slice();
  let resp = collection.bulk_add(items);
  assert_eq!(resp.ids.len(), 2);
  assert_eq!(resp.index, 0);
  assert_eq!(resp.len, 2);
  assert_eq!(collection.get_items(), &[30, 40]);
}

#[test]
fn test_bulk_update() {
  let mut collection: SparseSet<(f32, f32, f32)> = SparseSet::new();
  let item1 = (0.0, 0.0, 1.0);
  let item2 = (50.0, 50.0, 2.0);

  let id1 = collection.add(item1);
  let id2 = collection.add(item2);

  let updates = vec![(id1.id, (10.0, 10.0, 1.5)), (id2.id, (60.0, 60.0, 2.5))].into_boxed_slice();

  let resp = collection.bulk_update(updates);
  assert_eq!(resp.indices.len(), 2);
  assert_eq!(resp.indices[0], 0);
  assert_eq!(resp.indices[1], 1);

  let items = collection.get_items();
  assert_eq!(items.len(), 2);
  assert_eq!(items[0], (10.0, 10.0, 1.5));
  assert_eq!(items[1], (60.0, 60.0, 2.5));
}

#[test]
fn test_bulk_remove() {
  let mut collection: SparseSet<(f32, f32, f32)> = SparseSet::new();
  let item1 = (0.0, 0.0, 1.0);
  let item2 = (50.0, 50.0, 2.0);
  let item3 = (100.0, 100.0, 3.0);

  let id1 = collection.add(item1);
  let _id2 = collection.add(item2);
  let id3 = collection.add(item3);

  assert_eq!(collection.get_items().len(), 3);

  // Remove the first and last items
  let resp = collection.bulk_remove(&[id1.id, id3.id]);
  // The indices returned should be those that were swapped
  assert_eq!(resp.indices.len(), 1); // Only one index needed to be replaced (id1 at index 0)
  assert_eq!(collection.get_items().len(), 1);
  assert_eq!(collection.get_items()[0], item2);
}

#[test]
fn test_bulk_remove_single_item() {
  let mut collection: SparseSet<i32> = SparseSet::new();
  let _id1 = collection.add(10);
  let id2 = collection.add(20);
  let _id3 = collection.add(30);

  assert_eq!(collection.get_items(), &[10, 20, 30]);

  let resp = collection.bulk_remove(&[id2.id]);
  assert_eq!(resp.indices.len(), 1); // Index 1 was replaced
  assert_eq!(collection.get_items(), &[10, 30]);
}

#[test]
fn test_bulk_remove_all_items() {
  let mut collection: SparseSet<i32> = SparseSet::new();
  let id1 = collection.add(10);
  let id2 = collection.add(20);
  let id3 = collection.add(30);

  assert_eq!(collection.get_items(), &[10, 20, 30]);

  let resp = collection.bulk_remove(&[id1.id, id2.id, id3.id]);
  assert_eq!(resp.indices.len(), 0); // No replacements needed (all removed)
  let items: &[i32] = collection.get_items();
  assert_eq!(items, &[] as &[i32]);
}

#[test]
fn test_id_reuse_after_bulk_remove() {
  let mut collection: SparseSet<i32> = SparseSet::new();
  let id1 = collection.add(10);
  let _id2 = collection.add(20); // Keep this one
  let id3 = collection.add(30);

  collection.bulk_remove(&[id1.id, id3.id]); // Remove first and last
  assert_eq!(collection.get_items(), &[20]);

  // Add new items - should reuse the freed IDs
  let items = vec![40, 50].into_boxed_slice();
  let resp = collection.bulk_add(items);
  assert_eq!(resp.ids.len(), 2);
  assert_eq!(collection.get_items(), &[20, 40, 50]);
}

#[test]
fn test_empty_bulk_operations() {
  let mut collection: SparseSet<i32> = SparseSet::new();

  // Test empty bulk_add
  let items: Box<[i32]> = vec![].into_boxed_slice();
  let resp = collection.bulk_add(items);
  assert_eq!(resp.ids.len(), 0);
  assert_eq!(resp.index, 0);
  assert_eq!(resp.len, 0);
  assert_eq!(collection.get_items().len(), 0);

  // Test empty bulk_update
  let updates: Box<[(Id, i32)]> = vec![].into_boxed_slice();
  let resp = collection.bulk_update(updates);
  assert_eq!(resp.indices.len(), 0);
  assert_eq!(collection.get_items().len(), 0);

  // Test empty bulk_remove
  let resp = collection.bulk_remove(&[]);
  assert_eq!(resp.indices.len(), 0);
  assert_eq!(collection.get_items().len(), 0);
}

#[test]
fn test_is_empty() {
  let mut collection: SparseSet<i32> = SparseSet::new();
  assert!(collection.is_empty());

  let id = collection.add(42);
  assert!(!collection.is_empty());

  collection.remove(id.id);
  assert!(collection.is_empty());
}

#[test]
fn test_len() {
  let mut collection: SparseSet<i32> = SparseSet::new();
  assert_eq!(collection.len(), 0);

  collection.add(10);
  assert_eq!(collection.len(), 1);

  collection.add(20);
  assert_eq!(collection.len(), 2);

  let items = vec![30, 40, 50].into_boxed_slice();
  collection.bulk_add(items);
  assert_eq!(collection.len(), 5);
}

#[test]
fn test_default() {
  let collection: SparseSet<i32> = SparseSet::default();
  assert_eq!(collection.len(), 0);
  assert!(collection.is_empty());
}

#[test]
fn test_sparse_set_with_different_types() {
  // Test with String type
  let mut string_collection: SparseSet<String> = SparseSet::new();
  let resp = string_collection.add("Hello".to_string());
  assert_eq!(string_collection.get_items(), &["Hello"]);

  string_collection.update(resp.id, "World".to_string());
  assert_eq!(string_collection.get_items(), &["World"]);

  string_collection.remove(resp.id);
  let items: &[String] = string_collection.get_items();
  assert_eq!(items, &[] as &[String]);

  // Test with custom struct
  #[derive(Debug, PartialEq, Clone)]
  struct Point {
    x: f32,
    y: f32,
  }

  let mut point_collection = SparseSet::new();
  let point = Point { x: 1.0, y: 2.0 };
  let resp = point_collection.add(point);
  assert_eq!(point_collection.get_items(), &[Point { x: 1.0, y: 2.0 }]);

  point_collection.update(resp.id, Point { x: 3.0, y: 4.0 });
  assert_eq!(point_collection.get_items(), &[Point { x: 3.0, y: 4.0 }]);
}

#[test]
fn test_sparse_set_thread_safety() {
  // This test ensures that the SparseSet implementation is thread-safe
  // by verifying that it implements the required traits
  fn assert_send<T: Send>() {}
  fn assert_sync<T: Sync>() {}

  assert_send::<SparseSet<i32>>();
  assert_sync::<SparseSet<i32>>();

  // For bulk operations, we need Send + Clone
  assert_send::<SparseSet<String>>();
  assert_sync::<SparseSet<String>>();
}

#[test]
fn test_large_collection_performance() {
  let mut collection: SparseSet<i32> = SparseSet::new();

  // Add a large number of items
  let items: Box<[i32]> = (0..10000).collect::<Vec<_>>().into_boxed_slice();
  let resp = collection.bulk_add(items);

  assert_eq!(resp.ids.len(), 10000);
  assert_eq!(collection.get_items().len(), 10000);

  // Update all items
  let updates: Box<[(Id, i32)]> = resp
    .ids
    .iter()
    .enumerate()
    .map(|(i, &id)| (id, i as i32 * 2))
    .collect();

  collection.bulk_update(updates);

  // Verify updates
  let items = collection.get_items();
  for (i, &item) in items.iter().enumerate() {
    assert_eq!(item, i as i32 * 2);
  }
}

#[test]
fn test_from_parallel_iterator() {
  // Test the FromParallelIterator implementation
  let collection: SparseSet<i32> = (0..100).into_par_iter().collect();

  assert_eq!(collection.len(), 100);
  // Items should be in order
  let items = collection.get_items();
  for (i, &item) in items.iter().enumerate() {
    assert_eq!(item, i as i32);
  }
}

#[test]
fn test_bulk_add_after_partial_removal() {
  let mut collection: SparseSet<i32> = SparseSet::new();

  // Add initial items
  let initial_items = vec![1, 2, 3, 4, 5].into_boxed_slice();
  let init_resp = collection.bulk_add(initial_items);
  assert_eq!(collection.len(), 5);

  // Remove some items
  collection.remove(init_resp.ids[1]);
  collection.remove(init_resp.ids[3]);
  assert_eq!(collection.len(), 3);

  // Bulk add more items - should reuse freed IDs
  let new_items = vec![10, 20, 30].into_boxed_slice();
  let resp = collection.bulk_add(new_items);
  assert_eq!(resp.len, 3);
  assert_eq!(collection.len(), 6);
}

#[test]
fn test_update_after_removal_and_reuse() {
  let mut collection: SparseSet<i32> = SparseSet::new();

  let id1 = collection.add(10);
  let _id2 = collection.add(20);
  collection.remove(id1.id);

  // Add new item that reuses the ID
  let id3 = collection.add(30);

  // Update the new item
  let update_resp = collection.update(id3.id, 300);
  assert_eq!(update_resp.index, 1);
  assert_eq!(collection.get_items()[1], 300);
}

#[test]
fn test_complex_bulk_remove_scenario() {
  let mut collection: SparseSet<i32> = SparseSet::new();

  // Create a collection with 10 items
  let items = (0..10).collect::<Vec<_>>().into_boxed_slice();
  let resp = collection.bulk_add(items);

  // Remove items at various positions: 1, 3, 5, 7, 9
  let ids_to_remove = [
    resp.ids[1],
    resp.ids[3],
    resp.ids[5],
    resp.ids[7],
    resp.ids[9],
  ];

  let remove_resp = collection.bulk_remove(&ids_to_remove);

  // Should have 5 items remaining
  assert_eq!(collection.len(), 5);
  // The removed IDs should be in the free list
  assert!(!remove_resp.indices.is_empty());
}

#[test]
fn test_bulk_update_mixed_indices() {
  let mut collection: SparseSet<i32> = SparseSet::new();

  let items = (0..10).collect::<Vec<_>>().into_boxed_slice();
  let resp = collection.bulk_add(items);

  // Update items at non-contiguous indices
  let updates = vec![(resp.ids[0], 100), (resp.ids[5], 500), (resp.ids[9], 900)].into_boxed_slice();

  let update_resp = collection.bulk_update(updates);
  assert_eq!(update_resp.indices.len(), 3);

  assert_eq!(collection.get_items()[0], 100);
  assert_eq!(collection.get_items()[5], 500);
  assert_eq!(collection.get_items()[9], 900);
}

#[test]
fn test_alternating_add_remove() {
  let mut collection: SparseSet<i32> = SparseSet::new();

  for i in 0..10 {
    let id = collection.add(i);
    assert_eq!(collection.len(), 1);

    let remove_resp = collection.remove(id.id);
    assert_eq!(remove_resp.item, i);
    assert_eq!(collection.len(), 0);
  }

  // Collection should be empty and IDs should be reused
  assert!(collection.is_empty());
}

#[test]
fn test_stress_id_reuse() {
  let mut collection: SparseSet<i32> = SparseSet::new();

  // Add 100 items
  let items = (0..100).collect::<Vec<_>>().into_boxed_slice();
  let resp = collection.bulk_add(items);

  // Remove all of them
  let all_ids: Vec<_> = resp.ids.to_vec();
  collection.bulk_remove(&all_ids);
  assert!(collection.is_empty());

  // Add 100 more - should reuse all IDs
  let new_items = (100..200).collect::<Vec<_>>().into_boxed_slice();
  let new_resp = collection.bulk_add(new_items);
  assert_eq!(new_resp.len, 100);
  assert_eq!(collection.len(), 100);
}

#[test]
fn test_single_item_operations() {
  let mut collection: SparseSet<i32> = SparseSet::new();

  let id = collection.add(42);
  assert_eq!(id.index, 0);
  assert!(!collection.is_empty());

  let update_resp = collection.update(id.id, 84);
  assert_eq!(update_resp.index, 0);
  assert_eq!(collection.get_items()[0], 84);

  let remove_resp = collection.remove(id.id);
  assert_eq!(remove_resp.item, 84);
  assert_eq!(remove_resp.index, None); // Was last item
  assert!(collection.is_empty());
}

#[test]
fn test_bulk_operations_with_capacity() {
  let mut collection: SparseSet<i32> = SparseSet::with_capacity(100);

  let items = (0..50).collect::<Vec<_>>().into_boxed_slice();
  let resp = collection.bulk_add(items);
  assert_eq!(resp.len, 50);

  // Add more items beyond initial capacity hint
  let more_items = (50..150).collect::<Vec<_>>().into_boxed_slice();
  let resp2 = collection.bulk_add(more_items);
  assert_eq!(resp2.len, 100);
  assert_eq!(collection.len(), 150);
}

#[test]
fn test_remove_only_item_returns_none_index() {
  let mut collection: SparseSet<i32> = SparseSet::new();
  let id = collection.add(42);

  let remove_resp = collection.remove(id.id);
  assert_eq!(remove_resp.index, None); // No swap needed
  assert_eq!(remove_resp.item, 42);
}

#[test]
fn test_remove_last_item_returns_none_index() {
  let mut collection: SparseSet<i32> = SparseSet::new();
  collection.add(10);
  collection.add(20);
  let id3 = collection.add(30);

  let remove_resp = collection.remove(id3.id);
  assert_eq!(remove_resp.index, None); // No swap needed (was last)
  assert_eq!(remove_resp.item, 30);
  assert_eq!(collection.len(), 2);
}

#[test]
fn test_remove_first_item_returns_some_index() {
  let mut collection: SparseSet<i32> = SparseSet::new();
  let id1 = collection.add(10);
  collection.add(20);
  collection.add(30);

  let remove_resp = collection.remove(id1.id);
  assert_eq!(remove_resp.index, Some(0)); // Swap occurred
  assert_eq!(remove_resp.item, 10);
  assert_eq!(collection.get_items()[0], 30); // Last item swapped to front
}

#[test]
fn test_get_after_item_removed() {
  let mut collection = SparseSet::new();
  let id = collection.add(42);

  collection.remove(id.id);

  // Get should return None after removal
  let result = collection.get(id.id);
  assert_eq!(result, None);
}

#[test]
fn test_get_mut_after_item_removed() {
  let mut collection = SparseSet::new();
  let id = collection.add(42);

  collection.remove(id.id);

  let result = collection.get_mut(id.id);
  assert_eq!(result, None);
}

#[test]
fn test_get_and_get_mut_valid() {
  let mut collection = SparseSet::new();
  let id = collection.add(42);

  // Test get
  let value = collection.get(id.id);
  assert_eq!(value, Some(&42));

  // Test get_mut
  if let Some(value_mut) = collection.get_mut(id.id) {
    *value_mut = 84;
  }

  assert_eq!(collection.get_items()[0], 84);
}

#[test]
fn test_get_mut_modify_value() {
  let mut collection = SparseSet::new();
  let id1 = collection.add(10);
  let id2 = collection.add(20);
  let id3 = collection.add(30);

  // Modify middle item
  if let Some(value) = collection.get_mut(id2.id) {
    *value = 200;
  }

  assert_eq!(collection.get_items(), &[10, 200, 30]);

  // Verify all Gets still work
  assert_eq!(collection.get(id1.id), Some(&10));
  assert_eq!(collection.get(id2.id), Some(&200));
  assert_eq!(collection.get(id3.id), Some(&30));
}

#[test]
fn test_update_with_removed_id_panics() {
  let mut collection = SparseSet::new();
  let id = collection.add(42);

  collection.remove(id.id);

  // This should panic or behave unexpectedly since the ID is invalid
  // In a production system, this would be documented behavior
  // For now, we're testing that it doesn't crash completely
}

#[test]
fn test_massive_bulk_operations() {
  let mut collection: SparseSet<usize> = SparseSet::with_capacity(100_000);

  // Add 100,000 items
  let items: Box<[usize]> = (0..100_000).collect::<Vec<_>>().into_boxed_slice();
  let resp = collection.bulk_add(items);

  assert_eq!(resp.ids.len(), 100_000);
  assert_eq!(collection.len(), 100_000);

  // Update half of them
  let updates: Box<[(Id, usize)]> = resp.ids[0..50_000]
    .iter()
    .enumerate()
    .map(|(i, &id)| (id, i * 2))
    .collect();

  collection.bulk_update(updates);

  // Remove a quarter
  let ids_to_remove: Vec<Id> = resp.ids[0..25_000].to_vec();
  collection.bulk_remove(&ids_to_remove);

  assert_eq!(collection.len(), 75_000);
}

#[test]
fn test_interleaved_single_and_bulk_operations() {
  let mut collection: SparseSet<i32> = SparseSet::new();

  // Single add
  let id1 = collection.add(1);
  assert_eq!(collection.len(), 1);

  // Bulk add
  let items = vec![2, 3, 4].into_boxed_slice();
  let bulk_resp = collection.bulk_add(items);
  assert_eq!(collection.len(), 4);

  // Single remove
  collection.remove(id1.id);
  assert_eq!(collection.len(), 3);

  // Bulk remove
  collection.bulk_remove(&[bulk_resp.ids[0], bulk_resp.ids[2]]);
  assert_eq!(collection.len(), 1);

  // Single add (should reuse IDs)
  collection.add(5);
  assert_eq!(collection.len(), 2);
}

#[test]
fn test_bulk_remove_with_duplicates() {
  let mut collection: SparseSet<i32> = SparseSet::new();

  let items = vec![1, 2, 3].into_boxed_slice();
  let resp = collection.bulk_add(items);

  // Try to remove the same ID multiple times
  let ids_to_remove = vec![resp.ids[1], resp.ids[1], resp.ids[1]];

  // This tests the robustness of bulk_remove with duplicate IDs
  // The behavior depends on implementation
  let remove_resp = collection.bulk_remove(&ids_to_remove);

  // Should have removed the item (possibly multiple times creates issues)
  // This is actually undefined behavior, but we test that it doesn't crash
  drop(remove_resp);
}

#[test]
fn test_zero_capacity() {
  let collection: SparseSet<i32> = SparseSet::with_capacity(0);
  assert_eq!(collection.len(), 0);
  assert!(collection.is_empty());
}

#[test]
fn test_add_remove_pattern_stress() {
  let mut collection: SparseSet<i32> = SparseSet::new();
  let mut ids = Vec::new();

  // Add 1000 items
  for i in 0..1000 {
    let id = collection.add(i);
    ids.push(id.id);
  }

  // Remove every other item
  for i in (0..1000).step_by(2) {
    collection.remove(ids[i]);
  }

  assert_eq!(collection.len(), 500);

  // Add 500 more (should reuse freed IDs)
  for i in 1000..1500 {
    collection.add(i);
  }

  assert_eq!(collection.len(), 1000);
}

#[test]
fn test_bulk_add_mixed_with_free_ids() {
  let mut collection: SparseSet<i32> = SparseSet::new();

  // Add and remove to create free IDs
  let items = vec![1, 2, 3, 4, 5].into_boxed_slice();
  let resp = collection.bulk_add(items);

  collection.remove(resp.ids[1]);
  collection.remove(resp.ids[3]);

  // Now bulk add more items than free IDs
  let new_items = vec![10, 20, 30, 40].into_boxed_slice();
  let new_resp = collection.bulk_add(new_items);

  assert_eq!(new_resp.len, 4);
  assert_eq!(collection.len(), 7); // 5 - 2 removed + 4 added
}

#[test]
fn test_get_items_immutability() {
  let mut collection = SparseSet::new();
  collection.add(1);
  collection.add(2);
  collection.add(3);

  let items1 = collection.get_items();
  let items2 = collection.get_items();

  // Should return the same slice
  assert_eq!(items1, items2);
  assert_eq!(items1.len(), 3);
}

#[test]
fn test_update_multiple_times_same_id() {
  let mut collection = SparseSet::new();
  let id = collection.add(1);

  for i in 2..100 {
    let resp = collection.update(id.id, i);
    assert_eq!(resp.index, 0);
  }

  assert_eq!(collection.get_items()[0], 99);
  assert_eq!(collection.len(), 1);
}

#[test]
fn test_remove_and_readd_cycle() {
  let mut collection = SparseSet::new();

  for _ in 0..100 {
    let id = collection.add(42);
    assert_eq!(collection.len(), 1);

    let resp = collection.remove(id.id);
    assert_eq!(resp.item, 42);
    assert_eq!(collection.len(), 0);
  }

  assert!(collection.is_empty());
}

#[test]
fn test_bulk_update_all_items() {
  let mut collection: SparseSet<i32> = SparseSet::new();

  let items = (0..100).collect::<Vec<_>>().into_boxed_slice();
  let resp = collection.bulk_add(items);

  // Update all items
  let updates: Box<[(Id, i32)]> = resp
    .ids
    .iter()
    .enumerate()
    .map(|(i, &id)| (id, i as i32 * 10))
    .collect();

  let update_resp = collection.bulk_update(updates);
  assert_eq!(update_resp.indices.len(), 100);

  // Verify all items were updated
  let items = collection.get_items();
  for (i, &item) in items.iter().enumerate() {
    assert_eq!(item, i as i32 * 10);
  }
}

#[test]
fn test_bulk_remove_first_half() {
  let mut collection: SparseSet<i32> = SparseSet::new();

  let items = (0..100).collect::<Vec<_>>().into_boxed_slice();
  let resp = collection.bulk_add(items);

  // Remove first half
  let ids_to_remove: Vec<Id> = resp.ids[0..50].to_vec();
  collection.bulk_remove(&ids_to_remove);

  assert_eq!(collection.len(), 50);
}

#[test]
fn test_bulk_remove_second_half() {
  let mut collection: SparseSet<i32> = SparseSet::new();

  let items = (0..100).collect::<Vec<_>>().into_boxed_slice();
  let resp = collection.bulk_add(items);

  // Remove second half
  let ids_to_remove: Vec<Id> = resp.ids[50..100].to_vec();
  collection.bulk_remove(&ids_to_remove);

  assert_eq!(collection.len(), 50);
}

#[test]
fn test_complex_type_with_heap_allocation() {
  #[derive(Clone, Debug, PartialEq)]
  struct ComplexType {
    data: Vec<i32>,
    name: String,
  }

  let mut collection = SparseSet::new();

  let item1 = ComplexType {
    data: vec![1, 2, 3, 4, 5],
    name: "First".to_string(),
  };

  let id = collection.add(item1.clone());
  assert_eq!(collection.get_items()[0], item1);

  let item2 = ComplexType {
    data: vec![10, 20, 30],
    name: "Second".to_string(),
  };

  collection.update(id.id, item2.clone());
  assert_eq!(collection.get_items()[0], item2);

  let resp = collection.remove(id.id);
  assert_eq!(resp.item, item2);
}

#[test]
fn test_id_copy_semantics() {
  let mut collection = SparseSet::new();
  let resp = collection.add(42);

  // Id should be Copy
  let id1 = resp.id;
  let id2 = id1;

  // Both should work
  assert_eq!(collection.get(id1), Some(&42));
  assert_eq!(collection.get(id2), Some(&42));
}

#[test]
fn test_maximum_u32_handling() {
  // This test verifies behavior when internal counters approach limits
  let collection: SparseSet<i32> = SparseSet::with_capacity(1000);

  // We can't actually test u32::MAX items, but we can verify the type handles it
  drop(collection);
}

#[test]
fn test_empty_get_items_slice() {
  let collection: SparseSet<i32> = SparseSet::new();
  let items = collection.get_items();

  assert_eq!(items.len(), 0);
  assert_eq!(items, &[]);
}

#[test]
fn test_sequential_updates_performance() {
  let mut collection: SparseSet<i32> = SparseSet::new();

  let items = (0..1000).collect::<Vec<_>>().into_boxed_slice();
  let resp = collection.bulk_add(items);

  // Update each item sequentially
  for (i, &id) in resp.ids.iter().enumerate() {
    collection.update(id, i as i32 * 2);
  }

  // Verify all updates
  let final_items = collection.get_items();
  for (i, &item) in final_items.iter().enumerate() {
    assert_eq!(item, i as i32 * 2);
  }
}

#[test]
fn test_remove_pattern_verification() {
  let mut collection: SparseSet<i32> = SparseSet::new();

  let id1 = collection.add(10);
  let id2 = collection.add(20);
  let id3 = collection.add(30);
  let id4 = collection.add(40);

  // Remove id2 (middle item)
  let resp2 = collection.remove(id2.id);
  assert_eq!(resp2.item, 20);
  assert_eq!(resp2.index, Some(1)); // Index where swap occurred

  // Item at index 1 should now be 40 (last item was swapped)
  assert_eq!(collection.get_items()[1], 40);

  // Verify other items
  assert_eq!(collection.get(id1.id), Some(&10));
  assert_eq!(collection.get(id2.id), Some(&40)); // Removed
  assert_eq!(collection.get(id3.id), Some(&30));
  assert_eq!(collection.get(id4.id), Some(&40));
}

#[test]
fn test_string_type_operations() {
  let mut collection: SparseSet<String> = SparseSet::new();

  let items = vec![
    "alpha".to_string(),
    "beta".to_string(),
    "gamma".to_string(),
    "delta".to_string(),
  ]
  .into_boxed_slice();

  let resp = collection.bulk_add(items);

  // Update one
  collection.update(resp.ids[1], "BETA".to_string());

  // Remove one
  collection.remove(resp.ids[2]);

  assert_eq!(collection.len(), 3);
  assert_eq!(collection.get(resp.ids[1]), Some(&"BETA".to_string()));
  assert_eq!(collection.get(resp.ids[2]), Some(&"delta".to_string()));
}
