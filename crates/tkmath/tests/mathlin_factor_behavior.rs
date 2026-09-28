//! Direct factor reconstruction and native failure-state coverage.
use rustycad_tkmath::{
    math::{Matrix, Vector},
    math_lin::*,
    math_utils::Status,
};
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-10, "{a} != {b}");
}
#[test]
fn crout_reconstructs_and_ignores_nonfinite_upper_triangle() {
    let mut a = Matrix::filled(-2, -1, 4, 5, 0.0).unwrap();
    a[(-2, 4)] = 4.0;
    a[(-1, 4)] = 2.0;
    a[(-1, 5)] = -2.0;
    a[(-2, 5)] = f64::NAN;
    let result = crout(&a, 1e-15);
    assert!(bool::from(&result));
    assert_eq!(result.determinant, Some(-12.0));
    let l = result.lower.as_ref().unwrap();
    let d = result.diagonal.as_ref().unwrap();
    assert_eq!(l[(1, 0)], 0.5);
    assert_eq!(d[1], -3.0);
    let b = Vector::filled(7, 8, 1.0).unwrap();
    let x = solve_crout(&a, &b, 1e-15).solution.unwrap();
    near(4.0 * x[0] + 2.0 * x[1], 1.0);
    near(2.0 * x[0] - 2.0 * x[1], 1.0);
    assert!(invert_crout(&a, 1e-15).is_done());
    a[(-2, 4)] = 0.0;
    assert_eq!(crout(&a, 1e-15).status, Status::Singular);
    assert_eq!(crout(&a, -1.0).status, Status::InvalidInput);
}
#[test]
fn qr_reconstructs_rectangular_matrix_and_solves_multiple_rhs() {
    let mut a = Matrix::filled(-1, 1, 4, 5, 0.0).unwrap();
    for i in 0..3 {
        *a.change_at(i, 0).unwrap() = 1.0;
        *a.change_at(i, 1).unwrap() = i as f64;
    }
    let result = qr(&a, 1e-15);
    assert!(bool::from(&result));
    assert_eq!(result.rank, 2);
    let rebuilt = result
        .q
        .unwrap()
        .multiplied_matrix(&result.r.unwrap())
        .unwrap();
    for i in 0..3 {
        for j in 0..2 {
            near(*rebuilt.at(i, j).unwrap(), *a.at(i, j).unwrap());
        }
    }
    let mut b = Vector::with_size(3);
    b[0] = 2.0;
    b[1] = 5.0;
    b[2] = 8.0;
    let x = solve_qr(&a, &b, 1e-15).solution.unwrap();
    near(x[0], 2.0);
    near(x[1], 3.0);
    let mut rhs = Matrix::filled(0, 2, 0, 1, 0.0).unwrap();
    for i in 0..3 {
        rhs[(i, 0)] = b[i];
        rhs[(i, 1)] = 1.0;
    }
    let x = solve_qr_multiple(&a, &rhs, 1e-15).solutions.unwrap();
    near(x[(0, 0)], 2.0);
    near(x[(1, 0)], 3.0);
    near(x[(0, 1)], 1.0);
    near(x[(1, 1)], 0.0);
}
#[test]
fn factor_errors_keep_native_status_distinctions() {
    let zero = Matrix::filled(0, 1, 0, 1, 0.0).unwrap();
    let b = Vector::filled(0, 1, 1.0).unwrap();
    assert_eq!(crout(&zero, 1e-15).status, Status::Singular);
    assert_eq!(solve_crout(&zero, &b, 1e-15).status, Status::Singular);
    assert_eq!(invert_crout(&zero, 1e-15).status, Status::Singular);
    let r = qr(&zero, 1e-15);
    assert!(r.is_done());
    assert_eq!(r.rank, 0);
    assert_eq!(solve_qr(&zero, &b, 1e-15).status, Status::Singular);
    assert_eq!(
        solve_qr_multiple(&zero, &zero, 1e-15).status,
        Status::Singular
    );
    let wrong = Vector::with_size(1);
    assert_eq!(solve_qr(&zero, &wrong, 1e-15).status, Status::InvalidInput);
    let mut nonfinite = zero.clone();
    nonfinite[(0, 0)] = f64::INFINITY;
    assert_eq!(qr(&nonfinite, 1e-15).status, Status::InvalidInput);
    assert_eq!(crout(&nonfinite, 1e-15).status, Status::InvalidInput);
}
