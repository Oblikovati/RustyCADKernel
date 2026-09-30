// Rust translation of BRepGraph_Tool::Edge::Curve(graph, CoEdgeUsage), CurveAdaptor and
// CurveOnSurface (ModelingData/TKBRep/BRepGraph/BRepGraph_Tool.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The located 3D curve, the transformed curve adaptors and the curve on surface of
//! `BRepGraph_Tool::Edge`, re-exported by [`edge`](super::edge).
use super::CoEdgeUsage;
use crate::brep_graph::{BRepGraph, EdgeId, FaceId};
use crate::brep_graph_inc::{CoEdgeCurve2DRep, CoEdgeDef, EdgeCurve3DRep, FaceSurfaceRep};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorSurface, GeomAdaptorTransformedCurve};
use rustycad_tkmath::gp::Transform3;
use rustycad_tkmath::toploc::Location;

// The active 3D curve record of an edge in range.
fn active_curve_record(graph: &BRepGraph, edge: EdgeId) -> Option<&EdgeCurve3DRep> {
    let storage = graph.inc_storage();
    let rep = storage.edge(edge).curve3d_rep_id;
    let active =
        rep.is_valid() && rep.is_valid_for(storage.nb_edge_curves3d()) && !storage.is_removed(rep);
    active.then(|| storage.edge_curve3d_rep(rep))
}

// The active p-curve record of a coedge.
fn active_pcurve_record<'g>(
    graph: &'g BRepGraph,
    co_edge: &CoEdgeDef,
) -> Option<&'g CoEdgeCurve2DRep> {
    let storage = graph.inc_storage();
    let rep = co_edge.curve2d_rep_id;
    let active = rep.is_valid()
        && rep.is_valid_for(storage.nb_co_edge_curves2d())
        && !storage.is_removed(rep);
    active.then(|| storage.co_edge_curve2d_rep(rep))
}

// The active surface record of a face in range.
fn active_surface_record(graph: &BRepGraph, face: FaceId) -> Option<&FaceSurfaceRep> {
    let storage = graph.inc_storage();
    let rep = storage.face(face).surface_rep_id;
    let active =
        rep.is_valid() && rep.is_valid_for(storage.nb_face_surfaces()) && !storage.is_removed(rep);
    active.then(|| storage.face_surface_rep(rep))
}

// The usage placement: identity for an identity location.
fn usage_trsf(location: &Location) -> Transform3 {
    if location.is_identity() {
        Transform3::default()
    } else {
        location.transformation()
    }
}

/// `Curve(theGraph, CoEdgeUsage)`: the 3D curve of the usage's edge, transformed by the
/// usage location (a copy; the stored curve for an identity location); null without an
/// active curve use.
///
/// ```
/// use rustycad_tkbrep::brep_graph::tool::{self, CoEdgeUsage};
/// use rustycad_tkbrep::brep_graph::{BRepGraph, CoEdgeId};
/// # let _ = |graph: &BRepGraph| tool::edge::curve_usage(graph, &CoEdgeUsage::new(CoEdgeId::new(0)));
/// ```
pub fn curve_usage(
    graph: &BRepGraph,
    usage: &CoEdgeUsage,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let co_edge = graph.inc_storage().co_edge(usage.def_id);
    let Some(record) = active_curve_record(graph, co_edge.child_edge_id) else {
        return Ok(Handle::default());
    };
    let Some(curve) = record.curve.get() else {
        return Ok(record.curve.clone());
    };
    if usage.location.is_identity() {
        return Ok(record.curve.clone());
    }
    let copy = curve.transformed(usage.location.transformation())?;
    Ok(copy.cast::<dyn GeomCurve>())
}

/// `CurveAdaptor(theGraph, theEdge)`: the 3D curve over its use range with the identity
/// placement; an empty adaptor without an active curve.
pub fn curve_adaptor(
    graph: &BRepGraph,
    edge: EdgeId,
) -> Result<GeomAdaptorTransformedCurve, Failure> {
    let Some(record) = active_curve_record(graph, edge) else {
        return Ok(GeomAdaptorTransformedCurve::default());
    };
    if record.curve.is_null() {
        return Ok(GeomAdaptorTransformedCurve::default());
    }
    GeomAdaptorTransformedCurve::with_range(
        &record.curve,
        record.param_first,
        record.param_last,
        Transform3::default(),
    )
}

/// `CurveAdaptor(theGraph, CoEdgeUsage)`: the 3D curve of the usage's edge with the
/// usage placement, else the p-curve on the face surface; an empty adaptor otherwise.
pub fn curve_adaptor_usage(
    graph: &BRepGraph,
    usage: &CoEdgeUsage,
) -> Result<GeomAdaptorTransformedCurve, Failure> {
    let co_edge = graph.inc_storage().co_edge(usage.def_id);
    let trsf = usage_trsf(&usage.location);
    if let Some(record) = active_curve_record(graph, co_edge.child_edge_id)
        && record.curve.is_some()
    {
        return GeomAdaptorTransformedCurve::with_range(
            &record.curve,
            record.param_first,
            record.param_last,
            trsf,
        );
    }
    let fallback = pcurve_fallback(graph, co_edge)?;
    let result = GeomAdaptorTransformedCurve::default();
    if let Some(curve_on_surface) = fallback {
        result.set_trsf(trsf);
        result.load_curve_on_surface(&curve_on_surface);
    }
    Ok(result)
}

// The fallback of `CurveAdaptor(CoEdgeUsage)`: the curve on surface of an active
// p-curve on the active surface of the coedge's face (the face must be in range).
fn pcurve_fallback(
    graph: &BRepGraph,
    co_edge: &CoEdgeDef,
) -> Result<Option<Handle<Adaptor3dCurveOnSurface>>, Failure> {
    let Some(pcurve) = active_pcurve_record(graph, co_edge) else {
        return Ok(None);
    };
    if !co_edge.face_id.is_valid() {
        return Ok(None);
    }
    let Some(surface) = active_surface_record(graph, co_edge.face_id) else {
        return Ok(None);
    };
    if pcurve.curve.is_null() || surface.surface.is_null() {
        return Ok(None);
    }
    Ok(Some(curve_on_surface_of(pcurve, surface)?))
}

// `new Adaptor3d_CurveOnSurface(new Geom2dAdaptor_Curve(PC, First, Last),
// new GeomAdaptor_Surface(S))`.
fn curve_on_surface_of(
    pcurve: &CoEdgeCurve2DRep,
    surface: &FaceSurfaceRep,
) -> Result<Handle<Adaptor3dCurveOnSurface>, Failure> {
    let curve2d: Handle<dyn Adaptor2dCurve2d> = Handle::new(Geom2dAdaptorCurve::with_range(
        &pcurve.curve,
        pcurve.param_first,
        pcurve.param_last,
    )?)
    .cast();
    let adapted: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&surface.surface)?).cast();
    Ok(Handle::new(Adaptor3dCurveOnSurface::with_curve(
        &curve2d, &adapted,
    )?))
}

/// `CurveOnSurface(theGraph, CoEdgeUsage, theFace)`: the usage's p-curve on the face
/// surface (the face given, not the coedge's); null without both active uses.
pub fn curve_on_surface(
    graph: &BRepGraph,
    usage: &CoEdgeUsage,
    face: FaceId,
) -> Result<Handle<Adaptor3dCurveOnSurface>, Failure> {
    let co_edge = graph.inc_storage().co_edge(usage.def_id);
    let pcurve = active_pcurve_record(graph, co_edge);
    let surface = active_surface_record(graph, face);
    let (Some(pcurve), Some(surface)) = (pcurve, surface) else {
        return Ok(Handle::default());
    };
    if pcurve.curve.is_null() || surface.surface.is_null() {
        return Ok(Handle::default());
    }
    curve_on_surface_of(pcurve, surface)
}
