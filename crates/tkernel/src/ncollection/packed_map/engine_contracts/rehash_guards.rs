use super::{PackedMap, pointers, snapshot};
use crate::ncollection::packed_map::storage::allocation::observation::capture;
fn rejected_modulus(modulus: usize) {
    let mut map = PackedMap::<i32>::default();
    for key in [0, 1, 32, 3232, -1] {
        map.add(key);
    }
    let before = snapshot(&map);
    let nodes = pointers(&map);
    let (result, events) = capture(false, || {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            map.buckets.rehashed(modulus)
        }))
    });
    assert!(result.is_err());
    assert!(events.is_empty());
    assert_eq!(snapshot(&map), before);
    assert_eq!(pointers(&map), nodes);
}
#[test]
fn zero_modulus_fails_before_allocation_or_mutation() {
    rejected_modulus(0);
}
#[test]
fn overflowing_terminal_slot_fails_before_allocation_or_mutation() {
    rejected_modulus(usize::MAX);
}
