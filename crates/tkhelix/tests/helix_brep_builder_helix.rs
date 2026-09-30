//! 1:1 translation of src/ModelingAlgorithms/TKHelix/GTests/HelixBRep_BuilderHelix_Test.cxx.
#![allow(non_snake_case)]

mod support;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkhelix::helix_brep::HelixBRepBuilderHelix;
use rustycad_tkhelix::helix_geom::{
    HelixGeomBuilderApproxCurve, HelixGeomBuilderHelix, HelixGeomBuilderHelixCoil,
    HelixGeomBuilderHelixGen, HelixGeomHelixCurve, HelixGeomTools,
};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use std::f64::consts::PI;
use support::brep::{array, compute_wire_length, count_shapes, my_axis};
use support::{expect_double_eq, expect_near};

/// The fixture `TKHelixTest`: `myTolerance = 1.e-4` (and `myAxis`, see `my_axis`).
const MY_TOLERANCE: f64 = 1.0e-4;

#[test]
fn TKHelixTest__HelixGeomHelixCurve_Basic() {
    let a_helix = HelixGeomHelixCurve::new();

    // Test default parameters
    expect_double_eq(a_helix.first_parameter().unwrap(), 0.0);
    expect_double_eq(a_helix.last_parameter().unwrap(), 2.0 * PI);

    // Test point calculation
    let a_p0 = a_helix.value(0.0).unwrap();
    expect_double_eq(a_p0.x(), 1.0); // RStart = 1.0 by default
    expect_double_eq(a_p0.y(), 0.0);
    expect_double_eq(a_p0.z(), 0.0);

    let a_p1 = a_helix.value(PI).unwrap();
    expect_double_eq(a_p1.x(), -1.0);
    expect_near(a_p1.y(), 0.0, 1e-15);
    expect_double_eq(a_p1.z(), PI / (2.0 * PI)); // Pitch / (2*PI) * t
}

#[test]
fn TKHelixTest__HelixGeomHelixCurve_CustomParameters() {
    let a_helix = HelixGeomHelixCurve::new();
    a_helix.load(0.0, 4.0 * PI, 10.0, 5.0, 0.0, true).unwrap();

    expect_double_eq(a_helix.first_parameter().unwrap(), 0.0);
    expect_double_eq(a_helix.last_parameter().unwrap(), 4.0 * PI);

    // Test point at start
    let a_p0 = a_helix.value(0.0).unwrap();
    expect_double_eq(a_p0.x(), 5.0); // RStart = 5.0
    expect_double_eq(a_p0.y(), 0.0);
    expect_double_eq(a_p0.z(), 0.0);

    // Test point at end (2 full turns)
    let a_p1 = a_helix.value(4.0 * PI).unwrap();
    expect_double_eq(a_p1.x(), 5.0);
    expect_near(a_p1.y(), 0.0, 1e-14);
    expect_double_eq(a_p1.z(), 20.0); // 2 turns * pitch(10)
}

#[test]
fn TKHelixTest__HelixGeomHelixCurve_TaperedHelix() {
    let a_helix = HelixGeomHelixCurve::new();
    let a_taper_angle = 0.1; // Small taper angle
    a_helix
        .load(0.0, 2.0 * PI, 5.0, 2.0, a_taper_angle, true)
        .unwrap();

    // At start
    let a_p0 = a_helix.value(0.0).unwrap();
    expect_double_eq(a_p0.x(), 2.0);

    // At end - radius should be larger due to taper
    let a_p1 = a_helix.value(2.0 * PI).unwrap();
    let a_expected_radius = 2.0 + (5.0 / (2.0 * PI)) * a_taper_angle.tan() * (2.0 * PI);
    expect_double_eq(a_p1.x(), a_expected_radius);
}

#[test]
fn TKHelixTest__HelixGeomBuilderHelixCoil_Basic() {
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

/// `gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(Z), gp_Dir(X))`.
fn standard_position() -> Axis2 {
    Axis2::with_x_direction(Point3::new(0., 0., 0.), Direction3::Z, Direction3::X).unwrap()
}

#[test]
fn TKHelixTest__HelixGeomBuilderHelix_SingleCoil() {
    let mut a_builder = HelixGeomBuilderHelix::new();

    let a_position = standard_position();
    a_builder.set_position(a_position);
    a_builder.set_tolerance(MY_TOLERANCE);
    a_builder.set_curve_parameters(0.0, 2.0 * PI, 10.0, 5.0, 0.0, true);

    a_builder.perform().unwrap();

    assert_eq!(a_builder.error_status(), 0);

    let a_curves = a_builder.curves();
    assert_eq!(a_curves.length(), 1);
}

#[test]
fn TKHelixTest__HelixGeomBuilderHelix_MultipleCoils() {
    let mut a_builder = HelixGeomBuilderHelix::new();

    let a_position = standard_position();
    a_builder.set_position(a_position);
    a_builder.set_tolerance(MY_TOLERANCE);

    // 3 full turns
    a_builder.set_curve_parameters(0.0, 6.0 * PI, 10.0, 5.0, 0.0, true);

    a_builder.perform().unwrap();

    assert_eq!(a_builder.error_status(), 0);

    let a_curves = a_builder.curves();
    assert_eq!(a_curves.length(), 3); // Should split into 3 coils
}

#[test]
fn TKHelixTest__HelixBRepBuilder_PureCylindricalHelix() {
    let mut a_builder = HelixBRepBuilderHelix::new();

    // Single part helix with diameter 100, height 100, pitch 20
    let a_heights = array(&[100.0]);
    let a_pitches = array(&[20.0]);
    let a_is_pitches = array(&[true]); // Pitch mode

    a_builder
        .set_parameters_diam(&my_axis(), 100.0, &a_heights, &a_pitches, &a_is_pitches)
        .unwrap();
    a_builder.set_approx_parameters(MY_TOLERANCE, 8, Continuity::C1);

    a_builder.perform().unwrap();

    assert_eq!(a_builder.error_status(), 0);

    let a_result = a_builder.shape();
    assert!(!a_result.is_null());
    assert_eq!(a_result.shape_type().unwrap(), ShapeEnum::Wire);

    // Check number of edges
    let a_nb_edges = count_shapes(a_result, ShapeEnum::Edge);
    assert!(a_nb_edges > 0);

    // Compute wire length - should be approximately sqrt(circumference^2 + height^2) * turns
    let a_wire_length = compute_wire_length(a_result);
    let a_circumference = PI * 100.0;
    let a_turns = 100.0 / 20.0; // height/pitch
    let a_expected_length = a_turns * (a_circumference * a_circumference + 20.0 * 20.0).sqrt();

    expect_near(a_wire_length, a_expected_length, a_expected_length * 0.01); // 1% tolerance
}

#[test]
fn TKHelixTest__HelixBRepBuilder_SpiralHelix() {
    let mut a_builder = HelixBRepBuilderHelix::new();

    // Single part spiral with diameter changing from 100 to 20
    let a_heights = array(&[100.0]);
    let a_pitches = array(&[10.0]);
    let a_is_pitches = array(&[true]);

    a_builder
        .set_parameters_spiral(
            &my_axis(),
            100.0,
            20.0,
            &a_heights,
            &a_pitches,
            &a_is_pitches,
        )
        .unwrap();
    a_builder.set_approx_parameters(MY_TOLERANCE, 8, Continuity::C1);

    a_builder.perform().unwrap();

    assert_eq!(a_builder.error_status(), 0);

    let a_result = a_builder.shape();
    assert!(!a_result.is_null());
    assert_eq!(a_result.shape_type().unwrap(), ShapeEnum::Wire);

    let a_nb_edges = count_shapes(a_result, ShapeEnum::Edge);
    assert!(a_nb_edges > 0);
}

#[test]
fn TKHelixTest__HelixBRepBuilder_MultiPartHelix() {
    let mut a_builder = HelixBRepBuilderHelix::new();

    // 3-part helix with different diameters
    let a_diams = array(&[100.0, 80.0, 60.0, 40.0]);
    let a_heights = array(&[30.0, 40.0, 30.0]);
    let a_pitches = array(&[10.0, 15.0, 10.0]);
    let a_is_pitches = array(&[true, true, true]);

    a_builder
        .set_parameters(&my_axis(), &a_diams, &a_heights, &a_pitches, &a_is_pitches)
        .unwrap();
    a_builder.set_approx_parameters(MY_TOLERANCE, 8, Continuity::C1);

    a_builder.perform().unwrap();

    assert_eq!(a_builder.error_status(), 0);

    let a_result = a_builder.shape();
    assert!(!a_result.is_null());
    assert_eq!(a_result.shape_type().unwrap(), ShapeEnum::Wire);

    let a_nb_edges = count_shapes(a_result, ShapeEnum::Edge);
    assert!(a_nb_edges > 2); // Should have multiple edges for multiple parts
}

#[test]
fn TKHelixTest__HelixBRepBuilder_NumberOfTurnsInterface() {
    let mut a_builder = HelixBRepBuilderHelix::new();

    // Single part helix using number of turns
    let a_pitches = array(&[20.0]);
    let a_nb_turns = array(&[5.0]); // 5 turns

    a_builder
        .set_parameters_diam_turns(&my_axis(), 100.0, &a_pitches, &a_nb_turns)
        .unwrap();
    a_builder.set_approx_parameters(MY_TOLERANCE, 8, Continuity::C1);

    a_builder.perform().unwrap();

    assert_eq!(a_builder.error_status(), 0);

    let a_result = a_builder.shape();
    assert!(!a_result.is_null());

    // Wire length should correspond to 5 turns
    let a_wire_length = compute_wire_length(a_result);
    let a_circumference = PI * 100.0;
    let a_expected_length = 5.0 * (a_circumference * a_circumference + 20.0 * 20.0).sqrt();

    expect_near(a_wire_length, a_expected_length, a_expected_length * 0.01);
}

#[test]
fn TKHelixTest__HelixGeomTools_ApprHelix() {
    let mut a_bspline: Handle<GeomBSplineCurve> = Handle::default();
    let mut a_max_error: f64 = 0.0;

    let a_result = HelixGeomTools::appr_helix(
        0.0,          // T1
        2.0 * PI,     // T2
        10.0,         // Pitch
        5.0,          // Start radius
        0.0,          // Taper angle
        true,         // Clockwise
        MY_TOLERANCE, // Tolerance
        &mut a_bspline,
        &mut a_max_error,
    )
    .unwrap();

    assert_eq!(a_result, 0); // Success
    assert!(!a_bspline.is_null());
    assert!(a_max_error <= MY_TOLERANCE);

    // Test curve properties
    let a_bspline = a_bspline.get().unwrap();
    assert!(a_bspline.degree() > 0);
    assert!(a_bspline.nb_poles() > 0);
}

#[test]
fn TKHelixTest__HelixBRepBuilder_ErrorConditions() {
    let mut a_builder = HelixBRepBuilderHelix::new();

    // Test with invalid height (too small)
    let a_heights = array(&[1.0e-6]); // Very small height
    let a_pitches = array(&[10.0]);
    let a_is_pitches = array(&[true]);

    a_builder
        .set_parameters_diam(&my_axis(), 100.0, &a_heights, &a_pitches, &a_is_pitches)
        .unwrap();
    a_builder.set_approx_parameters(MY_TOLERANCE, 8, Continuity::C1);

    a_builder.perform().unwrap();

    assert_ne!(a_builder.error_status(), 0); // Should have error
}

#[test]
fn TKHelixTest__HelixBRepBuilder_ZeroPitch() {
    let mut a_builder = HelixBRepBuilderHelix::new();

    let a_heights = array(&[100.0]);
    let a_pitches = array(&[0.0]); // Zero pitch
    let a_is_pitches = array(&[true]);

    a_builder
        .set_parameters_diam(&my_axis(), 100.0, &a_heights, &a_pitches, &a_is_pitches)
        .unwrap();
    a_builder.set_approx_parameters(MY_TOLERANCE, 8, Continuity::C1);

    a_builder.perform().unwrap();

    assert_ne!(a_builder.error_status(), 0); // Should have error
}

#[test]
fn TKHelixTest__HelixBRepBuilder_ToleranceReached() {
    let mut a_builder = HelixBRepBuilderHelix::new();

    let a_heights = array(&[100.0]);
    let a_pitches = array(&[20.0]);
    let a_is_pitches = array(&[true]);

    let a_very_tight_tolerance = 1.0e-8;
    a_builder
        .set_parameters_diam(&my_axis(), 100.0, &a_heights, &a_pitches, &a_is_pitches)
        .unwrap();
    a_builder.set_approx_parameters(a_very_tight_tolerance, 8, Continuity::C1);

    a_builder.perform().unwrap();

    if a_builder.error_status() == 0 {
        let a_tolerance_reached = a_builder.tolerance_reached();
        assert!(a_tolerance_reached > 0.0);
    }
}
