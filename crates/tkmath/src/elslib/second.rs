// Rust translation of FoundationClasses/TKMath/ElSLib/ElSLib.cxx *D2 at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Points with first and second partial derivatives of curved elementary surfaces.
use super::frame::{Frame, flush, torus_tolerance, vector};
use crate::gp::{Axis3, Point3, Vector3};

/// A surface point with its first and second partial derivatives.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceD2 {
    /// Surface point.
    pub point: Point3,
    /// `∂P/∂u`.
    pub du: Vector3,
    /// `∂P/∂v`.
    pub dv: Vector3,
    /// `∂²P/∂u²`.
    pub duu: Vector3,
    /// `∂²P/∂v²`.
    pub dvv: Vector3,
    /// `∂²P/∂u∂v`.
    pub duv: Vector3,
}
fn zero() -> Vector3 {
    Vector3::new(0.0, 0.0, 0.0)
}

/// Cone point with derivatives up to the second; `cone_d2(0.5,1.0,frame,2.0,0.3)`.
pub fn cone_d2(u: f64, v: f64, position: Axis3, radius: f64, semi_angle: f64) -> SurfaceD2 {
    let frame = Frame::of(position);
    let (cos_u, sin_u) = (u.cos(), u.sin());
    let (cos_a, sin_a) = (semi_angle.cos(), semi_angle.sin());
    let ring = radius + v * sin_a;
    let (a1, a2, a3) = (ring * cos_u, ring * sin_u, v * cos_a);
    let (r1, r2) = (sin_a * cos_u, sin_a * sin_u);
    let circle = frame.pair(a1, a2);
    SurfaceD2 {
        point: frame.point(circle + frame.z * a3),
        du: vector(frame.pair(-a2, a1)),
        dv: vector(frame.triple(r1, r2, cos_a)),
        duu: vector(circle.reversed()),
        dvv: zero(),
        duv: vector(frame.pair(-r2, r1)),
    }
}
/// Cylinder point with derivatives up to the second; `cylinder_d2(0.5,1.0,frame,2.0)`.
pub fn cylinder_d2(u: f64, v: f64, position: Axis3, radius: f64) -> SurfaceD2 {
    let frame = Frame::of(position);
    let (a1, a2) = (radius * u.cos(), radius * u.sin());
    let circle = frame.pair(a1, a2);
    SurfaceD2 {
        point: frame.point(circle + frame.z * v),
        du: vector(frame.pair(-a2, a1)),
        dv: vector(frame.z),
        duu: vector(circle.reversed()),
        dvv: zero(),
        duv: zero(),
    }
}
/// Sphere point with derivatives up to the second; `sphere_d2(0.5,0.2,frame,2.0)`.
pub fn sphere_d2(u: f64, v: f64, position: Axis3, radius: f64) -> SurfaceD2 {
    let frame = Frame::of(position);
    let (cos_u, sin_u) = (u.cos(), u.sin());
    let (ring, height) = (radius * v.cos(), radius * v.sin());
    let (a1, a2, a3, a4) = (ring * cos_u, ring * sin_u, height * cos_u, height * sin_u);
    let circle = frame.pair(a1, a2);
    let axial = frame.z * height;
    SurfaceD2 {
        point: frame.point(circle + axial),
        du: vector(frame.pair(-a2, a1)),
        dv: vector(frame.meridian(a3, a4, ring)),
        duu: vector(circle.reversed()),
        dvv: vector(circle.reversed() - axial),
        duv: vector(frame.difference(a4, a3)),
    }
}
/// Torus point with derivatives up to the second, flushing small terms;
/// `torus_d2(0.5,0.2,frame,5.0,1.0)`.
pub fn torus_d2(u: f64, v: f64, position: Axis3, major: f64, minor: f64) -> SurfaceD2 {
    let frame = Frame::of(position);
    let (cos_u, sin_u) = (u.cos(), u.sin());
    let (tube_cos, tube_sin) = (minor * v.cos(), minor * v.sin());
    let ring = major + tube_cos;
    let tolerance = torus_tolerance(major, minor);
    let [a1, a2, a3, a4, a5, a6] = [
        ring * cos_u,
        ring * sin_u,
        tube_sin * cos_u,
        tube_sin * sin_u,
        tube_cos * cos_u,
        tube_cos * sin_u,
    ]
    .map(|value| flush(value, tolerance));
    let circle = frame.pair(a1, a2);
    let axial = frame.z * tube_sin;
    SurfaceD2 {
        point: frame.point(circle + axial),
        du: vector(frame.pair(-a2, a1)),
        dv: vector(frame.meridian(a3, a4, tube_cos)),
        duu: vector(circle.reversed()),
        dvv: vector(frame.x * -a5 - frame.y * a6 - axial),
        duv: vector(frame.difference(a4, a3)),
    }
}
