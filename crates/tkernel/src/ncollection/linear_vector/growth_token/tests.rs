//! Native observations precede explicit replacement cleanup; no safe T reads cross holes.
#![allow(unsafe_code)]
use super::{LinearVector, LinearVectorGrowth};
use std::panic::{AssertUnwindSafe, catch_unwind};
#[path = "../../../../tests/support/linear_growth_destruction_fixture.rs"]
mod fixture;
use fixture::*;
fn assert_native(row: &str) {
    let expected =
        include_str!("../../../../../../tools/oracle/tkernel_linear_growth_destruction.expected");
    assert!(
        expected.lines().any(|native| native == row),
        "missing native destructor row {row:?}"
    );
}
fn require_interrupted<T>(token: LinearVectorGrowth<T>) -> LinearVectorGrowth<T> {
    match token.try_into_vector() {
        Ok(_) => panic!("growth extraction succeeded: expected interrupted ownership"),
        Err(token) => token,
    }
}
fn run(spec: &DestructionSpec) {
    let trace = DestructionTrace::default();
    let mut token = filled(spec, &trace);
    // SAFETY: fixture owns all slots/payloads exclusively; no references survive relocation.
    let failure = catch_unwind(AssertUnwindSafe(|| unsafe {
        token.reserve(spec.minimum, |item| item.relocate(spec.moving))
    }));
    assert_eq!(failure.is_err(), spec.fail.is_some());
    if failure.is_ok() {
        trace.dead_end.set(0);
    }
    assert_native(&row("native", spec, &token, &trace, failure.is_ok()));
    if let Some(index) = spec.fail {
        assert_eq!(failure.err().unwrap().downcast_ref::<usize>(), Some(&index));
        token = require_interrupted(token);
        assert_eq!(token.progress().dead_slots, 0..index + 1);
        let events = trace.events.borrow().clone();
        // SAFETY: entry is rejected before touching raw slots; observer holds no slot references.
        assert!(
            catch_unwind(AssertUnwindSafe(|| unsafe {
                token.reserve(0, |_| panic!("dead growth: expected no callback"))
            }))
            .is_err()
        );
        assert_eq!(*trace.events.borrow(), events);
        let replacement = token.take_abandoned_for_test().unwrap();
        let values: String = replacement.iter().map(DestructionItem::encoded).collect();
        assert_native(&format!(
            "replacement:{}:{}:{}:{values}:{}",
            spec.name,
            replacement.capacity(),
            replacement.len(),
            trace.resources.get()
        ));
        trace.fail_drop.set(None);
        let repair = catch_unwind(AssertUnwindSafe(|| {
            token.repair_with(|i| repaired_item(i, &trace))
        }));
        if let Some(value) = spec.repair_failure {
            assert_eq!(repair.unwrap_err().downcast_ref::<i32>(), Some(&value));
            token = require_interrupted(token);
            assert_native(&row("repair-failure", spec, &token, &trace, false));
            trace.fail_repair.set(None);
            token.repair_with(|i| repaired_item(i, &trace));
        } else {
            assert!(repair.is_ok());
        }
        assert_native(&row("restored", spec, &token, &trace, false));
        drop(replacement);
        assert_native(&row("reclaimed", spec, &token, &trace, false));
    }
    trace.active.set(false);
    drop(token);
    assert_eq!(trace.resources.get(), 0);
}
#[test]
fn destructor_and_every_repair_boundary_match_native() {
    for spec in DESTRUCTION_CASES {
        run(spec);
    }
}
mod safety;
