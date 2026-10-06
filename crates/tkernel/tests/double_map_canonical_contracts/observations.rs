use super::observation_fixture::{CallbackTrace, snapshot};
#[test]
fn borrowed_bind_allocates_actual_node_before_value_then_key_copy() {
    let trace = CallbackTrace::new();
    let mut map = trace.map(trace.0.arena.clone());
    let (key, value) = (trace.key('K', 1), trace.key('V', 5));
    trace.watch(&map, 1);
    map.bind_ref(&key, &value).unwrap();
    crate::native_row(
        "borrowed_allocation",
        format!("borrowed_allocation {}", trace.events()),
    );
    trace.0.mode.set(0);
}
#[test]
fn removal_publishes_first_unlink_before_opposite_hash_and_decrements_after_drop() {
    let trace = CallbackTrace::new();
    let mut map = trace.map(trace.0.arena.clone());
    let (key, value) = (trace.key('K', 1), trace.key('V', 5));
    map.bind_ref(&key, &value).unwrap();
    trace.watch(&map, 2);
    assert!(map.unbind1(&key));
    crate::native_row(
        "remove_observer",
        format!(
            "remove_observer {}after={}",
            trace.events(),
            snapshot(&map.observer().snapshot())
        ),
    );
    trace.0.mode.set(0);
}
#[test]
fn clear_observer_reads_opaque_native_heads_until_bucket_completes() {
    let trace = CallbackTrace::new();
    let mut map = trace.map(trace.0.arena.clone());
    map.bind(trace.key('K', 1), trace.key('V', 5)).unwrap();
    trace.watch(&map, 2);
    map.clear(false);
    crate::native_row(
        "clear_observer",
        format!(
            "clear_observer {}after={}",
            trace.events(),
            snapshot(&map.observer().snapshot())
        ),
    );
    assert_retained_tables(&map);
    trace.0.mode.set(0);
}
fn assert_retained_tables(map: &super::observation_fixture::ObservedMap) {
    let state = map.observer().snapshot();
    crate::native_row(
        "retained_tables",
        format!(
            "retained_tables {} {} {}",
            u8::from(state.first_table != 0),
            u8::from(state.second_buckets.iter().all(|p| *p == 0)),
            state.bucket_count
        ),
    );
}
#[test]
fn assign_copies_locals_then_hashes_then_allocates_and_copies_node_payloads() {
    let trace = CallbackTrace::new();
    let mut source = trace.map(Default::default());
    source.bind(trace.key('K', 1), trace.key('V', 5)).unwrap();
    let mut target = trace.map(trace.0.arena.clone());
    trace.watch(&target, 3);
    target.assign(&source);
    crate::native_row("assign_order", format!("assign_order {}", trace.events()));
    trace.0.mode.set(0);
}
#[test]
fn mixed_borrowed_owned_overloads_copy_only_the_borrowed_key_after_allocation() {
    let trace = CallbackTrace::new();
    let mut map = trace.map(trace.0.arena.clone());
    let key = trace.key('K', 1);
    let value = trace.key('V', 5);
    trace.watch(&map, 1);
    map.bind_key1_ref(&key, trace.key('V', 5)).unwrap();
    crate::native_row("mixed_first", format!("mixed_first {}", trace.events()));
    map.clear(false);
    trace.watch(&map, 1);
    map.bind_key2_ref(trace.key('K', 1), &value).unwrap();
    crate::native_row("mixed_second", format!("mixed_second {}", trace.events()));
    trace.0.mode.set(0);
}
#[test]
fn mixed_try_bind_overloads_skip_borrowed_clones_on_duplicate_keys() {
    let trace = CallbackTrace::new();
    let mut map = trace.map(trace.0.arena.clone());
    let key = trace.key('K', 1);
    let value = trace.key('V', 5);
    assert!(map.try_bind_key1_ref(&key, trace.key('V', 5)));
    trace.watch(&map, 1);
    assert!(!map.try_bind_key1_ref(&key, trace.key('V', 9)));
    assert!(trace.events().is_empty());
    map.clear(false);
    trace.watch(&map, 1);
    assert!(map.try_bind_key2_ref(trace.key('K', 1), &value));
    assert_eq!(trace.events(), "CV@1,");
    trace.watch(&map, 1);
    assert!(!map.try_bind_key2_ref(trace.key('K', 9), &value));
    assert!(trace.events().is_empty());
    assert!(map.bind_key1_ref(&key, trace.key('V', 9)).is_err());
    assert!(map.bind_key2_ref(trace.key('K', 9), &value).is_err());
    trace.0.mode.set(0);
}
