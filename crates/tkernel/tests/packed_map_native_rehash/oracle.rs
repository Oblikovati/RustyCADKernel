use super::trace::{self, Event, Operation};
use rustycad_tkernel::ncollection::{BaseAllocator, PackedInteger, PackedMap};
use std::collections::BTreeMap;
use std::sync::OnceLock;
fn rows() -> &'static BTreeMap<&'static str, &'static str> {
    static ROWS: OnceLock<BTreeMap<&str, &str>> = OnceLock::new();
    ROWS.get_or_init(|| {
        include_str!("../../../../tools/oracle/tkernel_packed_map_native_rehash.expected")
            .lines()
            .map(|line| line.split_once(' ').unwrap())
            .collect()
    })
}
fn initial<T: PackedInteger>(state: usize) -> PackedMap<T> {
    // Backend singleton setup is outside the operation, as are map population and fixture decoding.
    drop(BaseAllocator::common());
    let mut map = PackedMap::new(if state == 3 { 700 } else { 1 });
    if matches!(state, 1 | 2) {
        for bits in [0, 1, 32, 3232, u64::MAX] {
            map.add(T::from_bits(bits));
        }
    }
    if state == 4 {
        map.resize(0);
    }
    map
}
fn normalized_event(event: &Event) -> String {
    // The canonical raw backend prepends its 16-byte layout header to each logical request.
    assert_eq!(
        event.align, 16,
        "unexpected allocation {event:?}: expected only Standard-policy buckets"
    );
    let payload = event
        .size
        .checked_sub(16)
        .expect("bucket allocation: expected raw layout header");
    let kind = match event.operation {
        Operation::Allocate => 'A',
        Operation::Free => 'F',
        other => panic!("unexpected heap operation {other:?}: expected optimal allocate/free"),
    };
    format!("{kind}:{payload}")
}
fn expected(row: &str) -> (Vec<&str>, Vec<u64>) {
    let mut parts = row.split('|');
    let count: usize = parts.next().unwrap().parse().unwrap();
    // Standard::Free(nullptr) has no underlying heap operation to observe.
    let events = parts
        .by_ref()
        .take(count)
        .filter(|event| *event != "F:0")
        .collect();
    let state = parts.map(|value| value.parse().unwrap()).collect();
    (events, state)
}
pub fn compare<T: PackedInteger>(name: &str) {
    for state in 0..5 {
        let mut map = initial::<T>(state);
        let (expected_events, expected_state) =
            expected(rows()[format!("{name}:rehash:{state}").as_str()]);
        let (_, events) = trace::capture(|| map.resize(if state < 2 { 1000 } else { 0 }));
        let events: Vec<_> = events.iter().map(normalized_event).collect();
        assert_eq!(events, expected_events, "{name}:rehash:{state}");
        let mut observed = vec![map.bucket_count() as u64, map.len() as u64];
        observed.extend(map.iter().map(PackedInteger::bits));
        assert_eq!(observed, expected_state, "{name}:rehash:{state}");
    }
}
