//! Native fill construction: copy count, order and capacity on the success path.
//! Cases with a throwing copy (`fail != 0`) are D1 class B2 and are not reproduced.
use super::super::LinearVector;
#[path = "../../../../tests/support/linear_fill_constructor_fixture.rs"]
mod fixture;
use super::super::constructor_test_support::item;
use fixture::{CopyItem, CopyTrace, FILL_CASES, FillSpec, row};

fn assert_native(row: &str) {
    let expected =
        include_str!("../../../../../../tools/oracle/tkernel_linear_fill_constructor.expected");
    assert!(
        expected.lines().any(|native| native == row),
        "missing native fill row {row:?}"
    );
}

fn run(spec: &FillSpec) {
    assert_eq!(
        spec.fail, 0,
        "case {}: expected a nonthrowing copy",
        spec.name
    );
    let trace = CopyTrace::default();
    let donor = (!spec.owned).then(|| CopyItem::new(99, &trace));
    trace.active.set(true);
    let filled = if spec.owned {
        LinearVector::with_value(spec.count, CopyItem::new(99, &trace))
    } else {
        LinearVector::with_value_ref(spec.count, donor.as_ref().unwrap())
    };
    assert_native(&row("native", spec, Some(&filled), &trace, true));
    trace.active.set(false);
    drop((filled, donor));
    assert_eq!(trace.resources.get(), 0);
}

macro_rules! native_fill {
    ($name:ident,$index:literal) => {
        #[test]
        fn $name() {
            run(&FILL_CASES[$index]);
        }
    };
}
native_fill!(borrowed_zero, 0);
native_fill!(borrowed_one, 1);
native_fill!(borrowed_two, 2);
native_fill!(borrowed_three, 3);
native_fill!(borrowed_five, 4);
native_fill!(owned_zero, 10);
native_fill!(owned_one, 11);
native_fill!(owned_two, 12);
native_fill!(owned_three, 13);
native_fill!(owned_five, 14);
