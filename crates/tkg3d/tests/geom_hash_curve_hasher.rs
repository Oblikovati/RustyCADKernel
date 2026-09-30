//! Direct translation of GeomHash_CurveHasher_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomGeometry,
    GeomHyperbola, GeomLine, GeomOffsetCurve, GeomParabola, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_hash::GeomHashCurveHasher;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3, Vector3};

// The fixture member `GeomHash_CurveHasher myHasher;` with the default tolerances.
fn SetUp() -> GeomHashCurveHasher {
    GeomHashCurveHasher::default()
}

// The implicit `occ::handle<T>` to `occ::handle<Geom_Curve>` conversion at each call.
fn curve<T: ?Sized + RuntimeTyped>(handle: &Handle<T>) -> Handle<dyn GeomCurve> {
    handle.cast()
}

// `occ::down_cast<T>(theCurve->Copy())`.
fn copied<T: RuntimeTyped + GeomGeometry>(handle: &Handle<T>) -> Handle<T> {
    handle.copy().unwrap().cast()
}

fn pnt(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}
fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}
// `gp_Ax2(P, N)`: the frame with the automatic X direction.
fn ax2(location: Point3, normal: Direction3) -> Axis2 {
    Axis2::new(location, normal).unwrap()
}
fn standard_axis() -> Axis2 {
    ax2(pnt(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0))
}
fn line(location: Point3, direction: Direction3) -> Handle<GeomLine> {
    Handle::new(GeomLine::from_point(location, direction))
}
fn circle(frame: Axis2, radius: f64) -> Handle<GeomCircle> {
    Handle::new(GeomCircle::new(frame, radius).unwrap())
}

fn poles(points: &[(f64, f64, f64)]) -> Array1<'static, Point3> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, points.len() as i32).unwrap();
    for (i, &(x, y, z)) in points.iter().enumerate() {
        aPoles[i as i32 + 1] = pnt(x, y, z);
    }
    aPoles
}
fn reals(values: &[f64]) -> Array1<'static, f64> {
    let mut anArray: Array1<'static, f64> = Array1::with_bounds(1, values.len() as i32).unwrap();
    for (i, &value) in values.iter().enumerate() {
        anArray[i as i32 + 1] = value;
    }
    anArray
}
fn integers(values: &[i32]) -> Array1<'static, i32> {
    let mut anArray: Array1<'static, i32> = Array1::with_bounds(1, values.len() as i32).unwrap();
    for (i, &value) in values.iter().enumerate() {
        anArray[i as i32 + 1] = value;
    }
    anArray
}
// `new Geom_BSplineCurve(Poles, Knots, Mults, Degree)` (non-periodic).
fn bspline(
    aPoles: &Array1<'_, Point3>,
    aKnots: &Array1<'_, f64>,
    aMults: &Array1<'_, i32>,
    degree: i32,
) -> Handle<GeomBSplineCurve> {
    Handle::new(GeomBSplineCurve::new(aPoles, aKnots, aMults, degree, false).unwrap())
}

include!("geom_hash_curve_hasher/suites_0.rs");
include!("geom_hash_curve_hasher/suites_1.rs");
