// Rust translation of BRepGraph_Tool::CoEdge (ModelingData/TKBRep/BRepGraph/
// BRepGraph_Tool.hxx and .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_Tool::CoEdge`: the coedge orientation, links, seam pair and p-curve.
use super::{CoEdgeUsage, derived_state, edge, face};
use crate::brep_graph::reverse_iterator::CoEdgesOfEdge;
use crate::brep_graph::{BRepGraph, CoEdgeId, EdgeId, FaceId};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::geom::{GeomPlane, GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkmath::gp::Point2;

/// `Orientation(theGraph, theCoEdge)`: the orientation relative to the edge.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, CoEdgeId, tool};
/// # let _ = |graph: &BRepGraph| tool::co_edge::orientation(graph, CoEdgeId::new(0));
/// ```
pub fn orientation(graph: &BRepGraph, co_edge: CoEdgeId) -> Orientation {
    graph
        .inc_storage()
        .co_edge(co_edge)
        .orientation
        .orientation()
}

/// `IsReversed(theGraph, theCoEdge)`.
pub fn is_reversed(graph: &BRepGraph, co_edge: CoEdgeId) -> bool {
    orientation(graph, co_edge) == Orientation::Reversed
}

/// `EdgeOf(theGraph, theCoEdge)`: the stored edge id (not checked).
pub fn edge_of(graph: &BRepGraph, co_edge: CoEdgeId) -> EdgeId {
    graph.inc_storage().co_edge(co_edge).child_edge_id
}

/// `FaceOf(theGraph, theCoEdge)`: the stored face id (not checked).
pub fn face_of(graph: &BRepGraph, co_edge: CoEdgeId) -> FaceId {
    graph.inc_storage().co_edge(co_edge).face_id
}

/// `SeamPair(theGraph, theCoEdge)`: the first other active coedge of the same edge on
/// the same face with the opposite orientation, invalid otherwise.
pub fn seam_pair(graph: &BRepGraph, co_edge: CoEdgeId) -> CoEdgeId {
    let def = graph.inc_storage().co_edge(co_edge);
    if !def.child_edge_id.is_valid() || !def.face_id.is_valid() {
        return CoEdgeId::default();
    }
    let mut others = CoEdgesOfEdge::new(graph, graph.topo().edges().co_edges(def.child_edge_id));
    while others.more() {
        let (other, other_def) = (others.current_id(), others.definition());
        others.next();
        if other != co_edge
            && other_def.face_id == def.face_id
            && other_def.orientation != def.orientation
        {
            return other;
        }
    }
    CoEdgeId::default()
}

/// `IsSeam(theGraph, theCoEdge)`: a seam pair exists.
pub fn is_seam(graph: &BRepGraph, co_edge: CoEdgeId) -> bool {
    seam_pair(graph, co_edge).is_valid()
}

/// `HasPCurve(theGraph, theCoEdge)`: an active p-curve use.
pub fn has_pcurve(graph: &BRepGraph, co_edge: CoEdgeId) -> bool {
    let storage = graph.inc_storage();
    let rep = storage.co_edge(co_edge).curve2d_rep_id;
    rep.is_valid() && rep.is_valid_for(storage.nb_co_edge_curves2d()) && !storage.is_removed(rep)
}

/// `SameParameter(theGraph, theCoEdge)`: derived from the current geometry; false for
/// an inactive coedge.
pub fn same_parameter(graph: &BRepGraph, co_edge: CoEdgeId) -> Result<bool, Failure> {
    derived_state::co_edge_same_parameter(graph, co_edge)
}

/// `SameRange(theGraph, theCoEdge)`: derived from the current geometry; false for an
/// inactive coedge.
pub fn same_range(graph: &BRepGraph, co_edge: CoEdgeId) -> bool {
    derived_state::co_edge_same_range(graph, co_edge)
}

/// `PCurve(theGraph, theCoEdge)`: the p-curve of an active curve use, null otherwise.
pub fn pcurve(graph: &BRepGraph, co_edge: CoEdgeId) -> Handle<dyn Geom2dCurve> {
    if !has_pcurve(graph, co_edge) {
        return Handle::default();
    }
    let storage = graph.inc_storage();
    let rep = storage.co_edge(co_edge).curve2d_rep_id;
    storage.co_edge_curve2d_rep(rep).curve.clone()
}

/// `Range(theGraph, theCoEdge)`: the p-curve range of the active curve use, (0, 0)
/// otherwise.
pub fn range(graph: &BRepGraph, co_edge: CoEdgeId) -> (f64, f64) {
    if !has_pcurve(graph, co_edge) {
        return (0.0, 0.0);
    }
    let storage = graph.inc_storage();
    let record = storage.co_edge_curve2d_rep(storage.co_edge(co_edge).curve2d_rep_id);
    (record.param_first, record.param_last)
}

/// `UVPoints(theGraph, theCoEdge)`: the p-curve points at its range ends, the origins
/// without an active non-null p-curve.
pub fn uv_points(graph: &BRepGraph, co_edge: CoEdgeId) -> Result<(Point2, Point2), Failure> {
    let curve = pcurve(graph, co_edge);
    let Some(curve) = curve.get() else {
        return Ok((Point2::default(), Point2::default()));
    };
    let (first, last) = range(graph, co_edge);
    Ok((curve.eval_d0(first)?, curve.eval_d0(last)?))
}

/// `PCurveAdaptor(theGraph, CoEdgeUsage)`: the adaptor of the usage's coedge.
pub fn pcurve_adaptor_usage(
    graph: &BRepGraph,
    usage: &CoEdgeUsage,
) -> Result<Geom2dAdaptorCurve, Failure> {
    pcurve_adaptor(graph, usage.def_id)
}

/// `PCurveAdaptor(theGraph, theCoEdge)`: the stored p-curve over its range; without
/// one, on a planar face (a plane or a trimmed plane), the edge's 3D curve projected
/// on the plane over the edge range (`CurveOnPlane`); an empty adaptor otherwise.
pub fn pcurve_adaptor(graph: &BRepGraph, co_edge: CoEdgeId) -> Result<Geom2dAdaptorCurve, Failure> {
    if !co_edge.is_valid() {
        return Ok(Geom2dAdaptorCurve::default());
    }
    let stored = pcurve(graph, co_edge);
    if stored.is_some() {
        let (first, last) = range(graph, co_edge);
        return Geom2dAdaptorCurve::with_range(&stored, first, last);
    }
    projected_adaptor(graph, co_edge)
}

// The planar fallback of `PCurveAdaptor`.
fn projected_adaptor(graph: &BRepGraph, co_edge: CoEdgeId) -> Result<Geom2dAdaptorCurve, Failure> {
    let def = graph.inc_storage().co_edge(co_edge);
    if !def.face_id.is_valid() || !def.child_edge_id.is_valid() {
        return Ok(Geom2dAdaptorCurve::default());
    }
    let surface = face::surface(graph, def.face_id);
    if surface.is_null() || !edge::has_curve(graph, def.child_edge_id) {
        return Ok(Geom2dAdaptorCurve::default());
    }
    let Some(plane) = plane_basis(&surface) else {
        return Ok(Geom2dAdaptorCurve::default());
    };
    let curve3d = edge::curve(graph, def.child_edge_id);
    if curve3d.is_null() {
        return Ok(Geom2dAdaptorCurve::default());
    }
    let (first, last) = edge::range(graph, def.child_edge_id);
    let projected = geom_proj_lib::curve2d_range(&curve3d, (first, last), &plane)?;
    if projected.is_null() {
        return Ok(Geom2dAdaptorCurve::default());
    }
    Geom2dAdaptorCurve::with_range(&projected, first, last)
}

// The plane of a plane or of a rectangular trimmed plane (the trimmed basis is not
// unwrapped further, as natively).
fn plane_basis(surface: &Handle<dyn GeomSurface>) -> Option<Handle<dyn GeomSurface>> {
    if surface.cast::<GeomPlane>().is_some() {
        return Some(surface.clone());
    }
    let trimmed = surface.cast::<GeomRectangularTrimmedSurface>();
    let basis = trimmed.get()?.basis_surface();
    basis.cast::<GeomPlane>().is_some().then_some(basis)
}
