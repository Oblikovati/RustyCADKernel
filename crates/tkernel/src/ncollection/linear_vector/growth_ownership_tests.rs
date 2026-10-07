//! Native growth states precede explicit test-only replacement allocation repair.
#![allow(unsafe_code)]
use super::super::LinearVector;
use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
};
#[path = "../../../tests/support/linear_growth_fixture.rs"]
mod fixture;
use fixture::{GROWTH_CASES, GrowthItem, GrowthSpec, GrowthTrace, filled, row};

fn assert_native(row: &str) {
    let expected =
        include_str!("../../../../../tools/oracle/tkernel_linear_growth_ownership.expected");
    assert!(
        expected.lines().any(|line| line == row),
        "missing native growth row {row:?}"
    );
}
fn repair(spec: &GrowthSpec, trace: &GrowthTrace, allocation: (*mut GrowthItem<'_>, usize)) {
    let (pointer, capacity) = allocation;
    assert_eq!(capacity, (spec.capacity * 2).max(2).max(spec.minimum));
    let count = trace.completed.get();
    // SAFETY: observer captures fresh provenance after source/destination borrows end. Only
    // completed relocation results are initialized; the throwing slot never became live.
    let values: String = unsafe { std::slice::from_raw_parts(pointer, count) }
        .iter()
        .map(GrowthItem::encoded)
        .collect();
    assert_native(&format!(
        "unpublished:{}:{capacity}:{count}:{values}:{}",
        spec.name,
        trace.resources.get()
    ));
    // SAFETY: native state was asserted first. This owner is solely explicit test repair of
    // the leaked allocation and known valid prefix; the production method never does this.
    unsafe { drop(Vec::from_raw_parts(pointer, count, capacity)) };
}
fn run(spec: &GrowthSpec) {
    let trace = GrowthTrace::default();
    let mut vector = filled(spec, &trace);
    let before = vector.data_ptr();
    let abandoned = Cell::new(None);
    let result = catch_unwind(AssertUnwindSafe(|| {
        // Native SetValue creates its input donor before entering growth. Other pipeline
        // callbacks are separately tested through uninstrumented public probes.
        let _donor = (spec.operation == 'S').then(|| GrowthItem::new(99, &trace));
        vector.reserve_relocation_owned(
            spec.minimum,
            |item| item.relocate(spec.moving),
            |replacement| {
                assert_eq!(replacement.len(), 0);
                abandoned.set(Some((replacement.data_mut_ptr(), replacement.capacity())));
            },
        );
    }));
    assert_eq!(result.is_err(), spec.fail != 0);
    assert_native(&row(
        "native",
        spec,
        &vector,
        before,
        &trace,
        result.is_ok(),
    ));
    if let Some(allocation) = abandoned.get() {
        assert_eq!(
            result.as_ref().err().unwrap().downcast_ref::<usize>(),
            Some(&spec.fail)
        );
        repair(spec, &trace, allocation);
        assert_native(&row("repaired", spec, &vector, before, &trace, false));
    }
    trace.active.set(false);
    drop(vector);
    assert_eq!(trace.resources.get(), 0);
}
#[test]
fn growth_allocation_and_source_publication_match_native_matrix() {
    for spec in GROWTH_CASES {
        run(spec);
    }
}
#[path = "growth_ownership_tests/safety.rs"]
mod safety;
