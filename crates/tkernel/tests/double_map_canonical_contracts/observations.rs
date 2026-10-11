use super::observation_fixture::{CallbackTrace, heads_empty};
#[test]
fn borrowed_bind_copies_value_then_key() {
    let trace = CallbackTrace::new();
    let mut map = trace.map();
    let (key, value) = (trace.key('K', 1), trace.key('V', 5));
    trace.watch("C");
    map.bind_ref(&key, &value).unwrap();
    super::super::native_order(
        "borrowed_allocation",
        format!("borrowed_allocation {}", trace.events()),
    );
    trace.stop();
}
#[test]
fn removal_hashes_both_keys_then_drops_key1_then_key2() {
    let trace = CallbackTrace::new();
    let mut map = trace.map();
    let (key, value) = (trace.key('K', 1), trace.key('V', 5));
    map.bind_ref(&key, &value).unwrap();
    trace.watch("HD");
    assert!(map.unbind1(&key));
    super::super::native_order(
        "remove_observer",
        format!("remove_observer {}", trace.events().trim_end_matches(',')),
    );
    // Native `after=0:0:0`: no pair and empty heads once UnBind1 returns.
    assert!(map.is_empty() && heads_empty(&map));
    trace.stop();
}
#[test]
fn clear_drops_key1_then_key2_and_keeps_an_empty_table() {
    let trace = CallbackTrace::new();
    let mut map = trace.map();
    map.bind(trace.key('K', 1), trace.key('V', 5)).unwrap();
    trace.watch("HD");
    map.clear(false);
    super::super::native_order(
        "clear_observer",
        format!("clear_observer {}", trace.events().trim_end_matches(',')),
    );
    assert!(map.is_empty() && heads_empty(&map));
    assert_retained_tables(&map);
    trace.stop();
}
fn assert_retained_tables(map: &super::observation_fixture::ObservedMap) {
    // The native row's middle field reports the protected second table emptied (plan D1 B6);
    // the public first table stands in for it.
    super::super::native_row(
        "retained_tables",
        format!(
            "retained_tables {} {} {}",
            u8::from(map.buckets().is_some()),
            u8::from(heads_empty(map)),
            map.bucket_count()
        ),
    );
}
#[test]
fn assign_copies_locals_then_hashes_then_copies_value_then_key() {
    let trace = CallbackTrace::new();
    let mut source = trace.map();
    source.bind(trace.key('K', 1), trace.key('V', 5)).unwrap();
    let mut target = trace.map();
    trace.watch("CHD");
    target.assign(&source);
    super::super::native_order("assign_order", format!("assign_order {}", trace.events()));
    trace.stop();
}
#[test]
fn mixed_borrowed_owned_overloads_copy_only_the_borrowed_key() {
    let trace = CallbackTrace::new();
    let mut map = trace.map();
    let key = trace.key('K', 1);
    let value = trace.key('V', 5);
    let moved_value = trace.key('V', 5);
    trace.watch("C");
    map.bind_key1_ref(&key, moved_value).unwrap();
    super::super::native_order("mixed_first", format!("mixed_first {}", trace.events()));
    map.clear(false);
    let moved_key = trace.key('K', 1);
    trace.watch("C");
    map.bind_key2_ref(moved_key, &value).unwrap();
    super::super::native_order("mixed_second", format!("mixed_second {}", trace.events()));
    trace.stop();
}
#[test]
fn mixed_try_bind_overloads_skip_borrowed_clones_on_duplicate_keys() {
    let trace = CallbackTrace::new();
    let mut map = trace.map();
    let key = trace.key('K', 1);
    let value = trace.key('V', 5);
    assert!(map.try_bind_key1_ref(&key, trace.key('V', 5)));
    let rejected = trace.key('V', 9);
    trace.watch("C");
    assert!(!map.try_bind_key1_ref(&key, rejected));
    assert_eq!(trace.events(), "");
    map.clear(false);
    let accepted = trace.key('K', 1);
    trace.watch("C");
    assert!(map.try_bind_key2_ref(accepted, &value));
    assert_eq!(trace.events(), "CV,");
    let duplicate = trace.key('K', 9);
    trace.watch("C");
    assert!(!map.try_bind_key2_ref(duplicate, &value));
    assert_eq!(trace.events(), "");
    assert!(map.bind_key1_ref(&key, trace.key('V', 9)).is_err());
    assert!(map.bind_key2_ref(trace.key('K', 9), &value).is_err());
    trace.stop();
}
