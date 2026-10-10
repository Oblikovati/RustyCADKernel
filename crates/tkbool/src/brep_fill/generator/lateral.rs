// Rust translation of the lateral edges of BRepFill_Generator::Perform
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Generator.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The lateral edges of a ruled face (`Edge3` at the first vertices, `Edge4` at the
//! last ones), shared with the neighbouring face through the vertex map.
use super::kpart_detect::KPART_NONE;
use super::ruled_face::FaceSections;
use super::section_pair::SectionPair;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{one_based, put};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;

/// The surface of a face, its particular case code and its U bounds and V range.
pub(super) struct FaceFrame<'a> {
    pub(super) surface: &'a Handle<dyn GeomSurface>,
    pub(super) itype: i32,
    pub(super) u_first: f64,
    pub(super) u_last: f64,
    pub(super) v_first: f64,
    pub(super) v_last: f64,
    /// The range of the lateral edges: the V bounds in the general case, `[0, 1]`
    /// for a particular case.
    pub(super) range: (f64, f64),
}

/// A Bezier segment between two points (`Geom_BezierCurve(Extremities)`).
pub(super) fn segment(start: Point3, end: Point3) -> Result<Handle<dyn GeomCurve>, Failure> {
    let mut extremities = one_based::<Point3>(2)?;
    put(&mut extremities, 1, start)?;
    put(&mut extremities, 2, end)?;
    Ok(Handle::new(GeomBezierCurve::new(&extremities)?).cast::<dyn GeomCurve>())
}

/// `Edge3`: the reversed lateral edge of the previous face at `Vf_toMap`, or a new
/// edge from `V1f` to `V2f` (degenerated when they are the same; the iso U = u_first in
/// the general case, a segment otherwise), reversed and bound at `Vf_toMap`.
pub(super) fn first_lateral(
    pair: &mut SectionPair,
    s: &mut FaceSections,
    frame: &FaceFrame,
) -> Result<Edge, Failure> {
    if let Some(bound) = pair.lateral_edges.seek(&s.vf_to_map) {
        return topods::edge(&bound.reversed());
    }
    let mut edge: Edge = new_lateral(&s.v1f, &s.v2f, frame, frame.u_first)?;
    bound_lateral(&mut edge, &mut s.v1f, &mut s.v2f, frame.range)?;
    edge.reverse();
    pair.lateral_edges
        .bind(s.vf_to_map.as_ref().clone(), edge.as_ref().clone());
    Ok(edge)
}

/// `Edge4`: the reversed lateral edge bound at `Vl_toMap` when it joins `V1l` to
/// `V2l` (`CommonEdge`), or a new edge from `V1l` to `V2l` bound at `Vl_toMap`.
pub(super) fn last_lateral(
    pair: &mut SectionPair,
    s: &mut FaceSections,
    frame: &FaceFrame,
) -> Result<Edge, Failure> {
    if let Some(bound) = pair.lateral_edges.seek(&s.vl_to_map) {
        let common: Edge = topods::edge(&bound.reversed())?;
        let (v1, v2) = topexp::vertices(&common, false);
        if v1.is_same(&s.v1l) && v2.is_same(&s.v2l) {
            return Ok(common);
        }
    }
    let mut edge: Edge = new_lateral(&s.v1l, &s.v2l, frame, frame.u_last)?;
    bound_lateral(&mut edge, &mut s.v1l, &mut s.v2l, frame.range)?;
    pair.lateral_edges
        .bind(s.vl_to_map.as_ref().clone(), edge.as_ref().clone());
    Ok(edge)
}

/// A new lateral edge between two section vertices: degenerated for one vertex, the
/// U iso at `u` of the general surface, or the segment between the vertices.
fn new_lateral(start: &Vertex, end: &Vertex, frame: &FaceFrame, u: f64) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    if start.is_same(end) {
        builder.make_edge(&mut edge)?;
        builder.degenerated(&edge, true)?;
        return Ok(edge);
    }
    let curve: Handle<dyn GeomCurve> = if frame.itype == KPART_NONE {
        // The general case: the lateral edge is the iso U of the surface.
        match frame.surface.get() {
            Some(surface) => surface.u_iso(u)?,
            None => Handle::default(),
        }
    } else {
        // A particular case: the 3d curve is the segment between the vertices.
        segment(BRepTool::pnt(start)?, BRepTool::pnt(end)?)?
    };
    builder.make_edge_with_curve(&mut edge, &curve, precision::confusion())?;
    Ok(edge)
}

/// Adds the vertices (the start FORWARD, the end REVERSED; the orientations stay on
/// the caller's vertices, as the native locals) and the range to a lateral edge.
fn bound_lateral(
    edge: &mut Edge,
    start: &mut Vertex,
    end: &mut Vertex,
    range: (f64, f64),
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    start.set_orientation(Orientation::Forward);
    builder.add(edge, start)?;
    end.set_orientation(Orientation::Reversed);
    builder.add(edge, end)?;
    builder.range(edge, range.0, range.1, false)
}
