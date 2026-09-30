// Rust translation of FoundationClasses/TKMath/ElSLib/ElSLib.cxx *D1 at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Points and first partial derivatives of elementary surfaces.
use super::frame::{Frame, flush, torus_tolerance, vector};
use crate::gp::{Axis3, Point3, Vector3};

/// A surface point with its `∂/∂u` and `∂/∂v` derivatives.
pub type SurfaceD1 = (Point3, Vector3, Vector3);

/// Plane point with the X and Y directions as derivatives; `plane_d1(1.0,2.0,frame)`.
pub fn plane_d1(u: f64, v: f64, position: Axis3) -> SurfaceD1 {
    let frame = Frame::of(position);
    (
        frame.point(frame.pair(u, v)),
        vector(frame.x),
        vector(frame.y),
    )
}
/// Cone point and first derivatives; `cone_d1(0.5,1.0,frame,2.0,0.3)`.
pub fn cone_d1(u: f64, v: f64, position: Axis3, radius: f64, semi_angle: f64) -> SurfaceD1 {
    let frame = Frame::of(position);
    let (cos_u, sin_u) = (u.cos(), u.sin());
    let (cos_a, sin_a) = (semi_angle.cos(), semi_angle.sin());
    let ring = radius + v * sin_a;
    let (a1, a2, a3) = (ring * cos_u, ring * sin_u, v * cos_a);
    (
        frame.point(frame.triple(a1, a2, a3)),
        vector(frame.pair(-a2, a1)),
        vector(frame.triple(sin_a * cos_u, sin_a * sin_u, cos_a)),
    )
}
/// Cylinder point and first derivatives; `cylinder_d1(0.5,1.0,frame,2.0)`.
pub fn cylinder_d1(u: f64, v: f64, position: Axis3, radius: f64) -> SurfaceD1 {
    let frame = Frame::of(position);
    let (a1, a2) = (radius * u.cos(), radius * u.sin());
    (
        frame.point(frame.triple(a1, a2, v)),
        vector(frame.pair(-a2, a1)),
        vector(frame.z),
    )
}
/// Sphere point and first derivatives; `sphere_d1(0.5,0.2,frame,2.0)`.
pub fn sphere_d1(u: f64, v: f64, position: Axis3, radius: f64) -> SurfaceD1 {
    let frame = Frame::of(position);
    let (cos_u, sin_u) = (u.cos(), u.sin());
    let (ring, height) = (radius * v.cos(), radius * v.sin());
    let (a1, a2, a3, a4) = (ring * cos_u, ring * sin_u, height * cos_u, height * sin_u);
    (
        frame.point(frame.triple(a1, a2, height)),
        vector(frame.pair(-a2, a1)),
        vector(frame.meridian(a3, a4, ring)),
    )
}
/// Torus point and first derivatives with terms within `10·(r+R)·ε` flushed to zero;
/// `torus_d1(0.5,0.2,frame,5.0,1.0)`.
pub fn torus_d1(u: f64, v: f64, position: Axis3, major: f64, minor: f64) -> SurfaceD1 {
    let frame = Frame::of(position);
    let (cos_u, sin_u) = (u.cos(), u.sin());
    let (tube_cos, tube_sin) = (minor * v.cos(), minor * v.sin());
    let ring = major + tube_cos;
    let tolerance = torus_tolerance(major, minor);
    let [a1, a2, a3, a4] = [
        ring * cos_u,
        ring * sin_u,
        tube_sin * cos_u,
        tube_sin * sin_u,
    ]
    .map(|value| flush(value, tolerance));
    (
        frame.point(frame.triple(a1, a2, tube_sin)),
        vector(frame.pair(-a2, a1)),
        vector(frame.meridian(a3, a4, tube_cos)),
    )
}
