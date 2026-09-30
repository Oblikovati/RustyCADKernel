//! Direct translation of GC_MakeConicalSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkgeombase::gc::{ErrorType, MakeConicalSurface, Root};
use rustycad_tkmath::gp::{self, Axis2, Direction3, Point3};
use std::f64::consts::PI;

fn z_axis() -> Axis2 {
    Axis2::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap()
}

#[test]
fn GC_MakeConicalSurfaceTest__FromAxis_AngleBelowResolution_BadAngle() {
    let aAxis = z_axis();

    let aMaker = MakeConicalSurface::new(aAxis, 0.5 * gp::resolution(), 2.0).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::BadAngle);
}

#[test]
fn GC_MakeConicalSurfaceTest__FromAxis_ValidAngle_Done() {
    let aAxis = z_axis();

    let aMaker = MakeConicalSurface::new(aAxis, PI / 6.0, 2.0).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);
}
