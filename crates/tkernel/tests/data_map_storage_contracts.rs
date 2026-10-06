//! Typed DataMap delegates to native bucket/node storage; no parallel authoritative entry vectors.
#![allow(unsafe_code)]
#[path = "data_map_storage_contracts/cloning.rs"]
mod cloning;
#[path = "data_map_storage_contracts/fixture.rs"]
mod fixture;
#[path = "data_map_storage_contracts/panic_cleanup.rs"]
mod panic_cleanup;
#[path = "data_map_storage_contracts/pointers.rs"]
mod pointers;
#[path = "data_map_storage_contracts/traits.rs"]
mod traits;
use fixture::{CollisionHash, TestDataMap, keys, native_row, populated};
#[test]
fn native_initial_table_is_unallocated() {
    let map = TestDataMap::with_hasher(CollisionHash::default(), 5);
    native_row(
        "initial",
        format!(
            "initial {} {} {}",
            map.len(),
            map.bucket_count(),
            u8::from(map.buckets().is_null())
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
    let pointer = std::ptr::from_ref(map.find(&2).unwrap());
    map.hasher().calls.borrow_mut().clear();
    map.resize(200);
    let calls = map.hasher().calls.borrow().clone();
    let same = pointer == std::ptr::from_ref(map.find(&2).unwrap());
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
    assert_eq!(unsafe { *pointer }, 20);
}
#[test]
fn native_noop_resize_keeps_table_and_skips_hashing() {
    let mut map = populated(5);
    map.resize(200);
    let table = map.buckets();
    map.hasher().calls.borrow_mut().clear();
    map.resize(0);
    native_row(
        "resize_noop",
        format!(
            "resize_noop {} {}",
            u8::from(table == map.buckets()),
            map.hasher().calls.borrow().len()
        ),
    );
}
#[test]
fn native_retained_empty_lookup_and_reinsertion() {
    let mut map = populated(5);
    map.resize(200);
    let table = map.buckets();
    map.clear(false);
    map.hasher().calls.borrow_mut().clear();
    let found = map.is_bound(&2);
    native_row(
        "retained_empty",
        format!(
            "retained_empty {} {} {} {} {}",
            map.len(),
            map.bucket_count(),
            u8::from(table == map.buckets()),
            u8::from(found),
            map.hasher().calls.borrow().len()
        ),
    );
    assert_native_reused_insert(map, table);
}
fn assert_native_reused_insert(
    mut map: TestDataMap,
    table: *const *mut rustycad_tkernel::ncollection::MapLinkNode,
) {
    map.bind(7, 70);
    native_row(
        "reused_insert",
        format!(
            "reused_insert {} {} {}",
            keys(&map),
            u8::from(table == map.buckets()),
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
    let mut target = TestDataMap::with_hasher(CollisionHash::default(), 2003);
    target.bind(9, 90);
    let table = target.buckets();
    target.assign(&source);
    native_row(
        "assign ",
        format!(
            "assign {} {} {}",
            keys(&target),
            target.bucket_count(),
            u8::from(table == target.buckets())
        ),
    );
    assert_native_empty_assignment(source, target, table);
}
fn assert_native_empty_assignment(
    mut source: TestDataMap,
    mut target: TestDataMap,
    table: *const *mut rustycad_tkernel::ncollection::MapLinkNode,
) {
    source.clear(false);
    target.assign(&source);
    native_row(
        "empty_assign",
        format!(
            "empty_assign {} {} {}",
            target.bucket_count(),
            target.len(),
            u8::from(table == target.buckets())
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
            u8::from(copy.buckets().is_null())
        ),
    );
}
#[test]
fn native_unbind_handles_middle_head_and_last_node() {
    let mut map = populated(1);
    map.unbind(&2);
    native_nonempty_erase("erase_middle", &map);
    map.unbind(&3);
    native_nonempty_erase("erase_head", &map);
    map.unbind(&1);
    native_row(
        "erase_tail",
        format!(
            "erase_tail {} {}",
            map.len(),
            u8::from(!map.buckets().is_null())
        ),
    );
}

fn native_nonempty_erase(label: &str, map: &TestDataMap) {
    native_row(label, format!("{label} {} {}", keys(map), map.len()));
}
