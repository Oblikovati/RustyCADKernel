// Rust translation of BRepLib::BuildPCurveForEdgeOnPlane (BRepLib_1.cxx) and
// BRepLib::BuildPCurveForEdgesOnPlane (BRepLib.hxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib::BuildPCurveForEdgeOnPlane`: the planar projection of `BRep_Tool` stored as
//! the p-curve of an edge on a plane.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;

/// `BRepLib::BuildPCurveForEdgeOnPlane(theE, theF)`: stores the p-curve of the edge on
/// the face when it is not stored and the face surface is a plane (the projection of
/// `BRep_Tool::CurveOnSurface`), with the edge tolerance;
/// `brep_lib::build_pcurve_for_edge_on_plane(&edge, &face)?`.
pub fn build_pcurve_for_edge_on_plane(edge: &Edge, face: &Face) -> Result<(), Failure> {
    let (pcurve, to_update) = pcurve_for_edge_on_plane(edge, face)?;
    if to_update {
        let tol_e: f64 = BRepTool::edge_tolerance(edge)?;
        BRepBuilder::default().update_edge_pcurve_on_face(edge, &pcurve, face, tol_e)?;
    }
    Ok(())
}

/// `BRepLib::BuildPCurveForEdgeOnPlane(theE, theF, aC2D, bToUpdate)`: the p-curve of
/// the edge on the face and whether it was computed (not stored) and exists; the edge
/// is not updated; `brep_lib::pcurve_for_edge_on_plane(&edge, &face)?`.
pub fn pcurve_for_edge_on_plane(
    edge: &Edge,
    face: &Face,
) -> Result<(Handle<dyn Geom2dCurve>, bool), Failure> {
    let found = BRepTool::curve_on_face(edge, face)?;
    let to_update: bool = !found.is_stored && found.pcurve.is_some();
    Ok((found.pcurve, to_update))
}

/// `BRepLib::BuildPCurveForEdgesOnPlane(theLE, theF)`, the template over a container
/// of shapes: `build_pcurve_for_edge_on_plane` on each non-null edge; Standard_TypeMismatch
/// for a shape that is not an edge; `brep_lib::build_pcurve_for_edges_on_plane(&edges, &face)?`.
pub fn build_pcurve_for_edges_on_plane<'a>(
    edges: impl IntoIterator<Item = &'a Shape>,
    face: &Face,
) -> Result<(), Failure> {
    for shape in edges {
        let edge: Edge = topods::edge(shape)?;
        if !edge.is_null() {
            build_pcurve_for_edge_on_plane(&edge, face)?;
        }
    }
    Ok(())
}
