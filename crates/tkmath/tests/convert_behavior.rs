//! Regression and behaviour tests for the Convert port: preserved native quirks,
//! documented rejections of undefined native accesses, and the frame changes the
//! conversions rely on.
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkmath::convert::{
    CompBezierCurves2dToBSplineCurve2d, CompBezierCurvesToBSplineCurve, CompPolynomialToPoles,
    ConicToBSplineCurve, ElementarySurfaceToBSplineSurface as Surface, GridPolynomialToPoles,
    ParameterisationType as Param, PolynomialSpans, build_polynomial_cos_and_sin,
};
use rustycad_tkmath::gp::{
    Axis2d, Axis3, Circle2, Cylinder3, Direction2, Direction3, Hyperbola2, Point2, Point3, Sphere3,
    Transform2, Transform3,
};
use std::f64::consts::PI;

fn reals(lower: i32, values: &[f64]) -> Array1<'static, f64> {
    let mut array: Array1<'static, f64> =
        Array1::with_bounds(lower, lower + values.len() as i32 - 1).unwrap();
    array.as_mut_slice().copy_from_slice(values);
    array
}

fn points2(lower: i32, values: &[(f64, f64)]) -> Array1<'static, Point2> {
    let mut array: Array1<'static, Point2> =
        Array1::with_bounds(lower, lower + values.len() as i32 - 1).unwrap();
    for (point, (x, y)) in array.as_mut_slice().iter_mut().zip(values) {
        *point = Point2::new(*x, *y);
    }
    array
}

fn unit_circle() -> Circle2 {
    Circle2::from_x_axis(Axis2d::default(), 1.0, true).unwrap()
}

fn z_frame() -> Axis3 {
    Axis3::default()
}

#[test]
fn transformation_between_maps_frame_coordinates_into_the_target_frame() {
    let from = Axis2d::new(Point2::new(1.0, 2.0), Direction2::new(0.0, 1.0).unwrap());
    let moved = Point2::new(3.0, 0.0)
        .transformed(Transform2::transformation_between(from, Axis2d::default()));
    assert_eq!((moved.x(), moved.y()), (1.0, 5.0));
    let frame = Axis3::new(Point3::new(1.0, 2.0, 3.0), Direction3::X, Direction3::Y).unwrap();
    let moved = Point3::new(0.0, 0.0, 2.0)
        .transformed(Transform3::transformation_between(frame, z_frame()));
    assert_eq!((moved.x(), moved.y(), moved.z()), (3.0, 2.0, 3.0));
}

#[test]
fn periodic_tangent_circle_drops_the_closing_pole_and_uses_degree_multiplicities() {
    let circle = ConicToBSplineCurve::from_circle(&unit_circle(), Param::TgtThetaOver2).unwrap();
    assert_eq!((circle.nb_poles(), circle.nb_knots()), (6, 4));
    assert_eq!(circle.multiplicities().as_slice(), &[2, 2, 2, 2]);
    assert_eq!(circle.knots()[4], 2.0 * PI);
    assert_eq!(circle.weights()[2], (PI / 3.0).cos());
}

#[test]
fn periodic_rational_c1_circle_has_twelve_poles_on_quarter_knots() {
    let circle = ConicToBSplineCurve::from_circle(&unit_circle(), Param::RationalC1).unwrap();
    assert_eq!((circle.degree(), circle.nb_poles()), (4, 12));
    assert_eq!(circle.multiplicities().as_slice(), &[3, 3, 3, 3, 3]);
    assert_eq!(circle.knots()[2], PI * 0.5);
}

#[test]
fn only_two_parameterisations_have_a_periodic_form() {
    for param in [
        Param::QuasiAngular,
        Param::Polynomial,
        Param::TgtThetaOver2_3,
    ] {
        let error = ConicToBSplineCurve::build_cos_and_sin(param).unwrap_err();
        assert_eq!(error.exception_type(), "Standard_ConstructionError");
    }
    // The full circle then falls back to the arc [0, 2 PI].
    let circle = ConicToBSplineCurve::from_circle(&unit_circle(), Param::QuasiAngular).unwrap();
    assert!(!circle.is_periodic());
}

#[test]
fn span_count_limits_raise_construction_errors() {
    assert!(
        ConicToBSplineCurve::build_cos_and_sin_trimmed(Param::TgtThetaOver2_1, 0.0, 0.9999 * PI)
            .is_ok()
    );
    let error =
        ConicToBSplineCurve::build_cos_and_sin_trimmed(Param::TgtThetaOver2_1, 0.0, 0.99991 * PI)
            .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    let error =
        ConicToBSplineCurve::build_cos_and_sin_trimmed(Param::TgtThetaOver2_2, 0.0, 1.99991 * PI)
            .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
}

#[test]
fn negative_quasi_angular_arc_takes_the_small_angle_branch() {
    // alpha / 2 < 1e-7 holds for every negative opening, so b = -6/15 (native quirk).
    let reversed =
        ConicToBSplineCurve::build_cos_and_sin_trimmed(Param::QuasiAngular, 1.0, 0.5).unwrap();
    assert_eq!(reversed.degree, 6);
    assert!(reversed.cos_numerator.iter().all(|value| value.is_finite()));
}

#[test]
fn arc_openings_are_checked_against_two_pi_plus_pconfusion() {
    let circle = unit_circle();
    assert!(
        ConicToBSplineCurve::from_circle_arc(&circle, 0.0, 2.0 * PI + 1e-10, Param::TgtThetaOver2)
            .is_ok()
    );
    for (u1, u2) in [(1.0, 1.0), (1.0, 0.5), (0.0, 2.0 * PI + 1e-8)] {
        let error = ConicToBSplineCurve::from_circle_arc(&circle, u1, u2, Param::TgtThetaOver2)
            .unwrap_err();
        assert_eq!(error.exception_type(), "Standard_DomainError");
    }
}

#[test]
fn hyperbola_orders_its_bounds_and_rejects_an_empty_range() {
    let hyperbola = Hyperbola2::from_x_axis(Axis2d::default(), 3.0, 2.0, true).unwrap();
    let arc = ConicToBSplineCurve::from_hyperbola(&hyperbola, 1.0, -1.0).unwrap();
    assert_eq!(arc.knots().as_slice(), &[-1.0, 1.0]);
    assert_eq!(arc.weights()[2], 1.0_f64.cosh());
    let error = ConicToBSplineCurve::from_hyperbola(&hyperbola, 2.0, 2.0).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_DomainError");
}

#[test]
fn deprecated_accessors_check_one_based_indices() {
    let circle = ConicToBSplineCurve::from_circle(&unit_circle(), Param::TgtThetaOver2).unwrap();
    assert_eq!(circle.pole(1).unwrap(), circle.poles()[1]);
    assert_eq!(circle.multiplicity(4).unwrap(), 2);
    for error in [
        circle.pole(0).err(),
        circle.weight(7).err(),
        circle.knot(5).err(),
    ] {
        assert_eq!(error.unwrap().exception_type(), "Standard_OutOfRange");
    }
    let sphere = Surface::from_sphere(&Sphere3::new(z_frame(), 1.0).unwrap()).unwrap();
    assert_eq!(sphere.weight(2, 2).unwrap(), 0.5 * 2.0_f64.sqrt() / 2.0);
    assert_eq!(
        sphere.pole(7, 1).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        sphere.v_multiplicity(4).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn polynomial_arc_starts_on_the_exact_end_point() {
    let (mut cos, mut sin, mut den) = (
        reals(1, &[9.0; 8]),
        reals(1, &[9.0; 8]),
        reals(1, &[9.0; 8]),
    );
    build_polynomial_cos_and_sin(0.0, PI / 2.0, 8, &mut cos, &mut sin, &mut den).unwrap();
    assert_eq!((cos[1], sin[1]), (1.0, 0.0));
    assert!(den.iter().all(|weight| *weight == 1.0));
}

#[test]
fn sphere_lune_weights_its_meridian_middles_with_one_half() {
    // Native quirk: CosV = 0.5 although the two V spans open by PI / 4 each.
    let lune =
        Surface::from_sphere_band(&Sphere3::new(z_frame(), 2.0).unwrap(), 0.0, PI, true).unwrap();
    assert_eq!((lune.nb_u_poles(), lune.nb_v_poles()), (5, 5));
    assert_eq!(lune.weights()[(1, 2)], 0.5);
    assert_eq!(lune.weights()[(2, 1)], (PI / 4.0).cos());
}

#[test]
fn sphere_zone_opening_past_the_meridian_buffer_is_refused() {
    // The native ComputePoles writes a seventh meridian pole into a five-entry std::array.
    let sphere = Sphere3::new(z_frame(), 2.0).unwrap();
    let error = Surface::from_sphere_band(&sphere, 0.0, 2.0 * PI, false)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert!(Surface::from_sphere_band(&sphere, 0.0, 5.2, false).is_ok());
}

#[test]
fn reversed_sphere_latitudes_keep_the_native_span_arithmetic() {
    let sphere = Sphere3::new(z_frame(), 2.0).unwrap();
    let single = Surface::from_sphere_patch(&sphere, 0.0, 1.0, 1.5, -1.5).unwrap();
    assert_eq!((single.nb_v_poles(), single.nb_v_knots()), (1, 1));
    assert!(single.v_knots()[1].is_nan());
    assert_eq!(single.v_multiplicities()[1], 4);
    let error = Surface::from_sphere_patch(&sphere, 0.0, 1.0, 3.0, -3.0)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
}

#[test]
fn periodic_cylinder_keeps_six_of_the_seven_computed_poles() {
    let cylinder =
        Surface::from_cylinder(&Cylinder3::new(z_frame(), 1.0).unwrap(), 0.0, 1.0).unwrap();
    assert_eq!((cylinder.nb_u_poles(), cylinder.poles().nb_rows()), (6, 6));
    assert_eq!(cylinder.weights()[(2, 1)], 0.5);
    let error = Surface::from_cylinder(&Cylinder3::new(z_frame(), 1.0).unwrap(), 1.0, 1.0)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_DomainError");
}

#[test]
fn empty_bezier_sequence_keeps_the_previous_degree() {
    let mut conversion = CompBezierCurves2dToBSplineCurve2d::default();
    conversion.add_curve(&points2(1, &[(0.0, 0.0), (1.0, 0.0), (2.0, 1.0)]));
    conversion.perform().unwrap();
    let mut other = CompBezierCurves2dToBSplineCurve2d::default();
    other.perform().unwrap();
    assert_eq!((conversion.degree(), other.degree()), (2, 0));
    let mut longer: Array1<'static, Point2> = Array1::with_bounds(1, 4).unwrap();
    assert_eq!(
        conversion.poles(&mut longer).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn bezier_curve_not_based_at_one_rebases_the_work_array() {
    // NCollection_Array1::Assign copies the bounds, so pole Degree + 1 is out of range.
    let mut conversion = CompBezierCurves2dToBSplineCurve2d::default();
    conversion.add_curve(&points2(0, &[(0.0, 0.0), (1.0, 0.0)]));
    assert_eq!(
        conversion.perform().unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn only_the_3d_bezier_junction_guards_tiny_knot_ratios() {
    // Parallel chords of lengths 0.5 and 1e-20: lambda = 2e-20 is below 10 * Epsilon(1).
    let tiny = 1e-20;
    let mut flat = CompBezierCurves2dToBSplineCurve2d::default();
    flat.add_curve(&points2(1, &[(-1.0, 0.0), (-0.5, 0.0), (0.0, 0.0)]));
    flat.add_curve(&points2(1, &[(0.0, 0.0), (tiny, 0.0), (1.0, 1.0)]));
    flat.perform().unwrap();
    let mut space = CompBezierCurvesToBSplineCurve::default();
    let pole = |x: f64, y: f64| Point3::new(x, y, 0.0);
    let mut first: Array1<'static, Point3> = Array1::with_bounds(1, 3).unwrap();
    (first[1], first[2], first[3]) = (pole(-1.0, 0.0), pole(-0.5, 0.0), pole(0.0, 0.0));
    let mut second: Array1<'static, Point3> = Array1::with_bounds(1, 3).unwrap();
    (second[1], second[2], second[3]) = (pole(0.0, 0.0), pole(tiny, 0.0), pole(1.0, 1.0));
    space.add_curve(&first);
    space.add_curve(&second);
    space.perform().unwrap();
    let (mut knots, mut mults) = (
        reals(1, &[0.0; 3]),
        Array1::<i32>::with_bounds(1, 3).unwrap(),
    );
    flat.knots_and_mults(&mut knots, &mut mults).unwrap();
    assert_eq!((flat.nb_poles(), mults[2]), (4, 1));
    space.knots_and_mults(&mut knots, &mut mults).unwrap();
    assert_eq!((space.nb_poles(), mults[2]), (5, 2));
}

#[test]
fn comp_polynomial_without_tables_is_a_construction_error() {
    let error = CompPolynomialToPoles::from_uniform_continuity(1, 0, 1, 1, None)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    let counts: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    let mut counts = counts;
    counts.init(2);
    let (coeffs, trues) = (reals(1, &[0.0, 1.0, 1.0, -1.0]), reals(1, &[0.0, 0.5, 1.0]));
    let mut domains: Array2<'static, f64> = Array2::with_bounds(1, 2, 1, 2).unwrap();
    domains
        .as_mut_slice()
        .copy_from_slice(&[0.0, 1.0, 0.0, 1.0]);
    let spans = PolynomialSpans {
        num_coeff_per_curve: &counts,
        coefficients: &coeffs,
        polynomial_intervals: &domains,
        true_intervals: &trues,
    };
    let error = CompPolynomialToPoles::from_uniform_continuity(2, 2, 1, 1, Some(spans))
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    let conversion =
        CompPolynomialToPoles::from_uniform_continuity(2, 0, 1, 1, Some(spans)).unwrap();
    assert_eq!(conversion.multiplicities().unwrap().as_slice(), &[2, 1, 2]);
    assert_eq!(conversion.poles_copy().unwrap().nb_rows(), 3);
}

#[test]
fn grid_polynomial_checks_the_coefficient_layout() {
    let mut counts: Array1<'static, i32> = Array1::with_bounds(0, 1).unwrap();
    counts.init(2);
    let (coeffs, unit) = (reals(1, &[0.0; 12]), reals(1, &[0.0, 1.0]));
    let error = GridPolynomialToPoles::from_single_patch(1, 1, &counts, &coeffs, &unit, &unit)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_DomainError");
    let mut counts: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    counts.init(2);
    let shifted = reals(3, &[0.0, 1.0]);
    let conversion =
        GridPolynomialToPoles::from_single_patch(1, 1, &counts, &coeffs, &unit, &unit).unwrap();
    assert_eq!(conversion.u_multiplicities().unwrap().as_slice(), &[2, 2]);
    // BuildArray gives 1-based multiplicities to knots based at 3; KnotSequence then
    // reads them with the knot index, past the array (undefined natively, refused here).
    let error = GridPolynomialToPoles::from_single_patch(1, 1, &counts, &coeffs, &shifted, &unit)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
}
