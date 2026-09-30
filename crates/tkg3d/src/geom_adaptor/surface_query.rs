// Rust translation of ModelingData/TKG3d/GeomAdaptor/GeomAdaptor_Surface.cxx (UResolution, VResolution and the surface-kind queries) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Resolutions and surface-kind queries of `GeomAdaptor_Surface`.
use super::curve::GeomAdaptorCurve;
use super::surface::GeomAdaptorSurface;
use super::surface_shape::Direction;
use super::surface_state::{SurfaceData, SurfaceState};
use crate::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use crate::geom::errors::no_such_object;
use crate::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomCircle, GeomConicalSurface, GeomCurve,
    GeomCylindricalSurface, GeomOffsetSurface, GeomSphericalSurface, GeomSurface,
    GeomSurfaceOfRevolution, GeomSweptSurface, GeomToroidalSurface,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkmath::gp::{Axis1, Cone3, Cylinder3, Direction3, Plane3, Sphere3, Torus3};
use std::f64::consts::PI;

// The last step of both resolutions: the angle subtended by a chord of 2 * Res.
fn angle_of(res: f64) -> f64 {
    if res <= 1. {
        return 2. * res.asin();
    }
    2. * PI
}

// R3d / (2 R) when R exceeds Precision::Confusion(), 0 otherwise.
fn half_ratio(r3d: f64, radius: f64) -> f64 {
    if radius > precision::confusion() {
        return r3d / (2. * radius);
    }
    0.
}

// The radius of the loaded Geom surface of class `T` (natively a down cast of mySurface).
fn radius_of<T: RuntimeTyped>(
    state: &SurfaceState,
    radius: impl Fn(&T) -> f64,
) -> Result<f64, Failure> {
    let surface: Handle<T> = state.surface.cast();
    surface.get().map(radius).ok_or_else(|| {
        no_such_object("GeomAdaptor_Surface: the loaded surface does not match its kind")
    })
}

/// `UResolution`; `u_resolution(&state, 1e-7)?`.
pub(super) fn u_resolution(state: &SurfaceState, r3d: f64) -> Result<f64, Failure> {
    let res: f64 = match &state.data {
        SurfaceData::Extrusion(data) => return data.curve()?.resolution(r3d),
        SurfaceData::Torus(_) => half_ratio(
            r3d,
            radius_of(state, |t: &GeomToroidalSurface| {
                t.major_radius() + t.minor_radius()
            })?,
        ),
        SurfaceData::Sphere(_) => half_ratio(r3d, radius_of(state, GeomSphericalSurface::radius)?),
        SurfaceData::Cylinder(_) => {
            half_ratio(r3d, radius_of(state, GeomCylindricalSurface::radius)?)
        }
        SurfaceData::Cone(_) => return cone_u_resolution(state, r3d),
        SurfaceData::Plane(_) => return Ok(r3d),
        SurfaceData::Bezier(data) => return Ok(data.surface.resolution(r3d)?.0),
        SurfaceData::BSpline(data) => return Ok(data.surface.resolution(r3d)?.0),
        SurfaceData::Offset(data) => return data.basis_adaptor.u_resolution(r3d),
        _ => return Ok(precision::parametric(r3d)),
    };
    Ok(angle_of(res))
}

// The cone: unknown on a range longer than 1e10, else R3d over the larger radius of
// the V isos at the adaptor V bounds.
fn cone_u_resolution(state: &SurfaceState, r3d: f64) -> Result<f64, Failure> {
    if state.v_last - state.v_first > 1.0e10 {
        // Not truly bounded => unknown resolution.
        return Ok(precision::parametric(r3d));
    }
    let cone: Handle<GeomConicalSurface> = state.surface.cast();
    let radius1: f64 = iso_radius(cone.v_iso(state.v_last)?)?;
    let radius2: f64 = iso_radius(cone.v_iso(state.v_first)?)?;
    let radius: f64 = if radius1 > radius2 { radius1 } else { radius2 };
    Ok(if radius > precision::confusion() {
        r3d / radius
    } else {
        0.
    })
}

fn iso_radius(curve: Handle<dyn GeomCurve>) -> Result<f64, Failure> {
    let circle: Handle<GeomCircle> = curve.cast();
    circle.get().map(GeomCircle::radius).ok_or_else(|| {
        no_such_object("GeomAdaptor_Surface::UResolution: the cone V iso is not a circle")
    })
}

/// `VResolution`; `v_resolution(&state, 1e-7)?`.
pub(super) fn v_resolution(state: &SurfaceState, r3d: f64) -> Result<f64, Failure> {
    let res: f64 = match &state.data {
        SurfaceData::Revolution(data) => return data.curve()?.resolution(r3d),
        SurfaceData::Torus(_) => {
            half_ratio(r3d, radius_of(state, GeomToroidalSurface::minor_radius)?)
        }
        SurfaceData::Sphere(_) => half_ratio(r3d, radius_of(state, GeomSphericalSurface::radius)?),
        SurfaceData::Extrusion(_)
        | SurfaceData::Cylinder(_)
        | SurfaceData::Cone(_)
        | SurfaceData::Plane(_) => return Ok(r3d),
        SurfaceData::Bezier(data) => return Ok(data.surface.resolution(r3d)?.1),
        SurfaceData::BSpline(data) => return Ok(data.surface.resolution(r3d)?.1),
        SurfaceData::Offset(data) => return data.basis_adaptor.v_resolution(r3d),
        _ => return Ok(precision::parametric(r3d)),
    };
    Ok(angle_of(res))
}

fn not_this_kind(member: &str) -> Failure {
    no_such_object(&format!(
        "GeomAdaptor_Surface::{member}: the loaded surface is not of this kind"
    ))
}

/// `Plane`; `plane(&state)?`.
pub(super) fn plane(state: &SurfaceState) -> Result<Plane3, Failure> {
    match state.data {
        SurfaceData::Plane(plane) => Ok(plane),
        _ => Err(not_this_kind("Plane")),
    }
}
/// `Cylinder`; `cylinder(&state)?`.
pub(super) fn cylinder(state: &SurfaceState) -> Result<Cylinder3, Failure> {
    match state.data {
        SurfaceData::Cylinder(cylinder) => Ok(cylinder),
        _ => Err(not_this_kind("Cylinder")),
    }
}
/// `Cone`; `cone(&state)?`.
pub(super) fn cone(state: &SurfaceState) -> Result<Cone3, Failure> {
    match state.data {
        SurfaceData::Cone(cone) => Ok(cone),
        _ => Err(not_this_kind("Cone")),
    }
}
/// `Sphere`; `sphere(&state)?`.
pub(super) fn sphere(state: &SurfaceState) -> Result<Sphere3, Failure> {
    match state.data {
        SurfaceData::Sphere(sphere) => Ok(sphere),
        _ => Err(not_this_kind("Sphere")),
    }
}
/// `Torus`; `torus(&state)?`.
pub(super) fn torus(state: &SurfaceState) -> Result<Torus3, Failure> {
    match state.data {
        SurfaceData::Torus(torus) => Ok(torus),
        _ => Err(not_this_kind("Torus")),
    }
}

/// `UDegree` / `VDegree`: the spline degree, or that of the basis curve of the swept
/// surface of that direction; `degree(&state, Direction::U)?`.
pub(super) fn degree(state: &SurfaceState, direction: Direction) -> Result<i32, Failure> {
    match (&state.data, direction) {
        (SurfaceData::BSpline(data), Direction::U) => Ok(data.surface.u_degree()),
        (SurfaceData::BSpline(data), Direction::V) => Ok(data.surface.v_degree()),
        (SurfaceData::Bezier(data), Direction::U) => Ok(data.surface.u_degree()),
        (SurfaceData::Bezier(data), Direction::V) => Ok(data.surface.v_degree()),
        (SurfaceData::Extrusion(data), Direction::U) => data.curve()?.degree(),
        (SurfaceData::Revolution(data), Direction::V) => data.curve()?.degree(),
        (_, Direction::U) => Err(not_this_kind("UDegree")),
        (_, Direction::V) => Err(not_this_kind("VDegree")),
    }
}

/// `NbUPoles` / `NbVPoles`, as `degree`; `nb_poles(&state, Direction::U)?`.
pub(super) fn nb_poles(state: &SurfaceState, direction: Direction) -> Result<i32, Failure> {
    match (&state.data, direction) {
        (SurfaceData::BSpline(data), Direction::U) => Ok(data.surface.nb_u_poles()),
        (SurfaceData::BSpline(data), Direction::V) => Ok(data.surface.nb_v_poles()),
        (SurfaceData::Bezier(data), Direction::U) => Ok(data.surface.nb_u_poles()),
        (SurfaceData::Bezier(data), Direction::V) => Ok(data.surface.nb_v_poles()),
        (SurfaceData::Extrusion(data), Direction::U) => data.curve()?.nb_poles(),
        (SurfaceData::Revolution(data), Direction::V) => data.curve()?.nb_poles(),
        (_, Direction::U) => Err(not_this_kind("NbUPoles")),
        (_, Direction::V) => Err(not_this_kind("NbVPoles")),
    }
}

/// `NbUKnots` (B-spline or extrusion basis) / `NbVKnots` (B-spline only);
/// `nb_knots(&state, Direction::U)?`.
pub(super) fn nb_knots(state: &SurfaceState, direction: Direction) -> Result<i32, Failure> {
    match (&state.data, direction) {
        (SurfaceData::BSpline(data), Direction::U) => Ok(data.surface.nb_u_knots()),
        (SurfaceData::BSpline(data), Direction::V) => Ok(data.surface.nb_v_knots()),
        (SurfaceData::Extrusion(data), Direction::U) => data.curve()?.nb_knots(),
        (_, Direction::U) => Err(not_this_kind("NbUKnots")),
        (_, Direction::V) => Err(not_this_kind("NbVKnots")),
    }
}

/// `IsURational` / `IsVRational`: false for every kind but the splines;
/// `is_rational(&state, Direction::U)`.
pub(super) fn is_rational(state: &SurfaceState, direction: Direction) -> bool {
    match (&state.data, direction) {
        (SurfaceData::BSpline(data), Direction::U) => data.surface.is_u_rational(),
        (SurfaceData::BSpline(data), Direction::V) => data.surface.is_v_rational(),
        (SurfaceData::Bezier(data), Direction::U) => data.surface.is_u_rational(),
        (SurfaceData::Bezier(data), Direction::V) => data.surface.is_v_rational(),
        _ => false,
    }
}

/// `Bezier`; `bezier(&state)?`.
pub(super) fn bezier(state: &SurfaceState) -> Result<Handle<GeomBezierSurface>, Failure> {
    match &state.data {
        SurfaceData::Bezier(_) => Ok(state.surface.cast()),
        _ => Err(not_this_kind("Bezier")),
    }
}
/// `BSpline`; `bspline(&state)?`.
pub(super) fn bspline(state: &SurfaceState) -> Result<Handle<GeomBSplineSurface>, Failure> {
    match &state.data {
        SurfaceData::BSpline(data) => Ok(data.surface.clone()),
        _ => Err(not_this_kind("BSpline")),
    }
}

/// `AxeOfRevolution`: the axis of the loaded Geom surface of revolution;
/// `axe_of_revolution(&state)?`.
pub(super) fn axe_of_revolution(state: &SurfaceState) -> Result<Axis1, Failure> {
    let revolution: Handle<GeomSurfaceOfRevolution> = state.surface.cast();
    match (&state.data, revolution.get()) {
        (SurfaceData::Revolution(_), Some(surface)) => Ok(surface.axis()),
        _ => Err(not_this_kind("AxeOfRevolution")),
    }
}
/// `Direction`: the direction of the loaded Geom surface of extrusion;
/// `direction(&state)?`.
pub(super) fn direction(state: &SurfaceState) -> Result<Direction3, Failure> {
    let swept: Handle<dyn GeomSweptSurface> = state.surface.cast();
    match (&state.data, swept.get()) {
        (SurfaceData::Extrusion(_), Some(surface)) => Ok(surface.direction()),
        _ => Err(not_this_kind("Direction")),
    }
}
/// `BasisCurve`: a new adaptor on the whole basis curve of the loaded Geom swept
/// surface; `basis_curve(&state)?`.
pub(super) fn basis_curve(state: &SurfaceState) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    let swept: Handle<dyn GeomSweptSurface> = state.surface.cast();
    match (&state.data, swept.get()) {
        (SurfaceData::Extrusion(_) | SurfaceData::Revolution(_), Some(surface)) => {
            let curve = GeomAdaptorCurve::with_curve(&surface.basis_curve())?;
            Ok(Handle::new(curve).cast())
        }
        _ => Err(not_this_kind("BasisCurve")),
    }
}

/// `BasisSurface` as a value: a new adaptor on the basis of the loaded offset
/// surface over the adaptor bounds, zero tolerances; `basis_surface_adaptor(&state)?`.
pub(super) fn basis_surface_adaptor(state: &SurfaceState) -> Result<GeomAdaptorSurface, Failure> {
    let offset: Handle<GeomOffsetSurface> = state.surface.cast();
    match (&state.data, offset.get()) {
        (SurfaceData::Offset(_), Some(surface)) => GeomAdaptorSurface::with_bounds(
            &surface.basis_surface(),
            (state.u_first, state.u_last, state.v_first, state.v_last),
        ),
        _ => Err(not_this_kind("BasisSurface")),
    }
}
/// `OffsetValue`; the native message names BasisSurface; `offset_value(&state)?`.
pub(super) fn offset_value(state: &SurfaceState) -> Result<f64, Failure> {
    let offset: Handle<GeomOffsetSurface> = state.surface.cast();
    match (&state.data, offset.get()) {
        (SurfaceData::Offset(_), Some(surface)) => Ok(surface.offset()),
        _ => Err(not_this_kind("BasisSurface")),
    }
}
