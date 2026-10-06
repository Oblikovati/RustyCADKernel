use super::observation_fixture::{CallbackTrace, caught};
fn populated(trace: &CallbackTrace) -> super::observation_fixture::ObservedMap {
    let mut map = trace.map(trace.0.arena.clone());
    for value in 1..=3 {
        map.bind(trace.key('K', value), trace.key('V', value * 10))
            .unwrap();
    }
    map
}
#[test]
fn first_unlink_rolls_back_if_opposite_hash_panics() {
    let trace = CallbackTrace::new();
    let mut map = populated(&trace);
    let before = map.observer().snapshot();
    let key = trace.key('K', 1);
    trace.watch(&map, 2);
    trace.0.reject_hash.set('V');
    assert!(caught(|| {
        map.unbind1(&key);
    }));
    assert_eq!(map.observer().snapshot(), before);
    trace.0.reject_hash.set('\0');
    assert_eq!(map.find1(&key).unwrap().value, 10);
    trace.0.mode.set(0);
}
#[test]
fn second_unlink_rolls_back_if_opposite_hash_panics() {
    let trace = CallbackTrace::new();
    let mut map = populated(&trace);
    let before = map.observer().snapshot();
    let key = trace.key('V', 10);
    trace.0.reject_hash.set('K');
    assert!(caught(|| {
        map.unbind2(&key);
    }));
    assert_eq!(map.observer().snapshot(), before);
    trace.0.reject_hash.set('\0');
    assert_eq!(map.find2(&key).unwrap().value, 1);
}
#[test]
fn changed_opposite_hasher_cannot_leave_a_dangling_secondary_node() {
    let trace = CallbackTrace::new();
    let mut map = populated(&trace);
    let before = map.observer().snapshot();
    trace.0.hash_bias.set(1);
    assert!(caught(|| {
        map.unbind1(&trace.key('K', 1));
    }));
    assert_eq!(map.observer().snapshot(), before);
    trace.0.hash_bias.set(0);
    assert!(map.unbind1(&trace.key('K', 1)));
    assert_eq!(map.len(), 2);
}
#[test]
fn resize_hash_panic_preserves_both_actual_chains_and_item_addresses() {
    let trace = CallbackTrace::new();
    let mut map = populated(&trace);
    let before = map.observer().snapshot();
    let address = map.seek1_ptr(&trace.key('K', 1));
    trace.0.reject_hash.set('V');
    assert!(caught(|| map.resize(200)));
    assert_eq!(map.observer().snapshot(), before);
    trace.0.reject_hash.set('\0');
    assert_eq!(map.seek1_ptr(&trace.key('K', 1)), address);
}
#[test]
fn equality_panic_preserves_existing_bindings() {
    let trace = CallbackTrace::new();
    let mut map = populated(&trace);
    let before = map.observer().snapshot();
    trace.0.reject_equal.set(true);
    assert!(caught(|| {
        map.try_bind(trace.key('K', 4), trace.key('V', 40));
    }));
    assert_eq!(map.observer().snapshot(), before);
    trace.0.reject_equal.set(false);
}
fn copy_failure(kind: char) {
    let trace = CallbackTrace::new();
    let mut map = trace.map(trace.0.arena.clone());
    trace.0.reject_clone.set(kind);
    let (key, value) = (trace.key('K', 1), trace.key('V', 10));
    assert!(caught(|| {
        map.bind_ref(&key, &value).unwrap();
    }));
    assert_eq!(map.len(), 0);
    assert!(!trace.live());
    let state = map.observer().snapshot();
    assert!(
        state
            .first_buckets
            .iter()
            .chain(&state.second_buckets)
            .all(|p| *p == 0)
    );
}
#[test]
fn value_copy_failure_releases_preallocated_node() {
    copy_failure('V');
}
#[test]
fn key_copy_failure_drops_initialized_value_and_releases_node() {
    copy_failure('K');
}
#[test]
fn unbind_drop_panic_releases_node_then_decrements_count() {
    let trace = CallbackTrace::new();
    let mut map = populated(&trace);
    trace.watch(&map, 2);
    trace.0.reject_drop.set('K');
    assert!(caught(|| {
        map.unbind1(&trace.key('K', 2));
    }));
    assert_eq!(map.len(), 2);
    assert!(trace.events().contains("DK@3:"));
    assert!(trace.events().contains("DV@3:"));
    trace.0.mode.set(0);
    assert!(!map.is_bound1(&trace.key('K', 2)));
    assert!(!map.is_bound2(&trace.key('V', 20)));
}
#[test]
fn clear_drop_panic_drains_all_nodes_with_native_old_count_observation() {
    let trace = CallbackTrace::new();
    let mut map = populated(&trace);
    trace.watch(&map, 2);
    trace.0.reject_drop.set('K');
    assert!(caught(|| map.clear(false)));
    assert_eq!(map.len(), 0);
    assert!(!trace.live());
    assert_eq!(trace.events().matches("@3:").count(), 6);
    let state = map.observer().snapshot();
    assert!(
        state
            .first_buckets
            .iter()
            .chain(&state.second_buckets)
            .all(|p| *p == 0)
    );
    trace.0.mode.set(0);
}
#[test]
fn observer_remains_valid_after_panicking_owner_drop() {
    let trace = CallbackTrace::new();
    let map = populated(&trace);
    let observer = map.observer();
    trace.0.reject_drop.set('V');
    assert!(caught(|| drop(map)));
    let state = observer.snapshot();
    assert_eq!(
        (state.size, state.first_table, state.second_table),
        (0, 0, 0)
    );
    assert!(!trace.live());
}
