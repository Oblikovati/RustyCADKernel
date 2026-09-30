//! 1:1 translation of src/ModelingAlgorithms/TKHelix/GTests/HelixGeom_BuilderHelix_Test.cxx.
#![allow(non_snake_case)]

mod support;
use rustycad_tkhelix::helix_geom::{
    HelixGeomBuilderApproxCurve, HelixGeomBuilderHelix, HelixGeomBuilderHelixGen,
};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use std::f64::consts::PI;
use support::expect_double_eq;

/// The fixture `HelixGeom_BuilderHelix_Test`: `myTolerance = 1.e-4`.
const MY_TOLERANCE: f64 = 1.0e-4;

/// `gp_Ax2(P, gp_Dir(N), gp_Dir(X))`.
fn frame(location: Point3, normal: Direction3, x: Direction3) -> Axis2 {
    Axis2::with_x_direction(location, normal, x).unwrap()
}

#[test]
fn HelixGeom_BuilderHelix_Test__SingleCoil() {
    let mut a_builder = HelixGeomBuilderHelix::new();

    let a_position = frame(Point3::new(0., 0., 0.), Direction3::Z, Direction3::X);
    a_builder.set_position(a_position);
    a_builder.set_tolerance(MY_TOLERANCE);
    a_builder.set_curve_parameters(0.0, 2.0 * PI, 10.0, 5.0, 0.0, true);

    a_builder.perform().unwrap();

    assert_eq!(a_builder.error_status(), 0);

    let a_curves = a_builder.curves();
    assert_eq!(a_curves.length(), 1);
}

#[test]
fn HelixGeom_BuilderHelix_Test__MultipleCoils() {
    let mut a_builder = HelixGeomBuilderHelix::new();

    let a_position = frame(Point3::new(0., 0., 0.), Direction3::Z, Direction3::X);
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
fn HelixGeom_BuilderHelix_Test__PositionGetterSetter() {
    let mut a_builder = HelixGeomBuilderHelix::new();

    let a_test_position = frame(Point3::new(10., 20., 30.), Direction3::X, Direction3::Y);
    a_builder.set_position(a_test_position);

    let a_retrieved_position = a_builder.position();

    assert!(
        a_retrieved_position
            .location()
            .is_equal(&a_test_position.location(), 1e-15)
    );
    assert!(
        a_retrieved_position
            .direction()
            .is_equal(&a_test_position.direction(), 1e-15)
    );
    assert!(
        a_retrieved_position
            .x_direction()
            .is_equal(&a_test_position.x_direction(), 1e-15)
    );
}

#[test]
fn HelixGeom_BuilderHelix_Test__ParameterManagement() {
    let mut a_builder = HelixGeomBuilderHelix::new();

    // Set curve parameters
    let (a_t1, a_t2, a_pitch, a_r_start, a_taper_angle) = (1.0, 7.0, 15.0, 4.0, 0.2);
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
