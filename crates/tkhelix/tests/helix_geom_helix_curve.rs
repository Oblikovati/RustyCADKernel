//! 1:1 translation of src/ModelingAlgorithms/TKHelix/GTests/HelixGeom_HelixCurve_Test.cxx.
#![allow(non_snake_case)]

mod support;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkhelix::helix_geom::{
    HelixGeomBuilderApproxCurve, HelixGeomBuilderHelixCoil, HelixGeomBuilderHelixGen,
    HelixGeomHelixCurve, HelixGeomTools,
};
use rustycad_tkmath::geom_abs::Shape;
use std::f64::consts::PI;
use support::{expect_double_eq, expect_near, expect_throw};

/// The fixture `HelixGeomTest`: `myTolerance = 1.e-6`.
const MY_TOLERANCE: f64 = 1.0e-6;

#[test]
fn HelixGeomTest__HelixCurve_Derivatives() {
    let a_helix = HelixGeomHelixCurve::new();
    a_helix.load(0.0, 2.0 * PI, 5.0, 2.0, 0.0, true).unwrap();

    let a_param = PI / 2.0;

    // Test D1
    let (a_p, a_v1) = a_helix.d1(a_param).unwrap();
    expect_near(a_p.x(), 0.0, 1e-15);
    expect_near(a_p.y(), 2.0, 1e-15);
    assert!(a_v1.magnitude() > 0.0);

    // Test D2
    let (a_p, a_v1, a_v2) = a_helix.d2(a_param).unwrap();
    expect_near(a_p.x(), 0.0, 1e-15);
    expect_near(a_p.y(), 2.0, 1e-15);
    assert!(a_v2.magnitude() > 0.0);

    // Test DN
    let a_vn1 = a_helix.dn(a_param, 1).unwrap();
    let a_vn2 = a_helix.dn(a_param, 2).unwrap();

    expect_near(a_vn1.x(), a_v1.x(), 1e-15);
    expect_near(a_vn1.y(), a_v1.y(), 1e-15);
    expect_near(a_vn1.z(), a_v1.z(), 1e-15);

    expect_near(a_vn2.x(), a_v2.x(), 1e-15);
    expect_near(a_vn2.y(), a_v2.y(), 1e-15);
    expect_near(a_vn2.z(), a_v2.z(), 1e-15);
}

#[test]
fn HelixGeomTest__HelixCurve_ErrorConditions() {
    let a_helix = HelixGeomHelixCurve::new();

    // Test invalid parameter range (T1 >= T2)
    expect_throw(
        a_helix.load(2.0, 1.0, 5.0, 2.0, 0.0, true),
        "Standard_ConstructionError",
    );

    // Test negative pitch
    expect_throw(
        a_helix.load(0.0, 2.0 * PI, -1.0, 2.0, 0.0, true),
        "Standard_ConstructionError",
    );

    // Test negative radius
    expect_throw(
        a_helix.load(0.0, 2.0 * PI, 5.0, -1.0, 0.0, true),
        "Standard_ConstructionError",
    );

    // Test invalid taper angle
    expect_throw(
        a_helix.load(0.0, 2.0 * PI, 5.0, 2.0, PI / 2.0, true),
        "Standard_ConstructionError",
    );
}

#[test]
fn HelixGeomTest__HelixCurve_CounterClockwise() {
    let a_helix = HelixGeomHelixCurve::new();
    a_helix.load(0.0, 2.0 * PI, 5.0, 2.0, 0.0, false).unwrap(); // Counter-clockwise

    let a_p0 = a_helix.value(0.0).unwrap();
    let a_p1 = a_helix.value(PI / 2.0).unwrap();

    expect_near(a_p0.x(), 2.0, 1e-15);
    expect_near(a_p0.y(), 0.0, 1e-15);

    expect_near(a_p1.x(), 0.0, 1e-15);
    expect_near(a_p1.y(), -2.0, 1e-15); // Negative Y for counter-clockwise
}

#[test]
fn HelixGeomTest__HelixCurve_AdaptorInterface() {
    let a_helix = HelixGeomHelixCurve::new();
    a_helix.load(0.0, 4.0 * PI, 10.0, 3.0, 0.0, true).unwrap();

    // Test continuity
    assert_eq!(a_helix.continuity().unwrap(), Shape::CN);

    // Test intervals
    assert_eq!(a_helix.nb_intervals(Shape::C0).unwrap(), 1);
    assert_eq!(a_helix.nb_intervals(Shape::C1).unwrap(), 1);
    assert_eq!(a_helix.nb_intervals(Shape::C2).unwrap(), 1);

    // Test intervals array
    let mut an_intervals: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    a_helix.intervals(&mut an_intervals, Shape::C0).unwrap();
    expect_double_eq(an_intervals[1], 0.0);
    expect_double_eq(an_intervals[2], 4.0 * PI);

    // Test not implemented methods
    expect_throw(a_helix.resolution(1.0), "Standard_NotImplemented");
    expect_throw(a_helix.is_closed(), "Standard_NotImplemented");
    expect_throw(a_helix.is_periodic(), "Standard_NotImplemented");
    expect_throw(a_helix.period(), "Standard_DomainError");
}

#[test]
fn HelixGeomTest__BuilderHelixCoil_Approximation() {
    let mut a_builder = HelixGeomBuilderHelixCoil::new();

    a_builder.set_tolerance(MY_TOLERANCE);
    a_builder.set_approx_parameters(Shape::C2, 8, 100);
    a_builder.set_curve_parameters(0.0, 2.0 * PI, 20.0, 5.0, 0.05, true); // Reduced parameter range and taper angle

    a_builder.perform().unwrap();

    assert_eq!(a_builder.error_status(), 0);
    assert!(a_builder.tolerance_reached() <= 0.1); // Allow up to 0.1 tolerance for tapered helix approximation

    let a_curves = a_builder.curves();
    assert_eq!(a_curves.length(), 1);

    let a_curve: Handle<dyn GeomCurve> = a_curves[1].clone();
    assert!(!a_curve.is_null());

    // Verify that the curve is a B-spline
    let a_bspline: Handle<GeomBSplineCurve> = a_curve.cast();
    assert!(!a_bspline.is_null());

    // Check B-spline properties
    let a_bspline = a_bspline.get().unwrap();
    assert!(a_bspline.degree() <= 8);
    assert!(a_bspline.nb_poles() > 0);
}

#[test]
fn HelixGeomTest__Tools_ApprCurve3D() {
    // Create a helix curve adaptor
    let a_helix = HelixGeomHelixCurve::new();
    a_helix.load(0.0, 2.0 * PI, 10.0, 3.0, 0.0, true).unwrap();
    let a_h_adaptor = a_helix.to_adaptor();

    let mut a_bspline: Handle<GeomBSplineCurve> = Handle::default();
    let mut a_max_error: f64 = 0.0;

    let a_result = HelixGeomTools::appr_curve_3d(
        &a_h_adaptor,
        MY_TOLERANCE,
        Shape::C1,
        50, // Max segments
        6,  // Max degree
        &mut a_bspline,
        &mut a_max_error,
    )
    .unwrap();

    assert_eq!(a_result, 0);
    assert!(!a_bspline.is_null());
    assert!(a_max_error <= MY_TOLERANCE * 10.0);

    // Verify approximation quality by sampling points
    let a_bspline = a_bspline.get().unwrap();
    let a_nb_samples = 10;
    for i in 0..=a_nb_samples {
        let (first, last) = (
            a_helix.first_parameter().unwrap(),
            a_helix.last_parameter().unwrap(),
        );
        let a_param = first + f64::from(i) * (last - first) / f64::from(a_nb_samples);

        let a_orig_pnt = a_helix.value(a_param).unwrap();

        // Map parameter to B-spline parameter space
        let a_bspline_param = a_bspline.first_parameter()
            + f64::from(i) * (a_bspline.last_parameter() - a_bspline.first_parameter())
                / f64::from(a_nb_samples);

        let a_approx_pnt = a_bspline.d0(a_bspline_param).unwrap();

        let a_distance = a_orig_pnt.distance(&a_approx_pnt);
        assert!(a_distance <= a_max_error * 2.0); // Allow some margin
    }
}

#[test]
fn HelixGeomTest__Tools_DifferentContinuity() {
    let a_helix = HelixGeomHelixCurve::new();
    a_helix.load(0.0, 6.0 * PI, 15.0, 4.0, 0.05, true).unwrap();
    let a_h_adaptor = a_helix.to_adaptor();

    // Test C0 continuity
    let mut a_bspline_c0: Handle<GeomBSplineCurve> = Handle::default();
    let mut a_max_error_c0: f64 = 0.0;
    let a_result_c0 = HelixGeomTools::appr_curve_3d(
        &a_h_adaptor,
        MY_TOLERANCE,
        Shape::C0,
        30,
        4,
        &mut a_bspline_c0,
        &mut a_max_error_c0,
    )
    .unwrap();

    assert_eq!(a_result_c0, 0);
    assert!(!a_bspline_c0.is_null());

    // Test C2 continuity
    let mut a_bspline_c2: Handle<GeomBSplineCurve> = Handle::default();
    let mut a_max_error_c2: f64 = 0.0;
    let a_result_c2 = HelixGeomTools::appr_curve_3d(
        &a_h_adaptor,
        MY_TOLERANCE,
        Shape::C2,
        30,
        6,
        &mut a_bspline_c2,
        &mut a_max_error_c2,
    )
    .unwrap();

    assert_eq!(a_result_c2, 0);
    assert!(!a_bspline_c2.is_null());

    // C2 curve should generally have higher degree
    assert!(a_bspline_c2.get().unwrap().degree() >= a_bspline_c0.get().unwrap().degree());
}

#[test]
fn HelixGeomTest__BuilderHelixCoil_DefaultParameters() {
    let mut a_builder = HelixGeomBuilderHelixCoil::new();

    // Use default parameters
    a_builder.perform().unwrap();

    assert_eq!(a_builder.error_status(), 0);

    // Check default approximation parameters
    let (a_cont, a_max_degree, a_max_seg) = a_builder.approx_parameters();

    assert_eq!(a_cont, Shape::C2);
    assert_eq!(a_max_degree, 8);
    assert_eq!(a_max_seg, 150);

    assert_eq!(a_builder.tolerance(), 0.0001);
}

#[test]
fn HelixGeomTest__BuilderHelixCoil_ParameterSymmetry() {
    let mut a_builder = HelixGeomBuilderHelixCoil::new();

    // Set parameters
    let (a_t1, a_t2, a_pitch, a_r_start, a_taper_angle) = (0.5, 5.5, 12.5, 3.5, 0.15);
    let a_is_clockwise = false;

    a_builder.set_curve_parameters(
        a_t1,
        a_t2,
        a_pitch,
        a_r_start,
        a_taper_angle,
        a_is_clockwise,
    );

    // Get parameters back
    let out = a_builder.curve_parameters();

    expect_double_eq(a_t1, out.t1);
    expect_double_eq(a_t2, out.t2);
    expect_double_eq(a_pitch, out.pitch);
    expect_double_eq(a_r_start, out.r_start);
    expect_double_eq(a_taper_angle, out.taper_angle);
    assert_eq!(a_is_clockwise, out.is_cw);
}
