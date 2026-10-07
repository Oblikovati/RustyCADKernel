use super::{cases::*, fixture::*};
use rustycad_tkernel::ncollection::{BaseAllocator, OrderedDataMap};
use std::panic::{AssertUnwindSafe, catch_unwind};
type Map<'a, V> = OrderedDataMap<BorrowKey<'a>, V, BorrowPolicy>;
fn initial_keys(kind: &str, count: usize) -> Vec<i32> {
    if count == 0 {
        return Vec::new();
    }
    if kind == "tracked" {
        return vec![7];
    }
    if count == 3 {
        return vec![4, 1, 7];
    }
    (0..count as i32).collect()
}
fn populated<'a, V: EmplaceValue<'a>>(trace: &'a BorrowTrace<'a>, count: usize) -> Map<'a, V> {
    let mut map = Map::with_allocator(BorrowPolicy, 0, trace.arena.clone());
    for id in initial_keys(V::KIND, count) {
        map.bind(
            BorrowKey { id, tag: 10, trace },
            V::construct(trace, id * 10),
        );
    }
    map
}
fn invoke<'a, V: EmplaceValue<'a>>(
    map: &mut Map<'a, V>,
    method: Method,
    key: &BorrowKey<'a>,
    trace: &'a BorrowTrace<'a>,
) -> String {
    match catch_unwind(AssertUnwindSafe(|| dispatch(map, method, key, trace))) {
        Ok(result) => result,
        Err(_) => "throw".to_owned(),
    }
}
fn dispatch<'a, V: EmplaceValue<'a>>(
    map: &mut Map<'a, V>,
    method: Method,
    key: &BorrowKey<'a>,
    trace: &'a BorrowTrace<'a>,
) -> String {
    let make = || V::construct(trace, 20);
    match method {
        Method::Emplace => format!("b{}", u8::from(map.emplace_key_ref(key, make))),
        Method::Emplaced => format!("r{}", map.emplaced_key_ref(key, make).id()),
        Method::TryEmplace => format!("b{}", u8::from(map.try_emplace_key_ref(key, make))),
        Method::TryEmplaced => format!("r{}", map.try_emplaced_key_ref(key, make).id()),
    }
}
fn order<'a, V: EmplaceValue<'a>>(map: &Map<'a, V>) -> String {
    let result: String = map
        .items()
        .map(|(key, value)| format!("{}/{}:{},", key.id, key.tag, value.id()))
        .collect();
    if result.is_empty() {
        "-".to_owned()
    } else {
        result
    }
}
fn stats<'a, V: EmplaceValue<'a>>(
    map: &Map<'a, V>,
    trace: &BorrowTrace<'_>,
    same_first: bool,
    same_value: bool,
) -> String {
    let counts = [
        map.len() as i64,
        map.bucket_count() as i64,
        i64::from(!map.buckets().is_none()),
        i64::from(trace.allocations()),
        i64::from(same_first),
        i64::from(same_value),
    ];
    counts.map(|value| value.to_string()).join(" ")
}
fn run_case<'a, V: EmplaceValue<'a>>(trace: &'a BorrowTrace<'a>, method: Method, test: BorrowCase) {
    let mut map = populated::<V>(trace, test.count);
    let key = BorrowKey {
        id: test.key,
        tag: 20,
        trace,
    };
    let before = map.seek(&key).map(std::ptr::from_ref);
    // The first entry is compared by key: slot growth may move it (plan D1 class B5).
    let first = map.first_node().map(|node| node.key().id);
    let source = std::ptr::from_ref(&key);
    test.configure(trace);
    let result = invoke(&mut map, method, &key, trace);
    trace.finish();
    let after = map.seek(&key);
    let same_value = before.is_none() || before == after.map(std::ptr::from_ref);
    let counts = stats(
        &map,
        trace,
        first == map.first_node().map(|node| node.key().id),
        same_value,
    );
    let label = format!("{}_{}_{}", V::KIND, method.name(), test.label);
    record_snapshot(&label, &map, &key, &result, &counts);
    assert_eq!(std::ptr::from_ref(&key), source);
}
fn record_snapshot<'a, V: EmplaceValue<'a>>(
    label: &str,
    map: &Map<'a, V>,
    key: &BorrowKey<'a>,
    result: &str,
    counts: &str,
) {
    let after = map.seek(key);
    let trace = key.trace;
    let row = format!(
        "{result} {counts} {}/{} {} {} {}",
        key.id,
        key.tag,
        after.map_or(-1, EmplaceValue::id),
        trace.events.borrow(),
        order(map)
    );
    native_row(label, &row);
}
fn scalar_case(method: Method, test: BorrowCase) {
    let arena = BaseAllocator::accumulating(16384).unwrap();
    let trace = BorrowTrace::new(&arena);
    run_case::<i32>(&trace, method, test);
}
fn tracked_case(method: Method, test: BorrowCase) {
    let arena = BaseAllocator::accumulating(16384).unwrap();
    let trace = BorrowTrace::new(&arena);
    run_case::<BorrowValue<'_>>(&trace, method, test);
}
fn tracked_cases(method: Method) {
    for test in &CASES[..3] {
        tracked_case(method, *test);
    }
    if method.is_try() {
        for test in &CASES[4..9] {
            tracked_case(method, *test);
        }
    }
}
macro_rules! emplace_suite {
    ($module:ident,$method:expr) => {
        mod $module {
            use super::*;
            #[test]
            fn borrowed_key_copy_occurs_only_after_native_allocation_and_value_constructor() {
                for test in CASES {
                    scalar_case($method, test);
                }
            }
            #[test]
            fn nonclone_value_lifetimes_match_native_key_copy_failure_and_try_hits() {
                tracked_cases($method);
            }
        }
    };
}
emplace_suite!(emplace, Method::Emplace);
emplace_suite!(emplaced, Method::Emplaced);
emplace_suite!(try_emplace, Method::TryEmplace);
emplace_suite!(try_emplaced, Method::TryEmplaced);
