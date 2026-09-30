// Rust translation of FoundationClasses/TKMath/ElSLib/ElSLib.cxx *Value/*D0 at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Points of elementary surfaces; each `*D0` repeats its `*Value` arithmetic exactly.
use super::frame::{Frame, flush, torus_tolerance};
use crate::gp::{Axis3, Point3};

/// Plane point `u·X + v·Y + origin`; `plane_value(1.0,2.0,frame)`.
pub fn plane_value(u: f64, v: f64, position: Axis3) -> Point3 {
    let frame = Frame::of(position);
    frame.point(frame.pair(u, v))
}
/// Cone point on the circle of radius `R + v·sin(a)` at height `v·cos(a)`;
/// `cone_value(0.5,1.0,frame,2.0,0.3)`.
pub fn cone_value(u: f64, v: f64, position: Axis3, radius: f64, semi_angle: f64) -> Point3 {
    let frame = Frame::of(position);
    let ring = radius + v * semi_angle.sin();
    let height = v * semi_angle.cos();
    frame.point(frame.triple(ring * u.cos(), ring * u.sin(), height))
}
/// Cylinder point `R·cos(u)·X + R·sin(u)·Y + v·Z + origin`; `cylinder_value(0.5,1.0,frame,2.0)`.
pub fn cylinder_value(u: f64, v: f64, position: Axis3, radius: f64) -> Point3 {
    let frame = Frame::of(position);
    frame.point(frame.triple(radius * u.cos(), radius * u.sin(), v))
}
/// Sphere point with longitude `u` and latitude `v`; `sphere_value(0.5,0.2,frame,2.0)`.
pub fn sphere_value(u: f64, v: f64, position: Axis3, radius: f64) -> Point3 {
    let frame = Frame::of(position);
    let ring = radius * v.cos();
    let height = radius * v.sin();
    frame.point(frame.triple(ring * u.cos(), ring * u.sin(), height))
}
/// Torus point with terms within `10·(r+R)·ε` flushed to zero; `torus_value(0.5,0.2,frame,5.0,1.0)`.
pub fn torus_value(u: f64, v: f64, position: Axis3, major: f64, minor: f64) -> Point3 {
    let frame = Frame::of(position);
    let ring = major + minor * v.cos();
    let height = minor * v.sin();
    let tolerance = torus_tolerance(major, minor);
    let (a1, a2) = (ring * u.cos(), ring * u.sin());
    frame.point(frame.triple(
        flush(a1, tolerance),
        flush(a2, tolerance),
        flush(height, tolerance),
    ))
}

/// ElSLib::PlaneD0, identical to [`plane_value`]; `plane_d0(1.0,2.0,frame)`.
pub fn plane_d0(u: f64, v: f64, position: Axis3) -> Point3 {
    plane_value(u, v, position)
}
/// ElSLib::ConeD0, identical to [`cone_value`]; `cone_d0(0.5,1.0,frame,2.0,0.3)`.
pub fn cone_d0(u: f64, v: f64, position: Axis3, radius: f64, semi_angle: f64) -> Point3 {
    cone_value(u, v, position, radius, semi_angle)
}
/// ElSLib::CylinderD0, identical to [`cylinder_value`]; `cylinder_d0(0.5,1.0,frame,2.0)`.
pub fn cylinder_d0(u: f64, v: f64, position: Axis3, radius: f64) -> Point3 {
    cylinder_value(u, v, position, radius)
}
/// ElSLib::SphereD0, identical to [`sphere_value`]; `sphere_d0(0.5,0.2,frame,2.0)`.
pub fn sphere_d0(u: f64, v: f64, position: Axis3, radius: f64) -> Point3 {
    sphere_value(u, v, position, radius)
}
/// ElSLib::TorusD0, identical to [`torus_value`]; `torus_d0(0.5,0.2,frame,5.0,1.0)`.
pub fn torus_d0(u: f64, v: f64, position: Axis3, major: f64, minor: f64) -> Point3 {
    torus_value(u, v, position, major, minor)
}
