// Rust translation of the file-static helpers TRIM_INFINIT_EDGE and TRIM_INFINIT_FACE of
// ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_DistanceSS.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The trimming of infinite edges and faces to the box of the other shape before an
//! edge-edge or edge-face extrema.
use crate::brep_bnd_lib::BRepBndLib;
use crate::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::exceptions::null_object;
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomRectangularTrimmedSurface, GeomSurface, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeomalgo::geom_api::{ProjectPointOnCurve, ProjectPointOnSurf};
use rustycad_tkgeombase::extrema::ExtAlgo;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point3;

/// The output of `TRIM_INFINIT_EDGE`: the trimmed edge and which edge it replaces.
#[derive(Default)]
pub(super) struct TrimmedEdge {
    pub(super) edge: Edge,
    pub(super) trim1: bool,
    pub(super) trim2: bool,
}

/// The eight corners of a box, in the native order
/// `(m,m,m) (m,M,m) (m,M,M) (m,m,M) (M,M,m) (M,M,M) (M,m,M) (M,m,m)`.
fn box_corners(bounds: &BoundingBox3) -> Result<[Point3; 8], Failure> {
    let [xmin, ymin, zmin, xmax, ymax, zmax] = bounds.get()?;
    Ok([
        Point3::new(xmin, ymin, zmin),
        Point3::new(xmin, ymax, zmin),
        Point3::new(xmin, ymax, zmax),
        Point3::new(xmin, ymin, zmax),
        Point3::new(xmax, ymax, zmin),
        Point3::new(xmax, ymax, zmax),
        Point3::new(xmax, ymin, zmax),
        Point3::new(xmax, ymin, zmin),
    ])
}

/// The native widening loop: each value below the minimum lowers it, else each value
/// above the maximum raises it (`if ... else if`).
fn widen(values: &[f64], mut min: f64, mut max: f64) -> (f64, f64) {
    for value in values {
        if *value < min {
            min = *value;
        } else if *value > max {
            max = *value;
        }
    }
    (min, max)
}

/// A range narrower than `eps` grows by `eps` on both sides.
fn at_least(min: f64, max: f64, eps: f64) -> (f64, f64) {
    if (min - max).abs() < eps {
        return (min - eps, max + eps);
    }
    (min, max)
}

/// The finite end of the half-infinite range `[first, last]`, as `(bUmin, Umin)` and
/// `(bUmax, Umax)`; neither for an infinite range at both ends.
fn finite_end(first: f64, last: f64) -> ((bool, f64), (bool, f64)) {
    if !precision::is_infinite(first) {
        return ((true, first), (false, 0.0));
    }
    if !precision::is_infinite(last) {
        return ((false, 0.0), (true, last));
    }
    ((false, 0.0), (false, 0.0))
}

/// `TRIM_INFINIT_EDGE(S1, S2, aResEdge, bIsTrim1, bIsTrim2)`: when the 3D curve of an
/// edge (the first one, else the second) is infinite at one end at least, the edge
/// made of that curve trimmed to the parameters of the projections of the corners of
/// the other edge's box (and to its finite end), widened by the resolution of three
/// times the edge tolerance when it is too short; the edge is S2 unchanged otherwise
/// and null for a degenerated edge.
pub(super) fn trim_infinite_edge(s1: &Edge, s2: &Edge) -> Result<TrimmedEdge, Failure> {
    let mut result = TrimmedEdge::default();
    if BRepTool::degenerated(s1)? || BRepTool::degenerated(s2)? {
        return Ok(result);
    }
    result.edge = s2.clone();
    let (curve1, first1, last1) = BRepTool::curve(s1)?;
    let (curve2, first2, last2) = BRepTool::curve(s2)?;
    let all_infinite = [first1, last1, first2, last2]
        .iter()
        .all(|u| precision::is_infinite(*u));
    if all_infinite {
        return Ok(result);
    }
    let half_infinite = |c: &Handle<dyn GeomCurve>, f: f64, l: f64| {
        !c.is_null() && (precision::is_infinite(f) || precision::is_infinite(l))
    };
    let (curve, ends, other, own) = if half_infinite(&curve1, first1, last1) {
        result.trim1 = true;
        (curve1, finite_end(first1, last1), s2, s1)
    } else if half_infinite(&curve2, first2, last2) {
        result.trim2 = true;
        (curve2, finite_end(first2, last2), s1, s2)
    } else {
        return Ok(result);
    };
    result.edge = trimmed_edge(&curve, ends, other, BRepTool::edge_tolerance(own)?)?;
    Ok(result)
}

/// The trimmed edge of `TRIM_INFINIT_EDGE` on `curve`, with the finite end `ends` of
/// its range, around the box of `other` and with the tolerance `tol` of its edge.
fn trimmed_edge(
    curve: &Handle<dyn GeomCurve>,
    ((b_umin, umin), (b_umax, umax)): ((bool, f64), (bool, f64)),
    other: &Edge,
    tol: f64,
) -> Result<Edge, Failure> {
    let mut edge_box = BoundingBox3::default();
    BRepBndLib::add(other, &mut edge_box, true)?;
    let corners: [Point3; 8] = box_corners(&edge_box)?;
    let mut projection = ProjectPointOnCurve::with_curve(corners[0], curve)?;
    let mut parameters: [f64; 8] = [projection.lower_distance_parameter()?; 8];
    for (parameter, corner) in parameters.iter_mut().zip(&corners).skip(1) {
        projection.perform(*corner)?;
        *parameter = projection.lower_distance_parameter()?;
    }
    let umin: f64 = if b_umin { umin } else { parameters[0] };
    let umax: f64 = if b_umax { umax } else { parameters[0] };
    let (umin, umax) = widen(&parameters, umin, umax);
    let eps_u: f64 = GeomAdaptorCurve::with_curve(curve)?.resolution(3.0 * tol)?;
    let (umin, umax) = at_least(umin, umax, eps_u);
    let trimmed: Handle<dyn GeomCurve> =
        Handle::new(GeomTrimmedCurve::new(curve, umin, umax, true, true)?).cast();
    BRepBuilderAPIMakeEdge::from_curve(&trimmed)?.edge()
}

/// `TRIM_INFINIT_FACE(S1, S2, aResFace, bIsInfinit)` for an edge and a face (in either
/// order): when the face is infinite (a natural restriction with an infinite surface
/// bound, or infinite UV bounds on all four sides), the face made of its surface
/// trimmed to the parameters of the projections of the corners of the edge box (and
/// to the finite surface bounds), widened by the resolutions of three times the face
/// tolerance when too narrow; `None` when the face is kept (or for a degenerated edge,
/// an edge box that is whole, other shape types).
///
/// Natively a failed projection of a corner leaves its parameters uninitialized
/// (docs/tktopalgo-progress.md); they are 0 here.
pub(super) fn trim_infinite_face(edge: &Edge, face: &Face) -> Result<Option<Face>, Failure> {
    if BRepTool::degenerated(edge)? {
        return Ok(None);
    }
    let surface: Handle<dyn GeomSurface> = BRepTool::surface(face)?;
    let bounds: Option<[(bool, f64); 4]> = infinite_face_bounds(face, &surface)?;
    let Some(bounds) = bounds else {
        return Ok(None);
    };
    let mut edge_box = BoundingBox3::default();
    BRepBndLib::add(edge, &mut edge_box, true)?;
    if edge_box.is_whole() {
        return Ok(None);
    }
    let (us, vs) = corner_parameters(&box_corners(&edge_box)?, &surface)?;
    let pick = |(known, value): (bool, f64), fallback: f64| if known { value } else { fallback };
    let (umin, umax) = widen(&us, pick(bounds[0], us[0]), pick(bounds[1], us[0]));
    let (vmin, vmax) = widen(&vs, pick(bounds[2], vs[0]), pick(bounds[3], vs[0]));
    let adaptor = GeomAdaptorSurface::with_surface(&surface)?;
    let tol: f64 = BRepTool::face_tolerance(face)?;
    let (umin, umax) = at_least(umin, umax, adaptor.u_resolution(3.0 * tol)?);
    let (vmin, vmax) = at_least(vmin, vmax, adaptor.v_resolution(3.0 * tol)?);
    let trimmed: Handle<dyn GeomSurface> = Handle::new(GeomRectangularTrimmedSurface::new(
        &surface,
        (umin, umax),
        (vmin, vmax),
        (true, true),
    )?)
    .cast();
    Ok(Some(
        BRepBuilderAPIMakeFace::from_surface(&trimmed, precision::confusion())?.face()?,
    ))
}

/// The known surface bounds `(bUmin, Umin), (bUmax, Umax), (bVmin, Vmin), (bVmax,
/// Vmax)` of an infinite face (`bIsTrim`), or `None` for a finite one. A natural
/// restriction knows its finite surface bounds; other faces know none.
fn infinite_face_bounds(
    face: &Face,
    surface: &Handle<dyn GeomSurface>,
) -> Result<Option<[(bool, f64); 4]>, Failure> {
    let unknown: [(bool, f64); 4] = [
        (false, real_last()),
        (false, real_first()),
        (false, real_last()),
        (false, real_first()),
    ];
    if !BRepTool::natural_restriction(face)? {
        let (u1, u2, v1, v2) = brep_tools::uv_bounds(face)?;
        let is_trim: bool = [u1, u2, v1, v2].iter().all(|u| precision::is_infinite(*u));
        return Ok(is_trim.then_some(unknown));
    }
    let geometry = surface
        .get()
        .ok_or_else(|| null_object("BRepExtrema_DistanceSS: TRIM_INFINIT_FACE on a face without surface"))?;
    let (u1, u2, v1, v2) = geometry.bounds();
    let mut known: [(bool, f64); 4] = unknown;
    let mut is_trim: bool = false;
    for (slot, value) in known.iter_mut().zip([u1, u2, v1, v2]) {
        if precision::is_infinite(value) {
            is_trim = true;
        } else {
            *slot = (true, value);
        }
    }
    Ok(is_trim.then_some(known))
}

/// The lower-distance parameters of the projections of the eight corners on the
/// surface, one Perform after the other on the same projector.
fn corner_parameters(
    corners: &[Point3; 8],
    surface: &Handle<dyn GeomSurface>,
) -> Result<([f64; 8], [f64; 8]), Failure> {
    let (mut us, mut vs) = ([0.0; 8], [0.0; 8]);
    let mut projection = ProjectPointOnSurf::with_surface(corners[0], surface, ExtAlgo::Grad)?;
    for (i, corner) in corners.iter().enumerate() {
        if i > 0 {
            projection.perform(*corner)?;
        }
        if projection.is_done() {
            (us[i], vs[i]) = projection.lower_distance_parameters()?;
        }
    }
    Ok((us, vs))
}
