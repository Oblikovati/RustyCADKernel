//! Native growth relocation order, capacity and callback phases on the success path.
use super::super::LinearVector;
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
fn run(spec: &GrowthSpec) {
    let trace = GrowthTrace::default();
    let mut vector = filled(spec, &trace);
    let before = vector.data_ptr();
    // Native SetValue creates its input donor before entering growth.
    let donor = (spec.operation == 'S').then(|| GrowthItem::new(99, &trace));
    vector.reserve_with_relocation(spec.minimum, |item| item.relocate(spec.moving));
    drop(donor);
    assert_native(&row("native", spec, &vector, before, &trace, true));
    trace.active.set(false);
    drop(vector);
    assert_eq!(trace.resources.get(), 0);
}
#[test]
fn growth_allocation_and_source_publication_match_native_matrix() {
    // Rows with a throwing relocation (`fail != 0`) are D1 class B2 and are not reproduced.
    for spec in GROWTH_CASES.iter().filter(|spec| spec.fail == 0) {
        run(spec);
    }
}
