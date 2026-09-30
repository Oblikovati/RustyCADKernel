// Rust translation of FoundationClasses/TKMath/ElSLib/ElSLib.cxx *D3 at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Points with partial derivatives up to the third of curved elementary surfaces.
//!
//! The D3 evaluators build `du` as `-(A2·X - A1·Y)` where D1/D2 use `-A2·X + A1·Y`; the two
//! differ only in the sign of a zero component, and each keeps its native form.
use super::frame::{Frame, flush, torus_tolerance, vector};
use super::second::SurfaceD2;
use crate::gp::{Axis3, Point3, Vector3, Xyz};

/// A surface point with its partial derivatives up to the third.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceD3 {
    /// Point and derivatives up to the second.
    pub second: SurfaceD2,
    /// `∂³P/∂u³`.
    pub duuu: Vector3,
    /// `∂³P/∂v³`.
    pub dvvv: Vector3,
    /// `∂³P/∂u²∂v`.
    pub duuv: Vector3,
    /// `∂³P/∂u∂v²`.
    pub duvv: Vector3,
}
fn zero() -> Vector3 {
    Vector3::new(0.0, 0.0, 0.0)
}
fn lower(point: Point3, du: Xyz, dv: Xyz, duu: Xyz) -> SurfaceD2 {
    SurfaceD2 {
        point,
        du: vector(du),
        dv: vector(dv),
        duu: vector(duu),
        dvv: zero(),
        duv: zero(),
    }
}

/// Cone point with derivatives up to the third; `cone_d3(0.5,1.0,frame,2.0,0.3)`.
pub fn cone_d3(u: f64, v: f64, position: Axis3, radius: f64, semi_angle: f64) -> SurfaceD3 {
    let frame = Frame::of(position);
    let (cos_u, sin_u) = (u.cos(), u.sin());
    let (cos_a, sin_a) = (semi_angle.cos(), semi_angle.sin());
    let ring = radius + v * sin_a;
    let (a1, a2, a3) = (ring * cos_u, ring * sin_u, v * cos_a);
    let (r1, r2) = (sin_a * cos_u, sin_a * sin_u);
    let (circle, slope, tangent) = (
        frame.pair(a1, a2),
        frame.pair(r1, r2),
        frame.difference(a2, a1),
    );
    let point = frame.point(circle + frame.z * a3);
    let mut second = lower(
        point,
        tangent.reversed(),
        slope + frame.z * cos_a,
        circle.reversed(),
    );
    second.duv = vector(frame.pair(-r2, r1));
    SurfaceD3 {
        second,
        duuu: vector(tangent),
        dvvv: zero(),
        duuv: vector(slope.reversed()),
        duvv: zero(),
    }
}
/// Cylinder point with derivatives up to the third; `cylinder_d3(0.5,1.0,frame,2.0)`.
pub fn cylinder_d3(u: f64, v: f64, position: Axis3, radius: f64) -> SurfaceD3 {
    let frame = Frame::of(position);
    let (a1, a2) = (radius * u.cos(), radius * u.sin());
    let (circle, tangent) = (frame.pair(a1, a2), frame.difference(a2, a1));
    let point = frame.point(circle + frame.z * v);
    SurfaceD3 {
        second: lower(point, tangent.reversed(), frame.z, circle.reversed()),
        duuu: vector(tangent),
        dvvv: zero(),
        duuv: zero(),
        duvv: zero(),
    }
}
/// Sphere point with derivatives up to the third; the native `duvv` equals `duuu`, which is
/// kept as OCCT 3d097a computes it; `sphere_d3(0.5,0.2,frame,2.0)`.
pub fn sphere_d3(u: f64, v: f64, position: Axis3, radius: f64) -> SurfaceD3 {
    let frame = Frame::of(position);
    let (cos_u, sin_u) = (u.cos(), u.sin());
    let (ring, height) = (radius * v.cos(), radius * v.sin());
    let (a1, a2, a3, a4) = (ring * cos_u, ring * sin_u, height * cos_u, height * sin_u);
    spherical_d3(&frame, [a1, a2, a3, a4], (ring, height), None)
}
/// Torus point with derivatives up to the third, flushing small terms;
/// `torus_d3(0.5,0.2,frame,5.0,1.0)`.
pub fn torus_d3(u: f64, v: f64, position: Axis3, major: f64, minor: f64) -> SurfaceD3 {
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
    spherical_d3(
        &frame,
        [a1, a2, a3, a4],
        (tube_cos, tube_sin),
        Some((a5, a6)),
    )
}
/// Shared sphere/torus D3 layout; `tube` carries the torus `(A5, A6)` terms.
fn spherical_d3(
    frame: &Frame,
    [a1, a2, a3, a4]: [f64; 4],
    (cosine, sine): (f64, f64),
    tube: Option<(f64, f64)>,
) -> SurfaceD3 {
    let (circle, meridian) = (frame.pair(a1, a2), frame.pair(a3, a4));
    let tangent = frame.difference(a2, a1);
    let (axial_cos, axial_sin) = (frame.z * cosine, frame.z * sine);
    let (dvv, duvv) = tube_terms(frame, (circle, tangent, axial_sin), tube);
    let dv = meridian.reversed() + axial_cos;
    let mut second = lower(
        frame.point(circle + axial_sin),
        tangent.reversed(),
        dv,
        circle.reversed(),
    );
    second.duv = vector(frame.difference(a4, a3));
    second.dvv = vector(dvv);
    SurfaceD3 {
        second,
        duuu: vector(tangent),
        dvvv: vector(meridian - axial_cos),
        duuv: vector(meridian),
        duvv: vector(duvv),
    }
}
/// `(dvv, duvv)`: the torus uses its `A5/A6` terms; the sphere reuses `-circle - axial` and,
/// as native SphereD3 does, the `du`-derived tangent for `duvv`.
fn tube_terms(
    frame: &Frame,
    (circle, tangent, axial_sin): (Xyz, Xyz, Xyz),
    tube: Option<(f64, f64)>,
) -> (Xyz, Xyz) {
    match tube {
        Some((a5, a6)) => (
            frame.x * -a5 - frame.y * a6 - axial_sin,
            frame.difference(a6, a5),
        ),
        None => (circle.reversed() - axial_sin, tangent),
    }
}
