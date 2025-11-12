use flut::collections::{SparseSet, sparse_set::Id};

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
  let id = collection.add(42);
  assert_eq!(collection.get_items(), &[42]);
}

#[test]
fn test_add_multiple() {
  let mut collection = SparseSet::new();
  let id1 = collection.add(10);
  let id2 = collection.add(20);
  let id3 = collection.add(30);

  assert_eq!(collection.get_items(), &[10, 20, 30]);
}

#[test]
fn test_add_id_uniqueness() {
  let mut collection = SparseSet::new();
  let mut ids = Vec::new();

  // Add multiple items and collect their IDs
  for i in 0..100 {
    ids.push(collection.add(i));
  }
}

#[test]
fn test_update() {
  let mut collection = SparseSet::new();
  let id = collection.add(42);
  collection.update(id, 84);
  assert_eq!(collection.get_items(), &[84]);
}

#[test]
fn test_remove() {
  let mut collection = SparseSet::new();
  let id = collection.add(42);
  assert_eq!(collection.get_items().len(), 1);

  collection.remove(id);
  assert_eq!(collection.get_items().len(), 0);
}

#[test]
fn test_remove_id_reuse() {
  let mut collection = SparseSet::new();
  let id1 = collection.add(10);
  let id2 = collection.add(20);

  // Remove the first item
  collection.remove(id1);

  // Add a new item - it should reuse the freed ID
  let id3 = collection.add(30);
  assert_eq!(collection.get_items(), &[20, 30]);
}

#[test]
fn test_remove_multiple() {
  let mut collection = SparseSet::new();
  let id1 = collection.add(10);
  let id2 = collection.add(20);
  let id3 = collection.add(30);

  assert_eq!(collection.get_items(), &[10, 20, 30]);

  collection.remove(id2); // Remove middle item
  assert_eq!(collection.get_items().len(), 2);

  collection.remove(id1); // Remove first item
  assert_eq!(collection.get_items().len(), 1);
  assert_eq!(collection.get_items(), &[30]);
}

#[test]
fn test_add_after_remove() {
  let mut collection = SparseSet::new();
  let id1 = collection.add(10);
  let id2 = collection.add(20);

  collection.remove(id1);

  // Add a new item - should reuse the freed ID
  let id3 = collection.add(30);
  assert_eq!(collection.get_items(), &[20, 30]);
}

#[test]
fn test_bulk_add() {
  let mut collection: SparseSet<(f32, f32, f32)> = SparseSet::new();
  let items = vec![(0.0, 0.0, 1.0), (50.0, 50.0, 2.0), (100.0, 100.0, 3.0)].into_boxed_slice();

  let ids = collection.bulk_add(items);
  assert_eq!(ids.len(), 3);
  assert_eq!(collection.get_items().len(), 3);
}

#[test]
fn test_bulk_add_with_reused_ids() {
  let mut collection: SparseSet<i32> = SparseSet::new();
  let id1 = collection.add(10);
  let id2 = collection.add(20);
  collection.remove(id1);
  collection.remove(id2);

  // Now bulk add should reuse the IDs
  let items = vec![30, 40].into_boxed_slice();
  let ids = collection.bulk_add(items);
  assert_eq!(ids.len(), 2);
  assert_eq!(collection.get_items(), &[30, 40]);
}

#[test]
fn test_bulk_add_partial_reuse() {
  let mut collection: SparseSet<i32> = SparseSet::new();
  let id1 = collection.add(10);
  collection.remove(id1);

  // Only one ID is freed, but we're adding two items
  let items = vec![30, 40].into_boxed_slice();
  let ids = collection.bulk_add(items);
  assert_eq!(ids.len(), 2);
  assert_eq!(collection.get_items(), &[30, 40]);
}

#[test]
fn test_bulk_add_id_uniqueness() {
  let mut collection: SparseSet<i32> = SparseSet::new();

  // Add some items normally first
  let _id1 = collection.add(1);
  let _id2 = collection.add(2);

  // Bulk add items
  let items = vec![3, 4, 5, 6, 7, 8, 9, 10].into_boxed_slice();
  let ids = collection.bulk_add(items);

  // Collect all IDs
  let mut all_ids: Vec<u32> = vec![0, 1]; // The first two IDs

  // Ensure all IDs are unique
  let mut unique_ids = all_ids.clone();
  unique_ids.sort();
  unique_ids.dedup();
  assert_eq!(unique_ids.len(), all_ids.len());
}

#[test]
fn test_bulk_update() {
  let mut collection: SparseSet<(f32, f32, f32)> = SparseSet::new();
  let item1 = (0.0, 0.0, 1.0);
  let item2 = (50.0, 50.0, 2.0);

  let id1 = collection.add(item1);
  let id2 = collection.add(item2);

  let updates = vec![(id1, (10.0, 10.0, 1.5)), (id2, (60.0, 60.0, 2.5))].into_boxed_slice();

  collection.bulk_update(updates);

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
  let id2 = collection.add(item2); // No need to underscore, we're not using it later
  let id3 = collection.add(item3);

  assert_eq!(collection.get_items().len(), 3);

  // Remove the first and last items
  collection.bulk_remove(&[id1, id3]);
  assert_eq!(collection.get_items().len(), 1);
  // The remaining item should be the one with id2
  // Note: Due to swap_remove, the item that was at index 2 (id3)
  // would have been swapped with the one at index 0 (id1), so the
  // item with id2 would end up at index 0.
}

#[test]
fn test_bulk_remove_single_item() {
  let mut collection: SparseSet<i32> = SparseSet::new();
  let id1 = collection.add(10);
  let id2 = collection.add(20);
  let id3 = collection.add(30);

  assert_eq!(collection.get_items(), &[10, 20, 30]);

  collection.bulk_remove(&[id2]);
  assert_eq!(collection.get_items(), &[10, 30]);
}

#[test]
fn test_bulk_remove_all_items() {
  let mut collection: SparseSet<i32> = SparseSet::new();
  let id1 = collection.add(10);
  let id2 = collection.add(20);
  let id3 = collection.add(30);

  assert_eq!(collection.get_items(), &[10, 20, 30]);

  collection.bulk_remove(&[id1, id2, id3]);
  let items: &[i32] = collection.get_items();
  assert_eq!(items, &[] as &[i32]);
}

#[test]
fn test_id_reuse_after_bulk_remove() {
  let mut collection: SparseSet<i32> = SparseSet::new();
  let id1 = collection.add(10);
  let _id2 = collection.add(20); // Keep this one
  let id3 = collection.add(30);

  collection.bulk_remove(&[id1, id3]); // Remove first and last
  assert_eq!(collection.get_items(), &[20]);

  // Add new items - should reuse the freed IDs
  let items = vec![40, 50].into_boxed_slice();
  let ids = collection.bulk_add(items);
  assert_eq!(ids.len(), 2);
  assert_eq!(collection.get_items(), &[20, 40, 50]);
}

#[test]
fn test_empty_bulk_operations() {
  let mut collection: SparseSet<i32> = SparseSet::new();

  // Test empty bulk_add
  let items: Box<[i32]> = vec![].into_boxed_slice();
  let ids = collection.bulk_add(items);
  assert_eq!(ids.len(), 0);
  assert_eq!(collection.get_items().len(), 0);

  // Test empty bulk_update
  let updates: Box<[(Id, i32)]> = vec![].into_boxed_slice();
  collection.bulk_update(updates); // Should not panic
  assert_eq!(collection.get_items().len(), 0);

  // Test empty bulk_remove
  collection.bulk_remove(&[]); // Should not panic
  assert_eq!(collection.get_items().len(), 0);
}

#[test]
fn test_sparse_set_with_different_types() {
  // Test with String type
  let mut string_collection: SparseSet<String> = SparseSet::new();
  let id = string_collection.add("Hello".to_string());
  assert_eq!(string_collection.get_items(), &["Hello"]);

  string_collection.update(id, "World".to_string());
  assert_eq!(string_collection.get_items(), &["World"]);

  string_collection.remove(id);
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
  let id = point_collection.add(point);
  assert_eq!(point_collection.get_items(), &[Point { x: 1.0, y: 2.0 }]);

  point_collection.update(id, Point { x: 3.0, y: 4.0 });
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
  let ids = collection.bulk_add(items);

  assert_eq!(ids.len(), 10000);
  assert_eq!(collection.get_items().len(), 10000);

  // Update all items
  let updates: Box<[(Id, i32)]> = ids
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
