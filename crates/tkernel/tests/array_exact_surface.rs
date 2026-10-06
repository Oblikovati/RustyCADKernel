//! Compiler-bound existing operations are exercised without cosmetic public aliases.
#[path = "array_exact_surface/support.rs"]
mod support;
use rustycad_tkernel::ncollection::{Array1, Array2};
use support::{assert_surface_row, check_lifecycle, linear_queries, matrix_queries};

#[test]
fn native_default_constructor_queries_match_default_trait_bodies() {
    assert_surface_row("default_linear", &linear_queries(&Array1::<i32>::default()));
    assert_surface_row("default_matrix", &matrix_queries(&Array2::<i32>::default()));
}
#[test]
fn native_static_range_layout_and_unsigned_product_helpers_match() {
    let values = [
        Array1::<i32>::range_size(-2, 3).to_string(),
        Array1::<i32>::range_size(5, 3).to_string(),
        Array2::<i32>::range_size(-2, 3).to_string(),
        Array2::<i32>::array_size(usize::MAX, 2).to_string(),
        Array2::<i32>::upper_for_size(4, 0).to_string(),
        Array2::<i32>::upper_for_size(3, usize::MAX).to_string(),
        Array2::<i32>::begin_position(-2, -1, 3, 5).to_string(),
        Array2::<i32>::last_position(-2, -1, 3, 5).to_string(),
    ];
    assert_surface_row("static", &values.join(":"));
}
#[test]
fn native_bound_updates_change_metadata_without_rebasing_inherited_storage() {
    let mut first = Array1::<i32>::with_bounds(-2, 1).unwrap();
    let mut second = Array2::<i32>::with_bounds(-2, -1, 3, 5).unwrap();
    first.update_lower_bound(7);
    first.update_upper_bound(14);
    second.update_lower_row(7);
    second.update_upper_row(14);
    second.update_lower_col(-7);
    second.update_upper_col(-2);
    assert_surface_row("updated_linear", &linear_queries(&first));
    let actual = format!(
        "{};{}",
        matrix_queries(&second),
        linear_queries(second.array1())
    );
    assert_surface_row("updated_matrix", &actual);
}
#[test]
fn native_linear_move_constructor_and_destruction_match_take_and_drop() {
    check_lifecycle(false, false);
    check_lifecycle(false, true);
}
#[test]
fn native_matrix_move_constructor_and_destruction_match_take_and_drop() {
    check_lifecycle(true, false);
    check_lifecycle(true, true);
}
