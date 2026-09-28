//! Native factor, SVD and least-squares result comparisons.
use super::modern_eigen_probe::{fields, optional};
use rustycad_tkmath::{
    math::{Matrix, Vector},
    math_lin::*,
    math_utils::{InverseResult, LinearResult},
};
fn linear(r: LinearResult) {
    print!("{} ", r.status as i32);
    optional(r.determinant);
    fields(&r.solution, &None);
}
fn inverse(r: InverseResult) {
    print!("{} ", r.status as i32);
    optional(r.determinant);
    fields(&None, &r.inverse);
}
fn least(r: LeastSquaresResult) {
    print!("{} {} ", r.status as i32, r.rank);
    optional(r.condition_number);
    optional(r.residual);
    optional(r.residual_sq);
    fields(&r.solution, &None);
}
fn crout_fields(a: &Matrix<'_>, b: &Vector<'_>, tol: f64) {
    let r = crout(a, tol);
    print!(
        "{} {} {} ",
        r.status as i32,
        r.min_abs_pivot.to_bits(),
        r.max_abs_pivot.to_bits()
    );
    optional(r.determinant);
    optional(r.condition_estimate);
    fields(&r.diagonal, &r.lower);
    fields(&None, &r.inverse);
    linear(solve_crout(a, b, tol));
    inverse(invert_crout(a, tol));
}
fn qr_fields(a: &Matrix<'_>, b: &Vector<'_>, tol: f64) {
    let r = qr(a, tol);
    print!(
        "{} {} {} ",
        r.status as i32,
        r.rank,
        r.diagonal_threshold.to_bits()
    );
    optional(r.condition_estimate);
    fields(&None, &r.q);
    fields(&None, &r.r);
    linear(solve_qr(a, b, tol));
    let mut rhs = Matrix::with_bounds(-2, a.row_number() - 3, 3, 4).unwrap();
    for i in 0..a.row_number() as usize {
        *rhs.change_at(i, 0).unwrap() = *b.at(i).unwrap();
        *rhs.change_at(i, 1).unwrap() = i as f64;
    }
    let r = solve_qr_multiple(a, &rhs, tol);
    print!("{} ", r.status as i32);
    fields(&None, &r.solutions);
}
fn svd_fields(a: &Matrix<'_>, b: &Vector<'_>, tol: f64) {
    let r = svd(a, tol);
    print!("{} {} ", r.status as i32, r.rank);
    optional(r.condition_number);
    fields(&r.singular_values, &r.u);
    fields(&None, &r.v);
    linear(solve_svd(a, b, tol));
    inverse(pseudo_inverse(a, tol));
    optional(condition_number(a));
    print!("{} ", numerical_rank(a, tol));
}
pub(super) fn modern_factors_case(v: &[f64]) {
    let m = v[0] as usize;
    let n = v[1] as usize;
    let mode = v[2] as i32;
    let tol = v[3];
    let lambda = v[4];
    let mut a = Matrix::with_bounds(-2, m as i32 - 3, 4, n as i32 + 3).unwrap();
    let mut b = Vector::with_bounds(5, m as i32 + 4).unwrap();
    for i in 0..m {
        for j in 0..n {
            *a.change_at(i, j).unwrap() = v[5 + i * n + j];
        }
        *b.change_at(i).unwrap() = v[5 + m * n + i];
    }
    if mode == 1 {
        *a.change_at(0, 0).unwrap() = f64::INFINITY;
    }
    if mode == 2 {
        *b.change_at(0).unwrap() = f64::NAN;
    }
    if mode == 3 && n > 1 {
        *a.change_at(0, 1).unwrap() = f64::NAN;
    }
    crout_fields(&a, &b, tol);
    qr_fields(&a, &b, tol);
    svd_fields(&a, &b, tol);
    let mut weights = Vector::filled(0, m as i32 - 1, 1.0).unwrap();
    if mode == 4 {
        weights[0] = 0.0;
    } else {
        weights[0] = 2.0;
    }
    for method in [
        LeastSquaresMethod::NormalEquations,
        LeastSquaresMethod::Qr,
        LeastSquaresMethod::Svd,
        LeastSquaresMethod::Invalid(4),
    ] {
        least(least_squares(&a, &b, method, tol));
        least(weighted_least_squares(&a, &b, &weights, method, tol));
    }
    least(regularized_least_squares(&a, &b, lambda, tol));
    optional(optimal_regularization(&a, &b, 1e-10, 1e2, 20));
    println!();
}
