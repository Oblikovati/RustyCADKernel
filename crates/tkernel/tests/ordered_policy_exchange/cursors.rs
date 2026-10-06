use super::{
    adapters::{Map, OrderedExchangeFixture},
    fixture::{SwapPolicy, SwapTrace},
};
use rustycad_tkernel::ncollection::BaseAllocator;
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};
fn maps(trace: &Rc<SwapTrace>) -> (Map, Map) {
    let mut first = Map::construct(SwapPolicy::new(7, trace), 101, BaseAllocator::common());
    let mut second = Map::construct(SwapPolicy::new(19, trace), 101, BaseAllocator::common());
    first.insert(4, trace);
    second.insert(7, trace);
    (first, second)
}
#[test]
fn successful_exchange_keeps_detached_cursor_owner_validation() {
    let trace = Rc::new(SwapTrace::default());
    let (mut first, mut second) = maps(&trace);
    let cursor = first.cursor();
    first.exchange_with(&mut second, SwapPolicy::native_swap);
    assert!(cursor.value(&first).is_err());
    assert!(cursor.value(&second).is_err());
    assert_eq!(second.cursor().value(&second).unwrap().key, 40);
}
#[test]
fn failed_policy_swap_keeps_both_existing_safe_cursors_valid() {
    let trace = Rc::new(SwapTrace::default());
    let (mut first, mut second) = maps(&trace);
    let (first_cursor, second_cursor) = (first.cursor(), second.cursor());
    trace.rejected.set(2);
    assert!(
        catch_unwind(AssertUnwindSafe(
            || first.exchange_with(&mut second, SwapPolicy::native_swap)
        ))
        .is_err()
    );
    assert_eq!(first_cursor.value(&first).unwrap().key, 40);
    assert_eq!(second_cursor.value(&second).unwrap().key, 70);
}
