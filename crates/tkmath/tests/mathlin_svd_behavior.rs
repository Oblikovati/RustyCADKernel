//! Native SVD padding, sorting, validation and inverse behavior.
use rustycad_tkmath::{
    math::{Matrix, Vector},
    math_lin::*,
    math_utils::Status,
};
fn diagonal() -> Matrix<'static> {
    let mut a: Matrix<'static> = Matrix::filled(-1, 0, 4, 6, 0.0).unwrap();
    a[(-1, 4)] = 2.0;
    a[(0, 5)] = 4.0;
    a
}
#[test]
fn wide_factors_sort_and_truncate_padding() {
    let a: Matrix<'static> = diagonal();
    let result: SvdResult = svd(&a, 1e-15);
    assert!(bool::from(&result));
    assert_eq!(result.rank, 2);
    assert_eq!(result.condition_number, Some(2.0));
    let w: Vector<'static> = result.singular_values.unwrap();
    assert_eq!((w[0], w[1], w[2]), (4.0, 2.0, 0.0));
    let u: Matrix<'static> = result.u.unwrap();
    let v: Matrix<'static> = result.v.unwrap();
    for i in 0..2 {
        for j in 0..3 {
            let sum: f64 = (0..3).map(|k| u[(i, k)] * w[k] * v[(j, k)]).sum();
            assert_eq!(sum, *a.at(i as usize, j as usize).unwrap());
        }
    }
}
#[test]
fn solves_minimum_norm_and_pseudoinverse() {
    let a: Matrix<'static> = diagonal();
    let b: Vector<'static> = Vector::filled(8, 9, 8.0).unwrap();
    let x: Vector<'static> = solve_svd(&a, &b, 1e-6).solution.unwrap();
    assert_eq!((x[0], x[1], x[2]), (4.0, 2.0, 0.0));
    let inverse: Matrix<'static> = pseudo_inverse(&a, 1e-6).inverse.unwrap();
    assert_eq!(
        (inverse[(0, 0)], inverse[(1, 1)], inverse[(2, 1)]),
        (0.5, 0.25, 0.0)
    );
    assert_eq!(condition_number(&a), Some(2.0));
    assert_eq!(numerical_rank(&a, 0.6), 1);
}
#[test]
fn invalid_and_rank_deficient_statuses() {
    let mut a: Matrix<'static> = diagonal();
    let b: Vector<'static> = Vector::with_size(1);
    assert_eq!(solve_svd(&a, &b, 1e-6).status, Status::InvalidInput);
    assert_eq!(svd(&a, -1.0).status, Status::InvalidInput);
    a[(-1, 4)] = 0.0;
    assert_eq!(condition_number(&a), None);
    a[(0, 5)] = f64::NAN;
    assert_eq!(svd(&a, 1e-15).status, Status::InvalidInput);
    assert_eq!(numerical_rank(&a, 1e-15), 0);
    assert_eq!(pseudo_inverse(&a, 1e-6).status, Status::InvalidInput);
    assert_eq!(solve_svd(&a, &b, 1e-6).status, Status::InvalidInput);
}
