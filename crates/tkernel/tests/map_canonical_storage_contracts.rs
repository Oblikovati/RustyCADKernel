//! Actual MapNode<K> facade storage and shared owner regressions.
#[path = "map_canonical_storage_contracts/allocator_policy.rs"]
mod allocator_policy;
#[path = "map_canonical_storage_contracts/fixture.rs"]
mod fixture;
#[path = "map_canonical_storage_contracts/graph.rs"]
mod graph;
#[path = "map_canonical_storage_contracts/panic_cleanup.rs"]
mod panic_cleanup;
#[path = "map_canonical_storage_contracts/traits.rs"]
mod traits;
use fixture::{CollisionHash, TestMap, keys, native_row, populated, table_of};
#[test]
fn native_initial_table_is_unallocated() {
    let map = TestMap::with_hasher(CollisionHash::default(), 5);
    native_row(
        "initial",
        format!(
            "initial {} {} {}",
            map.len(),
            map.bucket_count(),
            u8::from(table_of(&map).is_none())
        ),
    );
}
#[test]
fn native_head_insertion_order_uses_real_chains() {
    let map = populated(5);
    native_row(
        "head_order",
        format!(
            "head_order {} {} {}",
            keys(&map),
            map.len(),
            map.bucket_count()
        ),
    );
}
#[test]
fn native_rehash_reverses_collision_chain_without_moving_values() {
    let mut map = populated(5);
    let pointer = std::ptr::from_ref(map.contained(&2).unwrap());
    map.hasher().calls.borrow_mut().clear();
    map.resize(200);
    let calls = map.hasher().calls.borrow().clone();
    let same = pointer == std::ptr::from_ref(map.contained(&2).unwrap());
    native_row(
        "rehash",
        format!(
            "rehash {} {} {} {}",
            keys(&map),
            calls,
            map.bucket_count(),
            u8::from(same)
        ),
    );
    assert_eq!(map.contained(&2), Some(&2));
}
#[test]
fn native_noop_resize_keeps_table_and_skips_hashing() {
    let mut map = populated(5);
    map.resize(200);
    let table = table_of(&map);
    map.hasher().calls.borrow_mut().clear();
    map.resize(0);
    native_row(
        "resize_noop",
        format!(
            "resize_noop {} {}",
            u8::from(table == table_of(&map)),
            map.hasher().calls.borrow().len()
        ),
    );
}
#[test]
fn native_retained_empty_lookup_and_reinsertion() {
    let mut map = populated(5);
    map.resize(200);
    let table = table_of(&map);
    map.clear(false);
    map.hasher().calls.borrow_mut().clear();
    let found = map.contains(&2);
    native_row(
        "retained_empty",
        format!(
            "retained_empty {} {} {} {} {}",
            map.len(),
            map.bucket_count(),
            u8::from(table == table_of(&map)),
            u8::from(found),
            map.hasher().calls.borrow().len()
        ),
    );
    assert_native_reused_insert(map, table);
}
fn assert_native_reused_insert(mut map: TestMap, table: Option<*const ()>) {
    map.add(7);
    native_row(
        "reused_insert",
        format!(
            "reused_insert {} {} {}",
            keys(&map),
            u8::from(table == table_of(&map)),
            map.hasher().calls.borrow()
        ),
    );
}
#[test]
fn native_copy_preserves_modulus_and_replays_head_insertion() {
    let mut source = populated(1);
    source.resize(200);
    let copy = source.clone();
    native_row(
        "copy ",
        format!(
            "copy {} {} {}",
            keys(&copy),
            copy.bucket_count(),
            copy.len()
        ),
    );
}
#[test]
fn native_assignment_retains_destination_table() {
    let mut source = populated(1);
    source.resize(200);
    let mut target = TestMap::with_hasher(CollisionHash::default(), 2003);
    target.add(9);
    let table = table_of(&target);
    target.assign(&source);
    native_row(
        "assign ",
        format!(
            "assign {} {} {}",
            keys(&target),
            target.bucket_count(),
            u8::from(table == table_of(&target))
        ),
    );
    assert_native_empty_assignment(source, target, table);
}
fn assert_native_empty_assignment(
    mut source: TestMap,
    mut target: TestMap,
    table: Option<*const ()>,
) {
    source.clear(false);
    target.assign(&source);
    native_row(
        "empty_assign",
        format!(
            "empty_assign {} {} {}",
            target.bucket_count(),
            target.len(),
            u8::from(table == table_of(&target))
        ),
    );
}
#[test]
fn native_copy_of_retained_empty_map_has_no_table() {
    let mut source = populated(1);
    source.resize(200);
    source.clear(false);
    let copy = source.clone();
    native_row(
        "empty_copy",
        format!(
            "empty_copy {} {}",
            copy.bucket_count(),
            u8::from(table_of(&copy).is_none())
        ),
    );
}
#[test]
fn native_unbind_handles_middle_head_and_last_node() {
    let mut map = populated(1);
    map.remove(&2);
    native_nonempty_erase("erase_middle", &map);
    map.remove(&3);
    native_nonempty_erase("erase_head", &map);
    map.remove(&1);
    native_row(
        "erase_tail",
        format!(
            "erase_tail {} {}",
            map.len(),
            u8::from(table_of(&map).is_some())
        ),
    );
}

fn native_nonempty_erase(label: &str, map: &TestMap) {
    native_row(label, format!("{label} {} {}", keys(map), map.len()));
}
