use super::{FillProbe, FillScope};
use rustycad_tkernel::{
    ncollection::{Array2, HArray2},
    standard::{Handle, Transient},
};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn rectangle_bounds(matrix: &Array2<'_, FillProbe>) -> String {
    format!(
        "{} {} {} {}",
        matrix.lower_row(),
        matrix.upper_row(),
        matrix.lower_col(),
        matrix.upper_col()
    )
}
fn matrix_numbers(matrix: &Array2<'_, FillProbe>) -> String {
    matrix
        .iter()
        .map(|cell| cell.number.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
fn successful_state(matrix: &HArray2<FillProbe>, donor: &FillProbe, scope: &FillScope) -> String {
    let view = matrix.array2();
    format!(
        "success {} {} {} {} {} {}\n",
        rectangle_bounds(&view),
        view.len(),
        matrix_numbers(&view),
        matrix.transient_base().ref_count(),
        donor.reads(),
        scope.take_events()
    )
}
fn successful_matrix() -> String {
    let scope = FillScope::new(0);
    let donor = scope.donor(9);
    let matrix =
        HArray2::with_value_ref_with_assignment(-1, 0, 3, 4, &donor, FillProbe::assign).unwrap();
    let mut rows = successful_state(&matrix, &donor, &scope);
    drop(matrix);
    rows += &format!("success_drop {}\n", scope.take_events());
    rows += &format!("donor_retained {} {}\n", donor.number, donor.reads());
    rows
}
fn empty_matrix(name: &str, bounds: (i32, i32, i32, i32)) -> String {
    let scope = FillScope::new(0);
    let donor = scope.donor(9);
    let (rl, ru, cl, cu) = bounds;
    let matrix =
        HArray2::with_value_ref_with_assignment(rl, ru, cl, cu, &donor, FillProbe::assign).unwrap();
    let view = matrix.array2();
    format!(
        "{name} {} {} {} {} {} {} {}\n",
        rectangle_bounds(&view),
        view.nb_rows(),
        view.nb_columns(),
        view.len(),
        u8::from(view.is_deletable()),
        donor.reads(),
        scope.take_events()
    )
}
fn failed_matrix(fail: usize) -> String {
    let scope = FillScope::new(fail);
    let donor = scope.donor(9);
    let failed = catch_unwind(AssertUnwindSafe(|| {
        HArray2::with_value_ref_with_assignment(-1, 0, 3, 4, &donor, FillProbe::assign)
    }));
    assert!(failed.is_err());
    format!(
        "fail{fail} {} {} {}\n",
        donor.number,
        donor.reads(),
        scope.take_events()
    )
}
fn handled_matrix() -> String {
    let first = Handle::new(HArray2::with_value_ref(-1, 0, 3, 4, &7).unwrap());
    let owner = first.get().unwrap();
    let one = owner.transient_base().ref_count();
    let second = first.clone();
    let two = owner.transient_base().ref_count();
    owner.change_array2().set_value(0, 3, 55).unwrap();
    let observed = *second.get().unwrap().array2().value(0, 3).unwrap();
    drop(second);
    let final_count = owner.transient_base().ref_count();
    let view = owner.array2();
    format!(
        "handle {one} {two} {final_count} {observed} {} {} {} {}\n",
        view.lower_row(),
        view.upper_row(),
        view.lower_col(),
        view.upper_col()
    )
}
/// Generates each native row from live Rust state; `matrix_rows()`.
pub fn matrix_rows() -> String {
    let mut rows = successful_matrix();
    rows += &empty_matrix("empty_rows", (1, 0, 3, 4));
    rows += &empty_matrix("empty_cols", (1, 2, 3, 2));
    rows += &empty_matrix("empty_both", (1, 0, 3, 2));
    for fail in 1..=4 {
        rows += &failed_matrix(fail);
    }
    rows + &handled_matrix()
}
