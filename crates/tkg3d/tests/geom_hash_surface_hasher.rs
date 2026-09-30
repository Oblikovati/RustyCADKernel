//! Direct translation of GeomHash_SurfaceHasher_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomCircle, GeomConicalSurface, GeomCurve,
    GeomCylindricalSurface, GeomGeometry, GeomLine, GeomOffsetSurface, GeomPlane,
    GeomRectangularTrimmedSurface, GeomSphericalSurface, GeomSurface, GeomSurfaceOfLinearExtrusion,
    GeomSurfaceOfRevolution, GeomToroidalSurface,
};
use rustycad_tkg3d::geom_hash::GeomHashSurfaceHasher;
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Direction3, Point3, Vector3};
use std::f64::consts::PI;

// The fixture member `GeomHash_SurfaceHasher myHasher;` with the default tolerances.
fn SetUp() -> GeomHashSurfaceHasher {
    GeomHashSurfaceHasher::default()
}

// The implicit `occ::handle<T>` to `occ::handle<Geom_Surface>` conversion at each call.
fn surface<T: ?Sized + RuntimeTyped>(handle: &Handle<T>) -> Handle<dyn GeomSurface> {
    handle.cast()
}
// The implicit conversion to `occ::handle<Geom_Curve>`.
fn curve<T: ?Sized + RuntimeTyped>(handle: &Handle<T>) -> Handle<dyn GeomCurve> {
    handle.cast()
}

// `occ::down_cast<T>(theSurface->Copy())`.
fn copied<T: RuntimeTyped + GeomGeometry>(handle: &Handle<T>) -> Handle<T> {
    handle.copy().unwrap().cast()
}

fn pnt(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}
fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}
// `gp_Ax3(P, N)`: the frame with the automatic X direction.
fn ax3(location: Point3, normal: Direction3) -> Axis3 {
    Axis3::from_normal(location, normal).unwrap()
}
fn standard_axis() -> Axis3 {
    ax3(pnt(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0))
}
fn plane(frame: Axis3) -> Handle<GeomPlane> {
    Handle::new(GeomPlane::new(frame))
}
fn line(location: Point3, direction: Direction3) -> Handle<GeomLine> {
    Handle::new(GeomLine::from_point(location, direction))
}

// A 1-based `NCollection_Array2` filled row by row.
fn net<T: Copy + Default>(rows: &[&[T]]) -> Array2<'static, T> {
    let (nu, nv) = (rows.len() as i32, rows[0].len() as i32);
    let mut anArray: Array2<'static, T> = Array2::with_bounds(1, nu, 1, nv).unwrap();
    for (i, row) in rows.iter().enumerate() {
        for (j, &value) in row.iter().enumerate() {
            anArray[(i as i32 + 1, j as i32 + 1)] = value;
        }
    }
    anArray
}
// The 2x2 pole net most tests use.
fn square_poles(corner: f64) -> Array2<'static, Point3> {
    net(&[
        &[pnt(0.0, 0.0, 0.0), pnt(1.0, 0.0, 0.0)],
        &[pnt(0.0, 1.0, 0.0), pnt(1.0, 1.0, corner)],
    ])
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

include!("geom_hash_surface_hasher/suites_0.rs");
include!("geom_hash_surface_hasher/suites_1.rs");
