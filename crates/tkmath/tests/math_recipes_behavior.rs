//! math_Recipes kernels and the Dump text of the solvers built on them.
use rustycad_tkmath::math::recipes::{
    STATUS_NO_CONVERGENCE, STATUS_OK, STATUS_SINGULAR_MATRIX, STATUS_USER_ABORTED,
    dactcl_decompose, dactcl_solve, jacobi, lu_decompose, lu_decompose_with_scales, lu_invert,
    lu_solve, svd_decompose, svd_decompose_with_work, svd_solve,
};
use rustycad_tkmath::math::{Gauss, GaussLeastSquares, IntegerVector, Jacobi, Matrix, Svd, Vector};

fn matrix(rows: &[&[f64]]) -> Matrix<'static> {
    let (r, c) = (rows.len() as i32, rows[0].len() as i32);
    let mut result = Matrix::with_bounds(1, r, 1, c).unwrap();
    for (i, row) in rows.iter().enumerate() {
        for (j, value) in row.iter().enumerate() {
            *result.change_at(i, j).unwrap() = *value;
        }
    }
    result
}
fn vector(items: &[f64]) -> Vector<'static> {
    let mut result = Vector::with_bounds(1, items.len() as i32).unwrap();
    for (offset, item) in items.iter().enumerate() {
        *result.change_at(offset).unwrap() = *item;
    }
    result
}

#[test]
fn status_codes_match_the_native_constants() {
    assert_eq!(
        [
            STATUS_USER_ABORTED,
            STATUS_OK,
            STATUS_SINGULAR_MATRIX,
            STATUS_NO_CONVERGENCE
        ],
        [-1, 0, 1, 3]
    );
}
#[test]
fn lu_pivots_rows_and_solves_with_any_rhs_lower_bound() {
    let mut a = matrix(&[&[0.0, 1.0], &[2.0, 0.0]]);
    let mut indx = IntegerVector::with_bounds(1, 2).unwrap();
    let mut d = 0.0;
    assert_eq!(
        lu_decompose(&mut a, &mut indx, &mut d, 1e-20).unwrap(),
        STATUS_OK
    );
    assert_eq!((indx[1], indx[2], d), (2, 2, -1.0));
    let mut b = Vector::with_bounds(-4, -3).unwrap();
    b[-4] = 3.0;
    b[-3] = 4.0;
    lu_solve(&a, &indx, &mut b).unwrap();
    assert_eq!((b[-4], b[-3]), (2.0, 3.0));
}
#[test]
fn lu_reports_singular_rows_and_pivots() {
    let mut zero_row = matrix(&[&[1.0, 2.0], &[0.0, 0.0]]);
    let mut indx = IntegerVector::with_bounds(1, 2).unwrap();
    let mut scales = Vector::filled(1, 2, 7.0).unwrap();
    let mut d = 0.0;
    let status = lu_decompose_with_scales(&mut zero_row, &mut indx, &mut d, &mut scales, 1e-30);
    assert_eq!(status.unwrap(), STATUS_SINGULAR_MATRIX);
    assert_eq!(
        (scales[1], scales[2]),
        (0.5, 7.0),
        "scales stop at the zero row"
    );
    let mut rank_one = matrix(&[&[1.0, 2.0], &[2.0, 4.0]]);
    assert_eq!(lu_invert(&mut rank_one).unwrap(), STATUS_SINGULAR_MATRIX);
}
#[test]
fn lu_invert_replaces_the_matrix() {
    let mut a = matrix(&[&[4.0, 7.0], &[2.0, 6.0]]);
    assert_eq!(lu_invert(&mut a).unwrap(), STATUS_OK);
    assert!((a[(1, 1)] - 0.6).abs() < 1e-15 && (a[(2, 1)] + 0.2).abs() < 1e-15);
}
#[test]
fn kernels_refuse_arrays_that_do_not_cover_their_one_based_indices() {
    let mut shifted = Matrix::with_bounds(0, 1, 0, 1).unwrap();
    let mut indx = IntegerVector::with_bounds(1, 2).unwrap();
    let mut d = 0.0;
    let error = lu_decompose(&mut shifted, &mut indx, &mut d, 1e-20).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert!(error.message().contains("1..2"), "{}", error.message());
    let mut wide = Matrix::with_bounds(1, 1, 1, 2).unwrap();
    let mut w = Vector::with_bounds(1, 2).unwrap();
    let mut v = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    let error = svd_decompose(&mut wide, &mut w, &mut v).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_DimensionError");
}
#[test]
fn svd_orders_nothing_but_makes_values_nonnegative_and_solves() {
    let mut u = matrix(&[&[3.0, 0.0], &[0.0, -2.0], &[0.0, 0.0]]);
    let mut w = Vector::with_bounds(1, 2).unwrap();
    let mut v = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    let mut rv1 = Vector::with_bounds(1, 2).unwrap();
    assert_eq!(
        svd_decompose_with_work(&mut u, &mut w, &mut v, &mut rv1).unwrap(),
        STATUS_OK
    );
    assert_eq!((w[1], w[2]), (3.0, 2.0));
    let mut x = Vector::with_bounds(1, 2).unwrap();
    svd_solve(&u, &w, &v, &vector(&[6.0, 4.0, 1.0]), &mut x).unwrap();
    assert!((x[1] - 2.0).abs() < 1e-15 && (x[2] + 2.0).abs() < 1e-15);
}
#[test]
fn jacobi_sorts_descending_and_counts_rotations() {
    let mut a = matrix(&[&[2.0, 1.0], &[1.0, 2.0]]);
    let mut d = Vector::with_bounds(1, 2).unwrap();
    let mut v = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    let mut rotations = 0;
    assert_eq!(
        jacobi(&mut a, &mut d, &mut v, &mut rotations).unwrap(),
        STATUS_OK
    );
    assert_eq!(rotations, 1);
    assert!((d[1] - 3.0).abs() < 1e-15 && (d[2] - 1.0).abs() < 1e-15);
}
#[test]
fn dactcl_factors_and_solves_a_skyline_system() {
    // [[4, 2], [2, 5]] stored as column profiles: a = [4, 2, 5], diagonals at 1 and 3.
    let mut a = vector(&[4.0, 2.0, 5.0]);
    let mut indx = IntegerVector::with_bounds(1, 2).unwrap();
    indx[1] = 1;
    indx[2] = 3;
    assert_eq!(dactcl_decompose(&mut a, &indx, 1e-20).unwrap(), STATUS_OK);
    let mut b = vector(&[8.0, 19.0]);
    assert_eq!(dactcl_solve(&a, &mut b, &indx, 1e-20).unwrap(), STATUS_OK);
    assert!((b[1] - 0.125).abs() < 1e-15 && (b[2] - 3.75).abs() < 1e-15);
    let mut singular = vector(&[0.0, 1.0, 1.0]);
    assert_eq!(
        dactcl_decompose(&mut singular, &indx, 1e-20).unwrap(),
        STATUS_SINGULAR_MATRIX
    );
    indx[2] = 9;
    assert_eq!(
        dactcl_decompose(&mut a, &indx, 1e-20)
            .unwrap_err()
            .exception_type(),
        "Standard_OutOfRange"
    );
}
#[test]
fn solver_dumps_follow_native_text() {
    let a = matrix(&[&[0.0, 1.0], &[1.0, 0.0]]);
    assert_eq!(
        Gauss::new(&a).unwrap().to_string(),
        "math_Gauss  Status = Done \n Determinant of A = -1\n"
    );
    let singular = matrix(&[&[0.0, 0.0], &[0.0, 0.0]]);
    assert_eq!(
        Gauss::new(&singular).unwrap().to_string(),
        "math_Gauss  Status = not Done \n"
    );
    let mut text = String::new();
    GaussLeastSquares::new(&singular)
        .unwrap()
        .dump(&mut text)
        .unwrap();
    assert_eq!(text, "math_GaussLeastSquare Status = not Done \n");
    text.clear();
    Svd::new(&a).unwrap().dump(&mut text).unwrap();
    assert_eq!(text, "math_SVD Status = Done \n");
    text.clear();
    Jacobi::new(&a).unwrap().dump(&mut text).unwrap();
    assert!(
        text.starts_with("math_Jacobi  Status = Done \n The eigenvalues vector is: math_Vector")
    );
}
