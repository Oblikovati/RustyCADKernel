//! Native HArray1 value construction retains its donor and supports assignment-only types.
#[path = "harray1_borrowed_initialization/fixture.rs"]
mod fixture;
use fixture::{ClonePolicy, FillProbe, FillScope};
use rustycad_tkernel::{
    ncollection::HArray1,
    standard::{Handle, Transient},
};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn success_state(array: &HArray1<FillProbe>, donor: &FillProbe, scope: &FillScope) -> String {
    let base = array.array1();
    let values = base
        .iter()
        .map(|v| v.number.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "success {} {} {} {values} {} {} {}\n",
        base.lower(),
        base.upper(),
        base.len(),
        array.transient_base().ref_count(),
        donor.reads(),
        scope.take_events()
    )
}
fn successful_rows() -> String {
    let scope = FillScope::new(0);
    let donor = scope.donor(9);
    let array = HArray1::with_value_ref_with_assignment(-1, 1, &donor, FillProbe::assign).unwrap();
    let mut rows = success_state(&array, &donor, &scope);
    drop(array);
    rows += &format!("success_drop {}\n", scope.take_events());
    rows += &format!("donor_retained {} {}\n", donor.number, donor.reads());
    rows
}
fn empty_row() -> String {
    let scope = FillScope::new(0);
    let donor = scope.donor(9);
    let array = HArray1::with_value_ref_with_assignment(4, 3, &donor, FillProbe::assign).unwrap();
    let base = array.array1();
    format!(
        "empty {} {} {} {} {} {}\n",
        base.lower(),
        base.upper(),
        base.len(),
        u8::from(base.is_deletable()),
        donor.reads(),
        scope.take_events()
    )
}
fn failed_row(fail: usize) -> String {
    let scope = FillScope::new(fail);
    let donor = scope.donor(9);
    let failed = catch_unwind(AssertUnwindSafe(|| {
        HArray1::with_value_ref_with_assignment(-1, 1, &donor, FillProbe::assign)
    }));
    assert!(failed.is_err());
    format!(
        "fail{fail} {} {} {}\n",
        donor.number,
        donor.reads(),
        scope.take_events()
    )
}
fn handled_row() -> String {
    let first = Handle::new(HArray1::with_value_ref(-2, 0, &7).unwrap());
    let one = first.get().unwrap().transient_base().ref_count();
    let second = first.clone();
    let two = first.get().unwrap().transient_base().ref_count();
    let owner = first.get().unwrap();
    owner.change_array1().set_value(-1, 55).unwrap();
    let observed = *second.get().unwrap().array1().value(-1).unwrap();
    drop(second);
    let final_count = first.get().unwrap().transient_base().ref_count();
    let base = first.get().unwrap().array1();
    format!(
        "handle {one} {two} {final_count} {observed} {} {}\n",
        base.lower(),
        base.upper()
    )
}
#[test]
fn native_fill_constructor_rows_match_success_failure_and_handle_observations() {
    let mut actual = successful_rows() + &empty_row();
    for fail in 1..=3 {
        actual += &failed_row(fail);
    }
    actual += &handled_row();
    assert_eq!(
        actual,
        include_str!("../../../tools/oracle/tkernel_harray1_borrowed_initialization.expected")
    );
}
#[test]
fn borrowed_clone_policy_assigns_without_copy_constructing_the_donor() {
    let scope = FillScope::new(0);
    let donor = ClonePolicy(scope.donor(9));
    let array = HArray1::with_value_ref(1, 2, &donor).unwrap();
    assert_eq!(
        array
            .array1()
            .as_slice()
            .iter()
            .map(|x| x.0.number)
            .collect::<Vec<_>>(),
        [9, 9]
    );
    assert_eq!(donor.0.reads(), 2);
    assert_eq!(scope.take_events(), "d1;d2;a1=9;a2=9;");
    assert!(catch_unwind(AssertUnwindSafe(|| donor.clone())).is_err());
}
#[test]
fn invalid_safe_bounds_skip_defaults_and_assignment_without_consuming_donor() {
    let scope = FillScope::new(0);
    let donor = scope.donor(9);
    let result = HArray1::with_value_ref_with_assignment(3, 0, &donor, FillProbe::assign);
    assert!(result.is_err());
    assert_eq!((donor.number, donor.reads()), (9, 0));
    assert_eq!(scope.take_events(), "-");
}
#[test]
fn nested_constructor_journals_restore_the_outer_default_context() {
    let outer = FillScope::new(0);
    {
        let inner = FillScope::new(0);
        drop(FillProbe::default());
        assert_eq!(inner.take_events(), "d1;x1=-1;");
    }
    drop(FillProbe::default());
    assert_eq!(outer.take_events(), "d1;x1=-1;");
}
