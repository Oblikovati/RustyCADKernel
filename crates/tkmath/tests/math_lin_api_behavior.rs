//! Supplemental checks for the public MathLin::Utils helpers (MathLin_EigenSearch.hxx and
//! MathLin_Utils.hxx at 3d097a0); bit-level parity lives in the `integ_lin_*` numeric probes.
use rustycad_tkmath::math::{Matrix, Vector};
use rustycad_tkmath::math_lin::{eigen_tridiagonal, get_eigen_vector, utils};
use rustycad_tkmath::math_utils::Status;

fn vector(values: &[f64]) -> Vector<'static> {
    let mut v: Vector<'static> = Vector::with_size(values.len());
    for (i, value) in values.iter().enumerate() {
        *v.change_at(i).unwrap() = *value;
    }
    v
}

fn identity(n: usize) -> Matrix<'static> {
    let mut m: Matrix<'static> = Matrix::filled(0, n as i32 - 1, 0, n as i32 - 1, 0.0).unwrap();
    for i in 0..n {
        *m.change_at(i, i).unwrap() = 1.0;
    }
    m
}

#[test]
fn tridiagonal_constants_match_native_values() {
    assert_eq!(utils::TRIDIAGONAL_DEFLATION_TOLERANCE, f64::EPSILON);
    assert_eq!(utils::TRIDIAGONAL_RESIDUAL_TOLERANCE_FACTOR, 256.0);
}

#[test]
fn find_submatrix_end_stops_at_negligible_coupling() {
    let d: Vector<'static> = vector(&[1.0, 2.0, 3.0]);
    assert_eq!(
        utils::find_submatrix_end(&d, &vector(&[0.5, 0.0, 0.0]), 0, 3),
        1
    );
    assert_eq!(
        utils::find_submatrix_end(&d, &vector(&[0.5, 0.5, 0.0]), 0, 3),
        2
    );
    assert_eq!(
        utils::find_submatrix_end(&d, &vector(&[0.0, 0.5, 0.0]), 0, 3),
        0
    );
    // std::max keeps a NaN first operand, so the threshold is NaN and never deflates.
    let nan_first: Vector<'static> = vector(&[f64::NAN, 2.0]);
    assert_eq!(
        utils::find_submatrix_end(&nan_first, &vector(&[1e-300, 0.0]), 0, 2),
        1
    );
}

#[test]
fn wilkinson_shift_follows_native_sign_branches() {
    let d: Vector<'static> = vector(&[1.0, 3.0]);
    let e: Vector<'static> = vector(&[1.0, 0.0]);
    let ratio: f64 = (3.0 - 1.0) / 2.0;
    let expected: f64 = 3.0 - 1.0 + 1.0 / (ratio + 1.0_f64.hypot(ratio));
    assert_eq!(utils::compute_wilkinson_shift(&d, &e, 0, 1), expected);
    let reversed: Vector<'static> = vector(&[3.0, 1.0]);
    let negative: f64 = -1.0;
    let expected: f64 = 1.0 - 3.0 + 1.0 / (negative - 1.0_f64.hypot(negative));
    assert_eq!(
        utils::compute_wilkinson_shift(&reversed, &e, 0, 1),
        expected
    );
}

#[test]
fn ql_steps_converge_to_the_eigenvalues_of_a_two_by_two_block() {
    let (mut d, mut e, mut v) = (vector(&[2.0, 2.0]), vector(&[1.0, 0.0]), identity(2));
    for _ in 0..30 {
        if utils::find_submatrix_end(&d, &e, 0, 2) == 0 {
            break;
        }
        let shift: f64 = utils::compute_wilkinson_shift(&d, &e, 0, 1);
        assert!(utils::perform_ql_step(
            &mut d, &mut e, &mut v, 0, 1, shift, 2
        ));
    }
    let mut values: Vec<f64> = vec![*d.at(0).unwrap(), *d.at(1).unwrap()];
    values.sort_by(f64::total_cmp);
    assert!((values[0] - 1.0).abs() <= 1e-15 && (values[1] - 3.0).abs() <= 1e-15);
    assert!(utils::validate_eigenpairs(
        &vector(&[2.0, 2.0]),
        &vector(&[1.0, 0.0]),
        &d,
        &v
    ));
}

#[test]
fn ql_step_zero_radius_exit_clears_the_end_coupling() {
    let (mut d, mut e, mut v) = (vector(&[1.0, 4.0]), vector(&[0.0, 7.0]), identity(2));
    assert!(utils::perform_ql_step(&mut d, &mut e, &mut v, 0, 1, 0.0, 2));
    assert_eq!(*e.at(1).unwrap(), 0.0);
    assert_eq!(*d.at(1).unwrap(), 4.0);
    assert_eq!(*v.at(0, 0).unwrap(), 1.0);
}

#[test]
fn validate_eigenpairs_rejects_wrong_values_and_accepts_zero_matrices() {
    let (d, e) = (vector(&[2.0, 2.0]), vector(&[1.0, 0.0]));
    // Natural layout: n - 1 couplings (an n-length vector is the legacy layout, which skips e[0]).
    let result = eigen_tridiagonal(&d, &vector(&[1.0]), 30);
    assert_eq!(result.status, Status::Ok);
    let (values, vectors) = (
        result.eigen_values.as_ref().unwrap(),
        result.eigen_vectors.as_ref().unwrap(),
    );
    assert!(utils::validate_eigenpairs(&d, &e, values, vectors));
    let wrong: Vector<'static> = vector(&[*values.at(0).unwrap() + 1e-3, *values.at(1).unwrap()]);
    assert!(!utils::validate_eigenpairs(&d, &e, &wrong, vectors));
    let zero: Vector<'static> = vector(&[0.0, 0.0]);
    assert!(utils::validate_eigenpairs(
        &zero,
        &zero,
        &vector(&[5.0, 5.0]),
        &identity(2)
    ));
    assert_eq!(get_eigen_vector(&result, 1).unwrap().length(), 2);
}
