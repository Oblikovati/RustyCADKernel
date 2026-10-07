//! Direct translation of Aspect_GridParams_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::precision;
use rustycad_tkernel::quantity::{Color, ColorSpace, NamedColor};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::aspect::{AspectGridDrawMode, AspectGridParams};
use std::f64::consts::PI;

/// `EXPECT_NEAR(expected, actual, tolerance)`.
fn expect_near(expected: f64, actual: f64, tolerance: f64) {
    assert!(
        (expected - actual).abs() <= tolerance,
        "{expected} vs {actual}"
    );
}

/// `EXPECT_DOUBLE_EQ`: equal within 4 units in the last place.
fn expect_double_eq(expected: f64, actual: f64) {
    let ulps = (expected.to_bits() as i64 - actual.to_bits() as i64).abs();
    assert!(expected == actual || ulps <= 4, "{expected} vs {actual}");
}

#[test]
fn Aspect_GridParamsTest__Defaults_AreReasonable() {
    let aParams = AspectGridParams::default();
    assert_eq!(
        NamedColor::GRAY70,
        aParams.color().name(),
        "default color is grey"
    );
    assert_eq!(
        NamedColor::GRAY50,
        aParams.accent_color().name(),
        "default accent color is darker grey"
    );
    expect_double_eq(0.01, aParams.scale());
    expect_double_eq(0.0, aParams.scale_y());
    expect_double_eq(0.01, aParams.effective_scale_y());
    expect_double_eq(0.0, aParams.accent_scale_x());
    expect_double_eq(0.0, aParams.accent_scale_y());
    expect_double_eq(0.0, aParams.accent_angular_scale());
    expect_double_eq(0.01, aParams.line_thickness());
    expect_double_eq(0.0, aParams.rotation_angle());
    assert_eq!(
        0,
        aParams.angular_divisions(),
        "default grid is rectangular"
    );
    assert!(!aParams.is_circular());
    assert_eq!(AspectGridDrawMode::Lines, aParams.draw_mode());
    assert!(!aParams.is_background());
    assert!(aParams.is_draw_axis());
    assert!(!aParams.is_view_adaptive());
    expect_near(0.0, aParams.origin().x(), precision::confusion());
    expect_near(0.0, aParams.origin().y(), precision::confusion());
    expect_near(0.0, aParams.origin().z(), precision::confusion());
    expect_double_eq(0.0, aParams.size_x());
    expect_double_eq(0.0, aParams.size_y());
    expect_double_eq(0.0, aParams.radius());
    expect_double_eq(0.0, aParams.z_offset());
    expect_double_eq(0.0, aParams.angle_start());
    expect_double_eq(0.0, aParams.angle_end());
    assert!(!aParams.is_bounded());
    assert!(!aParams.is_arc());
}

#[test]
fn Aspect_GridParamsTest__Bounds_RoundTrip() {
    let mut aParams = AspectGridParams::default();
    assert!(!aParams.is_bounded());
    aParams.set_size_x(10.0).unwrap();
    expect_double_eq(10.0, aParams.size_x());
    assert!(aParams.is_bounded());
    aParams.set_size_x(0.0).unwrap();
    assert!(
        !aParams.is_bounded(),
        "clearing SizeX returns to unbounded when other bounds are 0"
    );
    aParams.set_size_y(5.0).unwrap();
    assert!(aParams.is_bounded());
    aParams.set_size_y(0.0).unwrap();
    aParams.set_radius(3.0).unwrap();
    expect_double_eq(3.0, aParams.radius());
    assert!(aParams.is_bounded());
    aParams.set_radius(0.0).unwrap();
    assert!(!aParams.is_bounded());
    aParams.set_z_offset(-0.01);
    expect_double_eq(-0.01, aParams.z_offset());
    assert!(!aParams.is_bounded(), "ZOffset alone is not a bound");
}

#[test]
fn Aspect_GridParamsTest__ArcRange_RoundTrip() {
    let mut aParams = AspectGridParams::default();
    assert!(!aParams.is_arc());
    aParams.set_arc_range(0.0, PI);
    expect_double_eq(0.0, aParams.angle_start());
    expect_double_eq(PI, aParams.angle_end());
    assert!(aParams.is_arc());
    // Equal start and end reverts to full circle sentinel.
    aParams.set_arc_range(1.0, 1.0);
    assert!(!aParams.is_arc());
    // Wraparound: Start > End is allowed; renderer walks CCW through the wrap.
    aParams.set_arc_range(2.356, -2.356);
    assert!(aParams.is_arc());
}

#[test]
fn Aspect_GridParamsTest__DrawMode_RoundTrip() {
    let mut aParams = AspectGridParams::default();
    aParams.set_draw_mode(AspectGridDrawMode::Points);
    assert_eq!(AspectGridDrawMode::Points, aParams.draw_mode());
    aParams.set_draw_mode(AspectGridDrawMode::None);
    assert_eq!(AspectGridDrawMode::None, aParams.draw_mode());
    aParams.set_draw_mode(AspectGridDrawMode::Lines);
    assert_eq!(AspectGridDrawMode::Lines, aParams.draw_mode());
}

#[test]
fn Aspect_GridParamsTest__AngularDivisions_ToggleCircular() {
    let mut aParams = AspectGridParams::default();
    assert!(!aParams.is_circular());
    aParams.set_angular_divisions(8);
    assert_eq!(8, aParams.angular_divisions());
    assert!(aParams.is_circular());
    aParams.set_angular_divisions(0);
    assert!(
        !aParams.is_circular(),
        "zero divisions falls back to rectangular"
    );
}

#[test]
fn Aspect_GridParamsTest__EffectiveScaleY_FollowsScaleWhenZero() {
    let mut aParams = AspectGridParams::default();
    aParams.set_scale(0.25).unwrap();
    expect_double_eq(0.25, aParams.effective_scale_y());
    aParams.set_scale_y(0.5).unwrap();
    expect_double_eq(0.5, aParams.effective_scale_y());
    aParams.set_scale_y(0.0).unwrap();
    expect_double_eq(0.25, aParams.effective_scale_y());
}

#[test]
fn Aspect_GridParamsTest__RotationAngle_RoundTrip() {
    let mut aParams = AspectGridParams::default();
    aParams.set_rotation_angle(0.5);
    expect_double_eq(0.5, aParams.rotation_angle());
    aParams.set_rotation_angle(-1.25);
    expect_double_eq(-1.25, aParams.rotation_angle());
}

#[test]
fn Aspect_GridParamsTest__Setters_RoundTrip() {
    let mut aParams = AspectGridParams::default();
    let aBlue = Color::named(NamedColor::BLUE1);
    aParams.set_color(aBlue);
    assert_eq!(aBlue, aParams.color());
    let aRed = Color::named(NamedColor::RED);
    aParams.set_accent_color(aRed);
    assert_eq!(aRed, aParams.accent_color());
    aParams.set_origin(Point3::new(1.0, 2.0, 3.0));
    expect_near(1.0, aParams.origin().x(), precision::confusion());
    expect_near(2.0, aParams.origin().y(), precision::confusion());
    expect_near(3.0, aParams.origin().z(), precision::confusion());
    aParams.set_scale(0.5).unwrap();
    expect_double_eq(0.5, aParams.scale());
    aParams.set_scale_y(0.75).unwrap();
    expect_double_eq(0.75, aParams.scale_y());
    expect_double_eq(0.75, aParams.effective_scale_y());
    aParams.set_accent_scale_x(0.05);
    aParams.set_accent_scale_y(0.1);
    aParams.set_accent_angular_scale(2.5);
    expect_double_eq(0.05, aParams.accent_scale_x());
    expect_double_eq(0.1, aParams.accent_scale_y());
    expect_double_eq(2.5, aParams.accent_angular_scale());
    aParams.set_line_thickness(0.02).unwrap();
    expect_double_eq(0.02, aParams.line_thickness());
    aParams.set_rotation_angle(0.123);
    expect_double_eq(0.123, aParams.rotation_angle());
    aParams.set_angular_divisions(12);
    assert_eq!(12, aParams.angular_divisions());
    assert!(aParams.is_circular());
    aParams.set_is_background(true);
    assert!(aParams.is_background());
    aParams.set_is_draw_axis(false);
    assert!(!aParams.is_draw_axis());
    aParams.set_is_view_adaptive(true);
    assert!(aParams.is_view_adaptive());
}

/// The source of `Copy_PreservesFields`, set field by field as the C++ test does.
fn copy_source() -> AspectGridParams {
    let mut aSrc = AspectGridParams::default();
    aSrc.set_color(Color::new(0.1, 0.2, 0.3, ColorSpace::Rgb).unwrap());
    aSrc.set_origin(Point3::new(4.0, 5.0, 6.0));
    aSrc.set_accent_color(Color::new(0.7, 0.6, 0.5, ColorSpace::Rgb).unwrap());
    aSrc.set_scale(0.25).unwrap();
    aSrc.set_scale_y(0.125).unwrap();
    aSrc.set_accent_scale_x(0.025);
    aSrc.set_accent_scale_y(0.0125);
    aSrc.set_accent_angular_scale(3.0);
    aSrc.set_line_thickness(0.04).unwrap();
    aSrc.set_rotation_angle(-0.5);
    aSrc.set_angular_divisions(16);
    aSrc.set_is_background(true);
    aSrc.set_is_draw_axis(false);
    aSrc.set_is_view_adaptive(true);
    aSrc.set_size_x(7.0).unwrap();
    aSrc.set_size_y(3.0).unwrap();
    aSrc.set_radius(2.0).unwrap();
    aSrc.set_z_offset(-0.002);
    aSrc.set_arc_range(0.25, 2.0);
    aSrc
}

#[test]
fn Aspect_GridParamsTest__Copy_PreservesFields() {
    let aCopy = copy_source();
    let tol = precision::confusion();
    expect_near(0.1, aCopy.color().red(), tol);
    expect_near(0.2, aCopy.color().green(), tol);
    expect_near(0.3, aCopy.color().blue(), tol);
    expect_near(4.0, aCopy.origin().x(), tol);
    expect_near(5.0, aCopy.origin().y(), tol);
    expect_near(6.0, aCopy.origin().z(), tol);
    expect_near(0.7, aCopy.accent_color().red(), tol);
    expect_near(0.6, aCopy.accent_color().green(), tol);
    expect_near(0.5, aCopy.accent_color().blue(), tol);
    expect_double_eq(0.25, aCopy.scale());
    expect_double_eq(0.125, aCopy.scale_y());
    expect_double_eq(0.125, aCopy.effective_scale_y());
    expect_double_eq(0.025, aCopy.accent_scale_x());
    expect_double_eq(0.0125, aCopy.accent_scale_y());
    expect_double_eq(3.0, aCopy.accent_angular_scale());
    expect_double_eq(0.04, aCopy.line_thickness());
    expect_double_eq(-0.5, aCopy.rotation_angle());
    assert_eq!(16, aCopy.angular_divisions());
    assert!(aCopy.is_circular());
    assert!(aCopy.is_background());
    assert!(!aCopy.is_draw_axis());
    assert!(aCopy.is_view_adaptive());
    expect_double_eq(7.0, aCopy.size_x());
    expect_double_eq(3.0, aCopy.size_y());
    expect_double_eq(2.0, aCopy.radius());
    expect_double_eq(-0.002, aCopy.z_offset());
    expect_double_eq(0.25, aCopy.angle_start());
    expect_double_eq(2.0, aCopy.angle_end());
    assert!(aCopy.is_bounded());
    assert!(aCopy.is_arc());
}

#[test]
fn Aspect_GridParamsTest__Setters_RejectNegativeValues() {
    let mut aParams = AspectGridParams::default();
    let negative = |failure: rustycad_tkernel::standard::Failure| {
        assert_eq!(failure.exception_type(), "Standard_NegativeValue");
    };
    negative(aParams.set_scale(-1.0).unwrap_err());
    negative(aParams.set_scale_y(-1.0).unwrap_err());
    negative(aParams.set_line_thickness(-0.1).unwrap_err());
    negative(aParams.set_size_x(-1.0).unwrap_err());
    negative(aParams.set_size_y(-1.0).unwrap_err());
    negative(aParams.set_radius(-1.0).unwrap_err());
}
