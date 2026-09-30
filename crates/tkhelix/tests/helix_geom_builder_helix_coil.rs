//! 1:1 translation of src/ModelingAlgorithms/TKHelix/GTests/HelixGeom_BuilderHelixCoil_Test.cxx.
#![allow(non_snake_case)]

mod support;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkhelix::helix_geom::{
    HelixGeomBuilderApproxCurve, HelixGeomBuilderHelixCoil, HelixGeomBuilderHelixGen,
};
use rustycad_tkmath::geom_abs::Shape;
use std::f64::consts::PI;
use support::{expect_double_eq, expect_near};

/// The fixture `HelixGeom_BuilderHelixCoil_Test`: `myTolerance = 1.e-4`.
const MY_TOLERANCE: f64 = 1.0e-4;

#[test]
fn HelixGeom_BuilderHelixCoil_Test__BasicConstruction() {
    let mut a_builder = HelixGeomBuilderHelixCoil::new();

    a_builder.set_tolerance(MY_TOLERANCE);
    a_builder.set_curve_parameters(0.0, 2.0 * PI, 5.0, 2.0, 0.0, true);

    a_builder.perform().unwrap();

    assert_eq!(a_builder.error_status(), 0);

    let a_curves = a_builder.curves();
    assert_eq!(a_curves.length(), 1);

    let a_curve: Handle<dyn GeomCurve> = a_curves[1].clone();
    assert!(!a_curve.is_null());

    // Test curve endpoints
    let a_curve = a_curve.get().unwrap();
    let a_p1 = a_curve.d0(a_curve.first_parameter()).unwrap();
    let a_p2 = a_curve.d0(a_curve.last_parameter()).unwrap();

    expect_near(a_p1.x(), 2.0, MY_TOLERANCE);
    expect_near(a_p1.y(), 0.0, MY_TOLERANCE);
    expect_near(a_p1.z(), 0.0, MY_TOLERANCE);

    expect_near(a_p2.x(), 2.0, MY_TOLERANCE);
    expect_near(a_p2.y(), 0.0, MY_TOLERANCE);
    expect_near(a_p2.z(), 5.0, MY_TOLERANCE);
}

#[test]
fn HelixGeom_BuilderHelixCoil_Test__DefaultParameters() {
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
fn HelixGeom_BuilderHelixCoil_Test__ParameterSymmetry() {
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

#[test]
fn HelixGeom_BuilderHelixCoil_Test__TaperedHelix() {
    let mut a_builder = HelixGeomBuilderHelixCoil::new();

    a_builder.set_tolerance(MY_TOLERANCE);
    a_builder.set_approx_parameters(Shape::C2, 8, 100);
    a_builder.set_curve_parameters(0.0, 4.0 * PI, 20.0, 5.0, 0.1, true);

    a_builder.perform().unwrap();

    assert_eq!(a_builder.error_status(), 0);
    assert!(a_builder.tolerance_reached() <= MY_TOLERANCE * 10.0); // Allow some tolerance margin

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
