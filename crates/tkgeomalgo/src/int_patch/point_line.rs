// Copyright (c) 1992-1999 Matra Datavision
// Copyright (c) 1999-2015 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_PointLine.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_PointLine`: the lines defined by a set of points (walking and
//! restriction lines), and the curvature radius of an intersection line.
use super::line::IntPatchLine;
use super::point::IntPatchPoint;
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::GeomSurfaceD2;
use rustycad_tkmath::gp::{Point2, Point3, Vector3};

/// `IntPatch_PointLine`: the pure virtual members of the point lines
/// ([`IntPatchWLine`](super::IntPatchWLine) and [`IntPatchRLine`](super::IntPatchRLine)),
/// reached through `Handle<dyn IntPatchPointLine>`. The members that read the points
/// return `Result`, since a restriction line may have no point set.
///
/// ```
/// use rustycad_tkgeomalgo::int_patch::{IntPatchPoint, IntPatchPointLine, IntPatchRLine};
/// let line = IntPatchRLine::new_undecided(false);
/// line.add_vertex(&IntPatchPoint::new(), false);
/// assert_eq!(line.nb_vertex(), 1);
/// assert!(line.nb_pnts().is_err());
/// ```
pub trait IntPatchPointLine: IntPatchLine {
    /// `AddVertex(Pnt, theIsPrepend)`: appends the vertex, or prepends it.
    fn add_vertex(&self, pnt: &IntPatchPoint, prepend: bool);
    /// `NbPnts()`: the number of points of the line.
    fn nb_pnts(&self) -> Result<i32, Failure>;
    /// `NbVertex()`.
    fn nb_vertex(&self) -> i32;
    /// `Point(Index)`: the point of range `index`, a copy.
    fn point(&self, index: i32) -> Result<IntSurfPntOn2S, Failure>;
    /// `Vertex(Index)`: the vertex of range `index`, a copy.
    fn vertex(&self, index: i32) -> Result<IntPatchPoint, Failure>;
    /// `ChangeVertex(Index)`: runs `edit` on the vertex of range `index` in place.
    fn change_vertex(
        &self,
        index: i32,
        edit: &mut dyn FnMut(&mut IntPatchPoint),
    ) -> Result<(), Failure>;
    /// `ClearVertexes()`.
    fn clear_vertexes(&self);
    /// `RemoveVertex(theIndex)`: Standard_OutOfRange outside `1..=NbVertex()`.
    fn remove_vertex(&self, index: i32) -> Result<(), Failure>;
    /// `Curve()`: the point set (a null handle when there is none).
    fn curve(&self) -> Handle<IntSurfLineOn2S>;
    /// `IsOutSurf1Box(P1)`: whether `p1` is out of the box of the points on S1.
    fn is_out_surf1_box(&self, p1: Point2) -> Result<bool, Failure>;
    /// `IsOutSurf2Box(P2)`: whether `p2` is out of the box of the points on S2.
    fn is_out_surf2_box(&self, p2: Point2) -> Result<bool, Failure>;
    /// `IsOutBox(P)`: whether `p` is out of the box of the 3d points.
    fn is_out_box(&self, p: Point3) -> Result<bool, Failure>;
}
impl_transient_interface!(
    dyn IntPatchPointLine,
    "IntPatch_PointLine",
    dyn IntPatchLine
);

/// `IntPatch_PointLine::CurvatureRadiusOfIntersLine(theS1, theS2, theUVPoint)`: the
/// radius of curvature of the intersection line of the two surfaces at the point, from
/// its first and second derivatives (Stoyagov's marching system); -1 when it cannot be
/// computed and `Precision::Infinite()` where the curvature is null.
///
/// ```ignore
/// let radius = curvature_radius_of_inters_line(&s1, &s2, &uv_point)?;
/// ```
pub fn curvature_radius_of_inters_line(
    s1: &Adaptor3dSurfaceHandle,
    s2: &Adaptor3dSurfaceHandle,
    uv_point: &IntSurfPntOn2S,
) -> Result<f64, Failure> {
    let (u1, v1, u2, v2) = uv_point.parameters();
    let d1: GeomSurfaceD2 = s1.d2(u1, v1)?;
    let d2: GeomSurfaceD2 = s2.d2(u2, v2)?;
    let n1: Vector3 = d1.d1u.crossed(d1.d1v);
    let n2: Vector3 = d2.d1u.crossed(d2.d1v);
    // Tangent vector to the intersection curve.
    let c_tan: Vector3 = n1.crossed(n2);
    let sq_magn_f_der: f64 = c_tan.square_magnitude();
    if sq_magn_f_der < 1.0e-8 {
        // 1.0e-4 (instead of aSmallValue) gives a stable computation between platforms
        // (test bugs modalg_7 bug29807_sc01).
        return Ok(-1.0);
    }
    let (du_s1, dv_s1) = surface_steps(c_tan, &d1, n1);
    let (du_s2, dv_s2) = surface_steps(c_tan, &d2, n2);
    let steps: [f64; 4] = [du_s1, dv_s1, du_s2, dv_s2];
    Ok(radius_from_system(
        c_tan,
        sq_magn_f_der,
        (&d1, &d2),
        (n1, n2),
        steps,
    ))
}

/// The derivatives of the surface parameters along the curve, as
/// `NonSingularProcessing()` of ApproxInt_ImpPrmSvSurfaces.gxx computes them.
fn surface_steps(c_tan: Vector3, d: &GeomSurfaceD2, n: Vector3) -> (f64, f64) {
    let sq_n_magn: f64 = n.square_magnitude();
    let tg_u: Vector3 = c_tan.crossed(d.d1u);
    let tg_v: Vector3 = c_tan.crossed(d.d1v);
    let delta_u: f64 = tg_v.square_magnitude() / sq_n_magn;
    let delta_v: f64 = tg_u.square_magnitude() / sq_n_magn;
    let du: f64 = delta_u.sqrt().copysign(tg_v.dot(n));
    let dv: f64 = -delta_v.sqrt().copysign(tg_u.dot(n));
    (du, dv)
}

/// Solves `{A*a + B*b = F1; B*a + C*b = F2}` for the principal normal `a*N1 + b*N2` of
/// the curve and returns the radius (Stoyagov, "Marching along surface/surface
/// intersection curves with an adaptive step length").
fn radius_from_system(
    c_tan: Vector3,
    sq_magn_f_der: f64,
    (d1, d2): (&GeomSurfaceD2, &GeomSurfaceD2),
    (n1, n2): (Vector3, Vector3),
    [du_s1, dv_s1, du_s2, dv_s2]: [f64; 4],
) -> f64 {
    let small_value: f64 = 1.0 / precision::infinite();
    let sq_small_value: f64 = small_value * small_value;
    let (a, b, c) = (n1.dot(n1), n1.dot(n2), n2.dot(n2));
    let det_syst: f64 = b * b - a * c;
    if det_syst.abs() < small_value {
        // Undetermined system solution.
        return -1.0;
    }
    let f1: f64 = du_s1 * du_s1 * d1.d2u.dot(n1)
        + 2.0 * du_s1 * dv_s1 * d1.d2uv.dot(n1)
        + dv_s1 * dv_s1 * d1.d2v.dot(n1);
    let f2: f64 = du_s2 * du_s2 * d2.d2u.dot(n2)
        + 2.0 * du_s2 * dv_s2 * d2.d2uv.dot(n2)
        + dv_s2 * dv_s2 * d2.d2v.dot(n2);
    // Principal normal to the intersection curve.
    let c_norm: Vector3 = (f1 * c - f2 * b) / det_syst * n1 + (a * f2 - f1 * b) / det_syst * n2;
    let sq_magn_s_der: f64 = c_norm.cross_square_magnitude(c_tan);
    if sq_magn_s_der < sq_small_value {
        // The intersection curve has a null curvature at the point.
        return precision::infinite();
    }
    // The square of the curvature radius.
    let fact_sq_rad: f64 = sq_magn_f_der * sq_magn_f_der * sq_magn_f_der / sq_magn_s_der;
    fact_sq_rad.sqrt()
}
