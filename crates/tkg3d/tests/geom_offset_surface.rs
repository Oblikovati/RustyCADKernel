//! Direct translation of Geom_OffsetSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{EXPECT_DOUBLE_EQ, EXPECT_NEAR};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCylindricalSurface, GeomElementarySurface, GeomGeometry,
    GeomOffsetSurface, GeomPlane, GeomRectangularTrimmedSurface, GeomSphericalSurface, GeomSurface,
    GeomToroidalSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3, Vector3};
use std::f64::consts::PI;

// gp_Ax3(P, gp_Dir(0, 0, 1)).
fn z_frame(x: f64, y: f64, z: f64) -> Axis3 {
    Axis3::from_normal(
        Point3::new(x, y, z),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap()
}

// `new Geom_OffsetSurface(theBasis, theOffset)` (isNotCheckC0 = false).
fn offset_of(basis: Handle<dyn GeomSurface>, offset: f64) -> Handle<GeomOffsetSurface> {
    Handle::new(GeomOffsetSurface::new(&basis, offset, false).unwrap())
}

// `GeomAdaptor_Surface anAdaptor(theSurface)`.
fn adaptor_of(surface: &Handle<dyn GeomSurface>) -> GeomAdaptorSurface {
    GeomAdaptorSurface::with_surface(surface).unwrap()
}

// The fixture SetUp: a plane through the origin with a Z normal, offset by 3.
fn SetUp() -> Handle<GeomOffsetSurface> {
    // Create a plane as basis surface
    let aBasisSurface: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::new(z_frame(0.0, 0.0, 0.0))).cast();

    // Create offset surface
    let anOffsetValue: f64 = 3.0;

    offset_of(aBasisSurface, anOffsetValue)
}

// `new Geom_OffsetSurface(*surface)` of the copy tests.
fn copied(surface: &GeomOffsetSurface) -> Handle<GeomOffsetSurface> {
    Handle::new(GeomOffsetSurface::copy_of(surface).unwrap())
}

// Helper to compare vectors with tolerance.
fn VectorsEqual(theV1: Vector3, theV2: Vector3, theTol: f64) -> bool {
    (theV1 - theV2).magnitude() < theTol
}

// The D0 block of the Derivatives tests: both adaptors at (u, v).
fn check_d0(a: &GeomAdaptorSurface, b: &GeomAdaptorSurface, (u, v): (f64, f64), aTol: f64) {
    let aP1 = a.d0(u, v).unwrap();
    let aP2 = b.d0(u, v).unwrap();
    assert!(aP1.is_equal(&aP2, aTol), "D0 mismatch at u={u}, v={v}");
}

// The D1 block of the Derivatives tests: point, D1U and D1V.
fn check_d1(a: &GeomAdaptorSurface, b: &GeomAdaptorSurface, (u, v): (f64, f64), aTol: f64) {
    let r1 = a.d1(u, v).unwrap();
    let r2 = b.d1(u, v).unwrap();
    assert!(
        r1.point.is_equal(&r2.point, aTol),
        "D1 point mismatch at u={u}, v={v}"
    );
    assert!(
        VectorsEqual(r1.d1u, r2.d1u, aTol),
        "D1U mismatch at u={u}, v={v}"
    );
    assert!(
        VectorsEqual(r1.d1v, r2.d1v, aTol),
        "D1V mismatch at u={u}, v={v}"
    );
}

// The D2 block of the Derivatives tests: point, D2U, D2V and D2UV.
fn check_d2(a: &GeomAdaptorSurface, b: &GeomAdaptorSurface, (u, v): (f64, f64), aTol: f64) {
    let r1 = a.d2(u, v).unwrap();
    let r2 = b.d2(u, v).unwrap();
    assert!(
        r1.point.is_equal(&r2.point, aTol),
        "D2 point mismatch at u={u}, v={v}"
    );
    assert!(
        VectorsEqual(r1.d2u, r2.d2u, aTol),
        "D2U mismatch at u={u}, v={v}"
    );
    assert!(
        VectorsEqual(r1.d2v, r2.d2v, aTol),
        "D2V mismatch at u={u}, v={v}"
    );
    assert!(
        VectorsEqual(r1.d2uv, r2.d2uv, aTol),
        "D2UV mismatch at u={u}, v={v}"
    );
}

// The D3 block of the Derivatives tests: point, D3U, D3V, D3UUV and D3UVV.
fn check_d3(a: &GeomAdaptorSurface, b: &GeomAdaptorSurface, (u, v): (f64, f64), aTol: f64) {
    let r1 = a.d3(u, v).unwrap();
    let r2 = b.d3(u, v).unwrap();
    assert!(
        r1.point.is_equal(&r2.point, aTol),
        "D3 point mismatch at u={u}, v={v}"
    );
    assert!(
        VectorsEqual(r1.d3u, r2.d3u, aTol),
        "D3U mismatch at u={u}, v={v}"
    );
    assert!(
        VectorsEqual(r1.d3v, r2.d3v, aTol),
        "D3V mismatch at u={u}, v={v}"
    );
    assert!(
        VectorsEqual(r1.d3uuv, r2.d3uuv, aTol),
        "D3UUV mismatch at u={u}, v={v}"
    );
    assert!(
        VectorsEqual(r1.d3uvv, r2.d3uvv, aTol),
        "D3UVV mismatch at u={u}, v={v}"
    );
}

// `for (double x = first; x <= last (or < last); x += step)` as a list of values.
fn steps(first: f64, step: f64, keep: impl Fn(f64) -> bool) -> Vec<f64> {
    let mut values: Vec<f64> = Vec::new();
    let mut x: f64 = first;
    while keep(x) {
        values.push(x);
        x += step;
    }
    values
}

include!("geom_offset_surface/suites_0.rs");
include!("geom_offset_surface/suites_1.rs");
