use super::observation_fixture::{CallbackTrace, ObservedMap, caught, heads_empty, state};
fn populated(trace: &CallbackTrace) -> ObservedMap {
    let mut map = trace.map();
    for value in 1..=3 {
        map.bind(trace.key('K', value), trace.key('V', value * 10))
            .unwrap();
    }
    map
}
#[test]
fn opposite_hash_panic_in_unbind1_leaves_both_chains_intact() {
    let trace = CallbackTrace::new();
    let mut map = populated(&trace);
    let before = state(&map);
    let key = trace.key('K', 1);
    trace.0.reject_hash.set('V');
    assert!(caught(|| {
        map.unbind1(&key);
    }));
    trace.0.reject_hash.set('\0');
    assert_eq!(state(&map), before);
    assert_eq!(map.find1(&key).unwrap().value, 10);
}
#[test]
fn opposite_hash_panic_in_unbind2_leaves_both_chains_intact() {
    let trace = CallbackTrace::new();
    let mut map = populated(&trace);
    let before = state(&map);
    let key = trace.key('V', 10);
    trace.0.reject_hash.set('K');
    assert!(caught(|| {
        map.unbind2(&key);
    }));
    trace.0.reject_hash.set('\0');
    assert_eq!(state(&map), before);
    assert_eq!(map.find2(&key).unwrap().value, 1);
}
#[test]
fn changed_opposite_hasher_still_unlinks_the_recorded_second_chain() {
    // Native UnBind1 would search the rehashed second bucket, miss the node and leave it
    // linked there after freeing it (undefined behaviour); the slot records its bucket instead.
    let trace = CallbackTrace::new();
    let mut map = populated(&trace);
    trace.0.hash_bias.set(1);
    assert!(map.unbind1(&trace.key('K', 1)));
    trace.0.hash_bias.set(0);
    assert_eq!(map.len(), 2);
    assert!(map.seek2(&trace.key('V', 10)).is_none());
    assert_eq!(map.find2(&trace.key('V', 20)).unwrap().value, 2);
    assert_eq!(map.find2(&trace.key('V', 30)).unwrap().value, 3);
}
#[test]
fn resize_hash_panic_preserves_both_chains_and_the_table() {
    let trace = CallbackTrace::new();
    let mut map = populated(&trace);
    let before = state(&map);
    trace.0.reject_hash.set('V');
    assert!(caught(|| map.resize(200)));
    trace.0.reject_hash.set('\0');
    assert_eq!(state(&map), before);
    map.resize(200);
    assert_eq!(map.find1(&trace.key('K', 1)).unwrap().value, 10);
}
#[test]
fn equality_panic_preserves_existing_bindings() {
    let trace = CallbackTrace::new();
    let mut map = populated(&trace);
    let before = state(&map);
    trace.0.reject_equal.set(true);
    assert!(caught(|| {
        map.try_bind(trace.key('K', 4), trace.key('V', 40));
    }));
    trace.0.reject_equal.set(false);
    assert_eq!(state(&map), before);
}
fn copy_failure(kind: char) {
    let trace = CallbackTrace::new();
    let mut map = trace.map();
    trace.0.reject_clone.set(kind);
    let (key, value) = (trace.key('K', 1), trace.key('V', 10));
    assert!(caught(|| {
        map.bind_ref(&key, &value).unwrap();
    }));
    trace.0.reject_clone.set('\0');
    assert_eq!(map.len(), 0);
    assert!(heads_empty(&map));
    assert!(map.seek2(&value).is_none());
}
#[test]
fn value_copy_failure_publishes_nothing() {
    copy_failure('V');
}
#[test]
fn key_copy_failure_drops_the_copied_value_and_publishes_nothing() {
    copy_failure('K');
}
