//! Direct translation of GeomFill_Gordon_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! `EXPECT_THROW(..., StdFail_NotDone)` checks the exception type of the `Err`.
#![allow(
    non_snake_case,
    non_upper_case_globals,
    dead_code,
    clippy::excessive_precision,
    clippy::approx_constant
)]
use rustycad_tkernel::ncollection::{Array1, Array2, HArray1};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomCurve, GeomLine, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkgeomalgo::geom_api::{Interpolate, ProjectPointOnCurve};
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillGordon, GeomFillNetworkSurface, GordonApproximationMode, GordonBuildStage,
    GordonStatus, NetworkSurfaceStatus,
};
use rustycad_tkmath::gp::{Direction3, Point3, Xyz};
use std::f64::consts::PI;

include!("geomfill_gordon/helpers.rs");

// `new Geom_TrimmedCurve(new Geom_Line(P, D), u1, u2)`.
fn trimmed_line(
    point: Point3,
    (dx, dy, dz): (f64, f64, f64),
    u1: f64,
    u2: f64,
) -> Handle<dyn GeomCurve> {
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
        point,
        Direction3::new(dx, dy, dz).unwrap(),
    ))
    .cast();
    Handle::new(GeomTrimmedCurve::new(&line, u1, u2, true, true).unwrap()).cast()
}

// The type name of the exception of `Surface()`.
fn surface_error(gordon: &GeomFillGordon) -> String {
    match gordon.surface() {
        Ok(_) => String::from("no exception"),
        Err(failure) => failure.exception_type().to_string(),
    }
}

// `aGordon.Init(Profiles, Guides, Tolerance); aGordon.Perform();`.
fn gordon_of(
    aProfiles: &Array1<'static, Handle<dyn GeomCurve>>,
    aGuides: &Array1<'static, Handle<dyn GeomCurve>>,
    theTolerance: f64,
) -> GeomFillGordon {
    let mut aGordon = GeomFillGordon::new();
    aGordon.init(aProfiles, aGuides, theTolerance).unwrap();
    aGordon.perform().unwrap();
    aGordon
}

// `NCollection_Array1<occ::handle<Geom_Curve>>` of any curves.
fn curve_array(list: Vec<Handle<dyn GeomCurve>>) -> Array1<'static, Handle<dyn GeomCurve>> {
    let mut array: Array1<'static, Handle<dyn GeomCurve>> =
        Array1::with_bounds(1, list.len() as i32).unwrap();
    for (i, c) in list.into_iter().enumerate() {
        array[i as i32 + 1] = c;
    }
    array
}

// Profiles and guides of a network, as the native test arrays.
type CurveNetwork = (
    Array1<'static, Handle<dyn GeomCurve>>,
    Array1<'static, Handle<dyn GeomCurve>>,
);

// The first and last U knots of a surface.
fn u_range(s: &Handle<GeomBSplineSurface>) -> (f64, f64) {
    (s.u_knot(1).unwrap(), s.u_knot(s.nb_u_knots()).unwrap())
}

// The first and last V knots of a surface.
fn v_range(s: &Handle<GeomBSplineSurface>) -> (f64, f64) {
    (s.v_knot(1).unwrap(), s.v_knot(s.nb_v_knots()).unwrap())
}

include!("geomfill_gordon/suites_1.rs");
include!("geomfill_gordon/suites_2.rs");
include!("geomfill_gordon/suites_3.rs");
