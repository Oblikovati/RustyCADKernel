//! Regressions for native PLib quirks and the port's documented rejections.
#![allow(clippy::excessive_precision)] // Assertions quote the native table literals verbatim.
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::plib::{self, HermitJacobi, JacobiPolynomial};

fn reals(lower: i32, values: &[f64]) -> Array1<'static, f64> {
    let mut result: Array1<'static, f64> =
        Array1::with_bounds(lower, lower + values.len() as i32 - 1).unwrap();
    result.as_mut_slice().copy_from_slice(values);
    result
}

#[test]
fn order_zero_request_uses_general_horner_branch() {
    // EvalPolynomial(DerivativeRequest=0) starts from 0*U + c, so an infinite U gives NaN,
    // while NoDerivativeEvalPolynomial copies c first.
    let mut general = [0.0];
    plib::eval_polynomial(f64::INFINITY, 0, 0, 1, &[2.0], &mut general).unwrap();
    assert!(general[0].is_nan());
    let mut direct = [0.0];
    plib::no_derivative_eval_polynomial(f64::INFINITY, 0, 1, 0, &[2.0], &mut direct).unwrap();
    assert_eq!(direct[0], 2.0);
}

#[test]
fn undefined_native_counts_and_short_buffers_are_rejected() {
    let mut out = [0.0; 4];
    let negative = plib::eval_polynomial(1.0, -1, 2, 1, &[1.0, 2.0, 3.0], &mut out).unwrap_err();
    assert_eq!(negative.exception_type(), "Standard_OutOfRange");
    assert!(negative.message().contains("DerivativeRequest = -1"));
    let short = plib::eval_polynomial(1.0, 3, 2, 1, &[1.0, 2.0, 3.0], &mut out[..3]).unwrap_err();
    assert!(
        short
            .message()
            .contains("Results holds 3 values; expected at least 4")
    );
}

#[test]
fn binomial_failures_name_the_offending_arguments() {
    assert_eq!(plib::bin(25, 12).unwrap(), 5200300.0);
    let degree = plib::bin(26, 1).unwrap_err();
    assert_eq!(degree.exception_type(), "Standard_OutOfRange");
    assert!(degree.message().contains("N = 26"));
    assert!(plib::bin(5, 6).unwrap_err().message().contains("P = 6"));
}

#[test]
fn rational_derivative_with_negative_degree_only_scales_zero_rows() {
    // Degree < 0 skips every numerator row; only the leading weight is read.
    let mut out = [7.0; 4];
    plib::rational_derivative(-1, 1, 2, &[5.0, 6.0, 2.0], &mut out, true).unwrap();
    assert_eq!(out, [0.0; 4]);
}

#[test]
fn rational_derivatives_match_the_quotient_rule() {
    // f = (1 + 2t) / (1 + t) at t = 0: f = 1, f' = 2 - 1 = 1.
    let mut out = [0.0; 2];
    plib::rational_derivatives(1, 1, &[1.0, 2.0], &[1.0, 1.0], &mut out).unwrap();
    assert_eq!(out, [1.0, 1.0]);
}

#[test]
fn lagrange_coincident_nodes_return_one_and_leave_results() {
    let mut out = [91.0; 2];
    let code = plib::eval_lagrange(0.5, 1, 1, 1, &[0.0, 1.0], &[1.0, 1.0], &mut out).unwrap();
    assert_eq!((code, out), (1, [91.0, 91.0]));
    let code = plib::eval_lagrange(0.25, 1, 1, 1, &[0.0, 1.0], &[0.0, 1.0], &mut out).unwrap();
    assert_eq!((code, out), (0, [0.25, 1.0]));
}

#[test]
fn cubic_hermite_reproduces_end_data() {
    let mut out = [0.0; 2];
    plib::eval_cubic_hermite(1.0, 1, 1, &[0.0, 1.0], &[1.0, 0.0], &[0.0, 1.0], &mut out).unwrap();
    assert_eq!(out, [1.0, 0.0]);
}

#[test]
fn singular_hermite_system_raises_from_the_native_solve() {
    let constraints: Array2<f64> = Array2::with_bounds(1, 1, 0, 0).unwrap();
    let mut coefficients = reals(0, &[0.0, 0.0]);
    let error = plib::hermite_interpolate(
        1,
        2.0,
        2.0,
        0,
        0,
        &constraints,
        &constraints,
        &mut coefficients,
    )
    .unwrap_err();
    assert_eq!(error.exception_type(), "StdFail_NotDone");
    // With no components the solve is never reached, so the singular system reports success.
    assert!(
        plib::hermite_interpolate(
            0,
            2.0,
            2.0,
            0,
            0,
            &constraints,
            &constraints,
            &mut coefficients
        )
        .unwrap()
    );
}

#[test]
fn hermite_coefficients_reject_close_bounds_before_solving() {
    let mut matrix = rustycad_tkmath::math::Matrix::filled(1, 2, 1, 2, 91.0).unwrap();
    assert!(!plib::hermite_coefficients(1.0, 1.005, 0, 0, &mut matrix).unwrap());
    assert_eq!(matrix[(1, 1)], 91.0);
    assert!(plib::hermite_coefficients(-1.0, 1.0, 0, 0, &mut matrix).unwrap());
    assert_eq!([matrix[(1, 1)], matrix[(1, 2)]], [0.5, -0.5]);
}

#[test]
fn point_coefficients_convert_to_bezier_poles() {
    // P(t) = (t, t^2, 0): Bezier poles (0,0,0), (0.5,0,0), (1,1,0).
    let mut coefs: Array1<Point3> = Array1::with_bounds(1, 3).unwrap();
    coefs[2] = Point3::new(1.0, 0.0, 0.0);
    coefs[3] = Point3::new(0.0, 1.0, 0.0);
    let mut poles: Array1<Point3> = Array1::with_bounds(1, 3).unwrap();
    plib::coefficients_poles(&coefs, None, &mut poles, None).unwrap();
    assert_eq!(poles[2], Point3::new(0.5, 0.0, 0.0));
    assert_eq!(poles[3], Point3::new(1.0, 1.0, 0.0));
    // A longer pole array would read an uninitialized native temporary.
    let mut longer: Array1<Point3> = Array1::with_bounds(1, 4).unwrap();
    let error = plib::coefficients_poles(&coefs, None, &mut longer, None).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_DimensionError");
}

#[test]
fn trimming_checks_weight_count_and_reparameterizes() {
    // c(u) = u on [2, 4] becomes 2 + 2v.
    let mut coefficients = reals(1, &[0.0, 1.0]);
    plib::trimming_scalar(2.0, 4.0, &mut coefficients, None).unwrap();
    assert_eq!(coefficients.as_slice(), [2.0, 2.0]);
    let mut weights = reals(1, &[1.0]);
    let error = plib::trimming_scalar(2.0, 4.0, &mut coefficients, Some(&mut weights)).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_Failure");
}

#[test]
fn jacobi_parameters_write_work_degree_before_code_check() {
    let (mut points, mut degree) = (-7, -7);
    let error = plib::jacobi_parameters(Shape::C1, 10, 0, &mut points, &mut degree).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    assert_eq!((points, degree), (-7, 16));
    plib::jacobi_parameters(Shape::C1, 10, 2, &mut points, &mut degree).unwrap();
    assert_eq!((points, degree), (40, 19));
    assert!(plib::niv_constr(Shape::G1).is_err());
}

#[test]
fn jacobi_tables_are_the_native_literals() {
    let basis = JacobiPolynomial::new(10, Shape::C0).unwrap();
    let mut maxima = reals(0, &[0.0; 2]);
    basis.max_value(&mut maxima).unwrap();
    assert_eq!(maxima[0], 0.968245836551854221294816349945600);
    assert_eq!(maxima[1], 0.986013297183269340427888048593603);
    let mut values = reals(0, &[0.0; 9]);
    basis.d0(0.0, &mut values).unwrap();
    // J0 = TNorm[0] and odd functions vanish at 0.
    assert_eq!(values[0], 9.68245836551854255347e-01);
    assert_eq!(values[1], 0.0);
}

#[test]
fn average_error_stops_at_the_q_degree() {
    // WorkDegree 8 with C1 has Q degree 4, below the cut of 7: no term is summed.
    let basis = JacobiPolynomial::new(8, Shape::C1).unwrap();
    let coefficients: Vec<f64> = (1..=10).map(|i| 1.0 / f64::from(i + 1)).collect();
    assert_eq!(basis.average_error(1, &coefficients, 6).unwrap(), 0.0);
    assert!(basis.max_error(1, &coefficients, 6).unwrap() > 0.0);
}

#[test]
fn jacobi_rejects_invalid_counts_and_table_overruns() {
    let basis = JacobiPolynomial::new(10, Shape::C0).unwrap();
    let mut points = reals(0, &[0.0; 5]);
    let error = basis.points(9, &mut points).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    let mut maxima = reals(0, &[0.0; 58]);
    assert_eq!(
        basis.max_value(&mut maxima).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert!(JacobiPolynomial::new(33, Shape::C0).is_err());
}

#[test]
fn hermit_jacobi_starts_with_the_hermite_basis() {
    // At U = -1 the C0 basis is H0 = 1, H1 = 0 and W(-1) = 0 for every Jacobi term.
    let basis = HermitJacobi::new(4, Shape::C0).unwrap();
    let mut values = reals(0, &[9.0; 5]);
    basis.d0(-1.0, &mut values).unwrap();
    assert_eq!(values.as_slice(), [1.0, 0.0, 0.0, 0.0, 0.0]);
    let mut short = reals(0, &[0.0; 4]);
    assert!(basis.d0(0.0, &mut short).is_err());
}
