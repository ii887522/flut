use flut::collections::sparse_set::SparseSet;

#[test]
fn test_new() {
  let mut vec: SparseSet<i32> = SparseSet::new();
  // Simply assert it exists and doesn't crash on creation
  assert_eq!(vec.add(1).id, 0);
}

#[test]
fn test_default() {
  let mut vec: SparseSet<i32> = SparseSet::default();
  assert_eq!(vec.add(1).id, 0);
}

#[test]
fn test_with_capacity() {
  let mut vec: SparseSet<i32> = SparseSet::with_capacity(10);
  // Verify it works as expected
  for i in 0..10 {
    assert_eq!(vec.add(i).id, i as u32);
  }
}

#[test]
fn test_add_remove_reuse() {
  let mut vec = SparseSet::new();
  let id0 = vec.add("a").id;
  let id1 = vec.add("b").id;
  let id2 = vec.add("c").id;

  assert_eq!(id0, 0);
  assert_eq!(id1, 1);
  assert_eq!(id2, 2);

  let remove_resp = vec.remove(id1);
  assert_eq!(remove_resp.item, "b");
  assert_eq!(remove_resp.index, Some(1));

  // LIFO reuse: should get 1 back
  let id3 = vec.add("d").id;
  assert_eq!(id3, 1);

  // Next fresh ID should be 3 (since 0, 1, 2 were used, 1 reused)
  let id4 = vec.add("e").id;
  assert_eq!(id4, 3);
}

#[test]
fn test_update() {
  let mut vec = SparseSet::new();
  let id = vec.add(10).id;
  let update_resp = vec.update(id, 20);
  assert_eq!(update_resp.index, 0);
  assert_eq!(vec.remove(id).item, 20);
}

#[test]
#[should_panic]
fn test_update_panic() {
  let mut vec: SparseSet<i32> = SparseSet::new();
  vec.update(0, 10);
}

#[test]
#[should_panic]
fn test_remove_panic() {
  let mut vec: SparseSet<i32> = SparseSet::new();
  vec.remove(0);
}

#[test]
fn test_bulk_ops() {
  let mut vec = SparseSet::new();
  let ids = vec.bulk_add(vec![1, 2, 3].into_boxed_slice()).ids;

  assert_eq!(ids.len(), 3);
  assert_eq!(ids[0], 0);
  assert_eq!(ids[1], 1);
  assert_eq!(ids[2], 2);

  let indices = vec
    .bulk_update(&ids, vec![10, 20, 30].into_boxed_slice())
    .indices;
  assert_eq!(indices.len(), 3);
  assert_eq!(indices[0], 0);
  assert_eq!(indices[1], 1);
  assert_eq!(indices[2], 2);

  let remove_resp = vec.bulk_remove(&ids);
  let removed_items = remove_resp.items;

  assert_eq!(removed_items.len(), 3);
  assert_eq!(removed_items[0], 10);
  assert_eq!(removed_items[1], 20);
  assert_eq!(removed_items[2], 30);
}

#[test]
fn test_bulk_add_with_reuse() {
  let mut vec = SparseSet::new();
  let id0 = vec.add(0).id;
  let id1 = vec.add(1).id;
  let _id2 = vec.add(2).id;

  vec.remove(id1); // free: [1]
  vec.remove(id0); // free: [1, 0] (assuming LIFO/push order)

  // free_ids should have 2 items.
  // bulk_add 3 items. 2 should be reused, 1 new.
  let ids = vec.bulk_add(vec![10, 20, 30].into_boxed_slice()).ids;

  // Just verify that ids contains 0 and 1.
  assert!(ids.contains(&0));
  assert!(ids.contains(&1));
  assert!(ids.contains(&3)); // 2 is still taken. so next is 3.

  assert_eq!(vec.remove(0).item, 20); // Mapping might vary, check logic if needed but value consistency is key
}

#[test]
fn test_remove_swap_logic() {
  let mut vec = SparseSet::new();
  let id0 = vec.add(0).id;
  let id1 = vec.add(1).id;
  let id2 = vec.add(2).id;

  // Removing id0 (index 0).
  // Last element is id2 (index 2).
  // Swap remove: put id2 at index 0.
  // id2's index becomes 0.
  let remove_resp_0 = vec.remove(id0);
  assert_eq!(remove_resp_0.item, 0);
  assert_eq!(remove_resp_0.index, Some(0)); // Was at index 0, and something moved there (or just logically it was at 0)

  // Explicit check for swap: id2 should now be at index 0 technically,
  // but public API hides index often unless returned.
  // update id2 and check index
  let update_resp = vec.update(id2, 22);
  assert_eq!(update_resp.index, 0); // Confirms swap move

  assert_eq!(vec.remove(id2).item, 22);
  assert_eq!(vec.remove(id1).item, 1);
}

#[test]
fn test_mixed_bulk_operations() {
  let mut vec = SparseSet::new();
  let ids_batch_1 = vec.bulk_add(vec![1, 2].into_boxed_slice()).ids;
  let ids_batch_2 = vec.bulk_add(vec![3, 4].into_boxed_slice()).ids;

  // remove one from batch 1
  vec.remove(ids_batch_1[0]);

  // bulk remove mixed
  let to_remove = vec![ids_batch_1[1], ids_batch_2[0]];
  let remove_resp = vec.bulk_remove(&to_remove);
  let removed_items = remove_resp.items;

  assert_eq!(removed_items.len(), 2);
  // order should correspond to input ids
  assert_eq!(removed_items[0], 2);
  assert_eq!(removed_items[1], 3);
}
