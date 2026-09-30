//! Direct translation of IntCurveSurface_IntersectionPoint_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkgeomalgo::int_curve_surface::{IntersectionPoint, TransitionOnCurve};
use rustycad_tkmath::gp::Point3;

/// `EXPECT_DOUBLE_EQ`: equal within 4 units in the last place.
fn expect_double_eq(a: f64, b: f64) {
    let ulps = (a.to_bits() as i64 - b.to_bits() as i64).unsigned_abs();
    assert!(a == b || ulps <= 4, "{a} != {b}");
}

// Regression test for bug #63: default constructor must initialize transition enum.
#[test]
fn IntCurveSurface_IntersectionPoint__DefaultConstructor_TransitionInitialized() {
    let a_pt = IntersectionPoint::default();

    // Transition must be deterministic (initialized to IntCurveSurface_Tangent).
    assert_eq!(a_pt.transition(), TransitionOnCurve::Tangent);

    // Scalar fields must be zero-initialized.
    expect_double_eq(a_pt.u(), 0.0);
    expect_double_eq(a_pt.v(), 0.0);
    expect_double_eq(a_pt.w(), 0.0);
}

// Value constructor initializes all fields correctly.
#[test]
fn IntCurveSurface_IntersectionPoint__ValueConstructor_AllFieldsSet() {
    let a_p = Point3::new(1.0, 2.0, 3.0);
    let a_pt = IntersectionPoint::new(a_p, 0.5, 0.6, 0.7, TransitionOnCurve::In);

    assert!((a_pt.pnt().x() - 1.0).abs() <= 1e-15);
    assert!((a_pt.pnt().y() - 2.0).abs() <= 1e-15);
    assert!((a_pt.pnt().z() - 3.0).abs() <= 1e-15);
    expect_double_eq(a_pt.u(), 0.5);
    expect_double_eq(a_pt.v(), 0.6);
    expect_double_eq(a_pt.w(), 0.7);
    assert_eq!(a_pt.transition(), TransitionOnCurve::In);
}

// SetValues overwrites all fields including transition.
#[test]
fn IntCurveSurface_IntersectionPoint__SetValues_OverwritesTransition() {
    let mut a_pt = IntersectionPoint::default();
    assert_eq!(a_pt.transition(), TransitionOnCurve::Tangent);

    let a_p = Point3::new(5.0, 6.0, 7.0);
    a_pt.set_values(a_p, 1.0, 2.0, 3.0, TransitionOnCurve::Out);
    assert_eq!(a_pt.transition(), TransitionOnCurve::Out);
}
