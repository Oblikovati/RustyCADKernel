//! Key callbacks are Rust extensions: native source order is the contract, not a C++ callback oracle.
use super::{PackedInteger, PackedMap, algo, binary, unary};
use std::cell::RefCell;
#[derive(Default)]
struct Trace {
    forbidden_conversion: Option<u64>,
    events: Vec<(bool, u64)>,
}
thread_local! { static TRACE: RefCell<Option<Trace>> = const { RefCell::new(None) }; }
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ObservedKey(u32);
impl PackedInteger for ObservedKey {
    type Word = u32;
    const IS_64_BIT: bool = false;
    const MIN: Self = Self(0);
    const MAX: Self = Self(u32::MAX);
    fn from_bits(value: u64) -> Self {
        observe(false, value);
        Self(value as u32)
    }
    fn bits(self) -> u64 {
        observe(true, u64::from(self.0));
        u64::from(self.0)
    }
}
fn observe(read: bool, value: u64) {
    TRACE.with_borrow_mut(|trace| {
        let Some(trace) = trace else {
            return;
        };
        trace.events.push((read, value));
        assert!(
            read || trace.forbidden_conversion != Some(value),
            "injected conversion {value}: expected a value outside the fault boundary"
        );
    });
}
struct ResetTrace;
impl Drop for ResetTrace {
    fn drop(&mut self) {
        TRACE.with_borrow_mut(|trace| *trace = None);
    }
}
fn recorded(fault: Option<u64>, operation: impl FnOnce()) -> (bool, Vec<(bool, u64)>) {
    TRACE.with_borrow_mut(|trace| {
        *trace = Some(Trace {
            forbidden_conversion: fault,
            events: Vec::new(),
        })
    });
    let _reset: ResetTrace = ResetTrace;
    let failed: bool = std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).is_err();
    let events: Vec<(bool, u64)> = TRACE.with_borrow_mut(|trace| trace.take().unwrap().events);
    (failed, events)
}
fn map(keys: &[u32]) -> PackedMap<ObservedKey> {
    let mut map: PackedMap<ObservedKey> = PackedMap::default();
    for key in keys {
        map.add(ObservedKey(*key));
    }
    map
}
fn ordered_keys(map: &PackedMap<ObservedKey>) -> Vec<u32> {
    map.iter().map(|key| key.0).collect()
}
fn operands(op: usize) -> [PackedMap<ObservedKey>; 3] {
    match op {
        0 => [map(&[97]), map(&[]), map(&[33, 65])],
        2 => [map(&[33, 65]), map(&[]), map(&[33, 97])],
        4 => [map(&[33, 65, 97]), map(&[]), map(&[33, 65])],
        6 => [map(&[33, 97]), map(&[]), map(&[33, 65])],
        3 => [map(&[129]), map(&[33, 65]), map(&[33, 97, 129])],
        5 => [map(&[129]), map(&[33, 65, 97]), map(&[33, 65])],
        _ => [map(&[129]), map(&[33, 65]), map(&[97])],
    }
}
fn conversion_prefix(op: usize, member: bool) {
    let [mut destination, left, right] = operands(op);
    let (failed, _) = recorded(Some(64), || {
        if op.is_multiple_of(2) {
            unary(op, member, &mut destination, &right);
        } else {
            binary(op, member, &mut destination, &left, &right);
        }
    });
    assert!(
        failed,
        "operation {op}: expected next-block conversion panic"
    );
    assert_eq!(
        ordered_keys(&destination),
        expected_prefix(op),
        "operation {op}, member {member}"
    );
}
#[test]
fn all_mutations_apply_current_key_before_next_conversion() {
    for op in 0..8 {
        for member in [false, true] {
            conversion_prefix(op, member);
        }
    }
}
#[test]
fn union_initializes_second_cursor_only_after_first_pass() {
    for member in [false, true] {
        let [mut destination, left, right] = operands(1);
        let (failed, _) = recorded(Some(96), || {
            binary(1, member, &mut destination, &left, &right);
        });
        assert!(failed);
        assert_eq!(ordered_keys(&destination), [33, 65]);
    }
}
fn empty_shortcuts(
    empty: &mut PackedMap<ObservedKey>,
    full: &mut PackedMap<ObservedKey>,
    other: &PackedMap<ObservedKey>,
) {
    assert!(!algo::subtract(empty, other));
    assert!(!algo::subtract(full, empty));
    assert!(!algo::intersect(empty, other));
    assert!(!algo::unite(full, empty));
    assert!(!algo::differ(full, empty));
    algo::intersection(full, other, empty);
    algo::intersection(full, empty, other);
    algo::difference(full, other, other);
}
#[test]
fn empty_and_identity_shortcuts_avoid_key_conversions() {
    let mut empty: PackedMap<ObservedKey> = map(&[]);
    let mut full: PackedMap<ObservedKey> = map(&[33]);
    let other: PackedMap<ObservedKey> = map(&[97]);
    let (failed, events) = recorded(None, || empty_shortcuts(&mut empty, &mut full, &other));
    assert!(!failed);
    // Construct fixtures before observation: their insertion conversions are not predicate work.
    assert!(events.is_empty(), "unexpected callbacks: {events:?}");
}
#[test]
fn differ_checks_contains_before_removing_present_key() {
    let mut actual: PackedMap<ObservedKey> = map(&[33, 97]);
    let other: PackedMap<ObservedKey> = map(&[33]);
    let (failed, calls) = recorded(None, || {
        assert!(algo::differ(&mut actual, &other));
    });
    assert!(!failed);
    assert_eq!(
        calls.iter().filter(|event| **event == (true, 33)).count(),
        present_key_action_reads()
    );
    assert_eq!(ordered_keys(&actual), [97]);
}

fn expected_prefix(op: usize) -> &'static [u32] {
    match op {
        0 => &[33, 97],
        2 => &[33, 65],
        4 | 5 => &[65, 97],
        6 => &[97],
        _ => &[33],
    }
}

fn present_key_action_reads() -> usize {
    let mut reference: PackedMap<ObservedKey> = map(&[33, 97]);
    let (failed, calls) = recorded(None, || {
        assert!(reference.contains(ObservedKey(33)));
        reference.remove(ObservedKey(33));
    });
    assert!(!failed);
    calls.len()
}
