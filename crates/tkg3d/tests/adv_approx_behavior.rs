//! Behavior tests of the AdvApprox port (no upstream GoogleTest suite exists);
//! the bit-exact comparison with OCCT is tools/verify_tkg3d_numeric.py.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkg3d::adv_approx::{
    AdvApproxApproxAFunction, AdvApproxApproximationInput, AdvApproxApproximationOutput,
    AdvApproxCutting, AdvApproxDichoCutting, AdvApproxEvaluatorFunction, AdvApproxPrefAndRec,
    AdvApproxSimpleApprox, approximation,
};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::plib::{self, JacobiPolynomial};

fn table(values: &[f64]) -> Array1<'static, f64> {
    let mut array = Array1::with_bounds(1, values.len() as i32).unwrap();
    array.as_mut_slice().copy_from_slice(values);
    array
}

// A helix (cos t, sin t, t / 4) and its derivatives.
fn helix(_: i32, _: [f64; 2], t: f64, d: i32, r: &mut [f64]) -> i32 {
    let (c, s) = (t.cos(), t.sin());
    let values = match d {
        0 => [c, s, 0.25 * t],
        1 => [-s, c, 0.25],
        _ => [-c, -s, 0.0],
    };
    r[..3].copy_from_slice(&values);
    0
}

// A cubic polynomial t^3 - t in every coordinate.
fn cubic(dimension: i32, _: [f64; 2], t: f64, d: i32, r: &mut [f64]) -> i32 {
    let value = match d {
        0 => t * t * t - t,
        1 => 3.0 * t * t - 1.0,
        _ => 6.0 * t,
    };
    r[..dimension as usize].fill(value);
    0
}

fn approx_helix(max_seg: i32) -> AdvApproxApproxAFunction {
    let tol = table(&[1e-6]);
    AdvApproxApproxAFunction::new(
        0,
        0,
        1,
        None,
        None,
        Some(&tol),
        0.0,
        6.0,
        Shape::C1,
        14,
        max_seg,
        &mut helix,
    )
    .unwrap()
}

#[test]
fn dicho_cutting_halves_long_intervals_only() {
    let tool = AdvApproxDichoCutting::new();
    assert_eq!(tool.value(0.0, 1.0), (true, 0.5));
    let (accepted, cut) = tool.value(0.0, 1e-8);
    assert!(!accepted);
    assert_eq!(cut, 0.5e-8);
}

#[test]
fn pref_and_rec_prefers_then_recommends_then_halves() {
    assert_eq!(
        AdvApproxPrefAndRec::new(&table(&[0.5]), &table(&[0.5]), 1.0)
            .unwrap_err()
            .exception_type(),
        "Standard_DomainError"
    );
    let tool = AdvApproxPrefAndRec::new(&table(&[0.2]), &table(&[0.45]), 5.0).unwrap();
    assert_eq!(tool.value(0.0, 1.0), (true, 0.45));
    // 0.45 is outside the weighted band of [0.4, 2.0]: the recommended 0.2 is outside too.
    assert_eq!(tool.value(0.4, 2.0), (true, 1.2));
    let tool = AdvApproxPrefAndRec::new(&table(&[0.3]), &table(&[5.0]), 5.0).unwrap();
    assert_eq!(tool.value(0.0, 1.0), (true, 0.3));
}

#[test]
fn closures_are_evaluators() {
    let mut f = |_: i32, _: [f64; 2], t: f64, _: i32, r: &mut [f64]| {
        r[0] = 2.0 * t;
        0
    };
    let mut values = [0.0];
    assert_eq!(f.evaluate(1, [0.0, 1.0], 0.25, 0, &mut values), 0);
    assert_eq!(values[0], 0.5);
}

#[test]
fn simple_approx_reproduces_a_cubic() {
    let (mut ng, mut wd) = (0, 0);
    plib::jacobi_parameters(Shape::C1, 13, 1, &mut ng, &mut wd).unwrap();
    let jacobi = JacobiPolynomial::new(wd, Shape::C1).unwrap();
    let mut f = cubic;
    let mut approx = AdvApproxSimpleApprox::new(2, 2, Shape::C1, wd, ng, &jacobi, &mut f).unwrap();
    let dims: Array1<'static, i32> = {
        let mut dims = Array1::with_bounds(1, 2).unwrap();
        dims.as_mut_slice().copy_from_slice(&[1, 1]);
        dims
    };
    assert_eq!(
        approx.max_error(1).unwrap_err().exception_type(),
        "Standard_NullObject"
    );
    approx
        .perform(&dims, &table(&[1e-9, 1e-9]), 0.0, 2.0, 13)
        .unwrap();
    assert!(approx.is_done());
    assert!(approx.degree() <= 3);
    assert!(approx.max_error(1).unwrap() < 1e-12);
    assert!(approx.average_error(2).unwrap() < 1e-12);
    assert_eq!(
        approx.max_error(3).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(approx.coefficients().len(), ((wd + 1) * 2) as usize);
    // f(0) = 0; f'(0) scaled by (Last - First) / 2 = -1.
    assert_eq!(*approx.first_constr().value(1, 0).unwrap(), 0.0);
    assert_eq!(*approx.first_constr().value(1, 1).unwrap(), -1.0);
    assert_eq!(*approx.last_constr().value(2, 0).unwrap(), 6.0);
    assert_eq!(approx.som_tab().len(), approx.dif_tab().len());
    let mut text = Vec::new();
    approx.dump(&mut text).unwrap();
    assert!(
        String::from_utf8(text)
            .unwrap()
            .starts_with("Dump of SimpleApprox \nError   ")
    );
}

#[test]
fn simple_approx_refuses_c3() {
    let jacobi = JacobiPolynomial::new(20, Shape::C1).unwrap();
    let mut f = cubic;
    let refused = AdvApproxSimpleApprox::new(1, 1, Shape::C3, 20, 25, &jacobi, &mut f);
    assert_eq!(
        refused.err().unwrap().exception_type(),
        "Standard_ConstructionError"
    );
}

#[test]
fn approx_a_function_builds_a_helix_bspline() {
    let approx = approx_helix(100);
    assert!(approx.is_done() && approx.has_result());
    assert_eq!(approx.num_sub_spaces(3).unwrap(), 1);
    assert_eq!(approx.num_sub_spaces(1).unwrap(), 0);
    assert_eq!(
        approx.num_sub_spaces(4).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    let poles = approx.poles().unwrap();
    assert_eq!(approx.nb_poles().unwrap(), poles.nb_rows() as i32);
    assert!(approx.poles_1d().is_none() && approx.poles_2d().is_none());
    let knots = approx.knots().unwrap();
    assert_eq!(approx.nb_knots().unwrap(), knots.len() as i32);
    assert_eq!((knots[1], knots[knots.upper()]), (0.0, 6.0));
    let mults = approx.multiplicities().unwrap();
    assert_eq!(mults[1], approx.degree() + 1);
    let mut column: Array1<'static, Point3> =
        Array1::with_bounds(1, approx.nb_poles().unwrap()).unwrap();
    approx.poles_of(1, &mut column).unwrap();
    assert!(column[1].distance(&Point3::new(1.0, 0.0, 0.0)) < 1e-12);
    assert!(approx.max_error(3, 1).unwrap() <= 1e-6);
    assert!(approx.average_error(3, 1).unwrap() <= approx.max_error(3, 1).unwrap());
    assert!(approx.max_errors(1).unwrap().is_none());
    assert_eq!(approx.max_errors(3).unwrap().unwrap().len(), 1);
    assert_eq!(approx.average_errors(3).unwrap().unwrap().len(), 1);
    assert_eq!(
        approx.max_errors(0).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    let mut text = Vec::new();
    approx.dump(&mut text).unwrap();
    assert!(
        String::from_utf8(text)
            .unwrap()
            .starts_with("Dump of ApproxAFunction\nError(s) 3d = \n   ")
    );
}

#[test]
fn approx_a_function_limited_segments_still_has_a_result() {
    // Five turns of the helix in one piece of degree 14 cannot reach 1e-6.
    let tol = table(&[1e-6]);
    let approx = AdvApproxApproxAFunction::new(
        0,
        0,
        1,
        None,
        None,
        Some(&tol),
        0.0,
        30.0,
        Shape::C1,
        14,
        1,
        &mut helix,
    )
    .unwrap();
    assert!(approx.has_result());
    assert_eq!(approx.nb_knots().unwrap(), 2);
    assert!(approx.max_error(3, 1).unwrap() > 1e-6);
}

#[test]
fn approx_a_function_1d_and_2d_spaces_with_cutting() {
    let tol = table(&[1e-7, 1e-7]);
    let cut = AdvApproxPrefAndRec::new(&table(&[0.5]), &table(&[1.0]), 5.0).unwrap();
    let mut f = cubic;
    let approx = AdvApproxApproxAFunction::with_cutting(
        1,
        1,
        0,
        Some(&tol),
        Some(&tol),
        None,
        -1.0,
        2.0,
        Shape::C2,
        6,
        10,
        &mut f,
        &cut,
    )
    .unwrap();
    assert!(approx.is_done());
    let n = approx.nb_poles().unwrap();
    let mut values: Array1<'static, f64> = Array1::with_bounds(1, n).unwrap();
    approx.poles_1d_of(1, &mut values).unwrap();
    let mut points: Array1<'static, Point2> = Array1::with_bounds(1, n).unwrap();
    approx.poles_2d_of(1, &mut points).unwrap();
    assert_eq!(values[1], 0.0);
    assert_eq!(points[1], Point2::new(0.0, 0.0));
    assert!(approx.poles().is_none());
    let mut missing: Array1<'static, Point3> = Array1::with_bounds(1, n).unwrap();
    assert_eq!(
        approx
            .poles_of(1, &mut missing)
            .unwrap_err()
            .exception_type(),
        "Standard_NullObject"
    );
}

#[test]
fn approx_a_function_rejects_bad_input() {
    let tol = table(&[1e-7]);
    let mut f = cubic;
    let bad_shape = AdvApproxApproxAFunction::new(
        1,
        0,
        0,
        Some(&tol),
        None,
        None,
        0.0,
        1.0,
        Shape::G1,
        14,
        10,
        &mut f,
    );
    assert_eq!(
        bad_shape.err().unwrap().exception_type(),
        "Standard_ConstructionError"
    );
    let reversed = AdvApproxApproxAFunction::new(
        1,
        0,
        0,
        Some(&tol),
        None,
        None,
        1.0,
        0.0,
        Shape::C1,
        14,
        10,
        &mut f,
    );
    assert_eq!(
        reversed.err().unwrap().exception_type(),
        "Standard_ConstructionError"
    );
    let null_table = AdvApproxApproxAFunction::new(
        0,
        1,
        0,
        Some(&tol),
        None,
        None,
        0.0,
        1.0,
        Shape::C1,
        14,
        10,
        &mut f,
    );
    assert_eq!(
        null_table.err().unwrap().exception_type(),
        "Standard_NullObject"
    );
}

#[test]
fn approx_a_function_without_result() {
    let tol = table(&[1e-7]);
    let mut f = cubic;
    let empty = AdvApproxApproxAFunction::new(
        1,
        0,
        0,
        Some(&tol),
        None,
        None,
        1.0,
        1.0,
        Shape::C1,
        14,
        10,
        &mut f,
    )
    .unwrap();
    assert!(!empty.is_done() && !empty.has_result());
    assert_eq!(empty.nb_poles().unwrap(), 0);
    assert_eq!(empty.degree(), 0);
    assert!(empty.knots().is_none() && empty.multiplicities().is_none());
    assert_eq!(
        empty.nb_knots().unwrap_err().exception_type(),
        "Standard_NullObject"
    );
    let mut failing = |_: i32, _: [f64; 2], _: f64, _: i32, _: &mut [f64]| 3;
    let failed = AdvApproxApproxAFunction::new(
        1,
        0,
        0,
        Some(&tol),
        None,
        None,
        0.0,
        1.0,
        Shape::C1,
        14,
        10,
        &mut failing,
    )
    .unwrap();
    assert!(!failed.has_result());
    assert_eq!(
        failed.max_error(1, 1).unwrap_err().exception_type(),
        "Standard_NullObject"
    );
    assert_eq!(
        failed.average_error(1, 1).unwrap_err().exception_type(),
        "Standard_NullObject"
    );
}

#[test]
fn static_approximation_fills_the_tables() {
    let mut dims: Array1<'static, i32> = Array1::with_bounds(1, 1).unwrap();
    dims[1] = 3;
    let tol = table(&[1e-6]);
    let input = AdvApproxApproximationInput {
        total_dimension: 3,
        total_num_ss: 1,
        local_dimension: &dims,
        first: 0.0,
        last: 6.0,
        continuity_order: 1,
        num_max_coeffs: 10,
        max_segments: 20,
        tolerances: &tol,
        code_precis: 1,
    };
    let mut ncoeff: Array1<'static, i32> = Array1::with_bounds(1, 20).unwrap();
    let mut coef: Array1<'static, f64> = Array1::with_bounds(1, 600).unwrap();
    let mut ints: Array1<'static, f64> = Array1::with_bounds(1, 21).unwrap();
    let mut emax: Array1<'static, f64> = Array1::with_bounds(1, 20).unwrap();
    let mut eavg: Array1<'static, f64> = Array1::with_bounds(1, 20).unwrap();
    let mut output = AdvApproxApproximationOutput {
        num_curves: 0,
        num_coeff_per_curve: &mut ncoeff,
        coefficients: &mut coef,
        intervals: &mut ints,
        error_max: &mut emax,
        average_error: &mut eavg,
        error_code: 0,
    };
    let mut f = helix;
    approximation(&input, &mut f, &AdvApproxDichoCutting::new(), &mut output).unwrap();
    assert_eq!(output.error_code, 0);
    let pieces = output.num_curves;
    assert!(pieces > 1);
    assert_eq!(ints[1], 0.0);
    assert_eq!(ints[pieces + 1], 6.0);
    assert!((1..=pieces).all(|i| emax[i] <= 1e-6 && ncoeff[i] >= 4));
    let bad = AdvApproxApproximationInput {
        total_dimension: 2,
        ..input
    };
    let mut output = AdvApproxApproximationOutput {
        num_curves: 0,
        num_coeff_per_curve: &mut ncoeff,
        coefficients: &mut coef,
        intervals: &mut ints,
        error_max: &mut emax,
        average_error: &mut eavg,
        error_code: 0,
    };
    AdvApproxApproxAFunction::approximation(
        &bad,
        &mut f,
        &AdvApproxDichoCutting::new(),
        &mut output,
    )
    .unwrap();
    assert_eq!(output.error_code, 1);
}
