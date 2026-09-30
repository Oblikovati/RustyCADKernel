// Rust translation of BRepTools::Compare, OuterWire, Map3DEdges, IsReallyClosed,
// DetectClosedness and OriEdgeInFace (ModelingData/TKBRep/BRepTools/BRepTools.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Topological queries of BRepTools.
use super::uv_bounds::uv_bounds_of_wire;
use crate::brep::BRepTool;
use crate::topexp::Explorer;
use crate::topods::{self, Edge, Face, Shape, Vertex, Wire};
use crate::toptools::IndexedMapOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// `BRepTools::Compare(V1, V2)`: the same vertex, or points closer than the tolerance
/// of either vertex; `compare_vertices(&v1, &v2)?`.
pub fn compare_vertices(first: &Vertex, second: &Vertex) -> Result<bool, Failure> {
    if first.is_same(second) {
        return Ok(true);
    }
    let distance = BRepTool::pnt(first)?.distance(&BRepTool::pnt(second)?);
    Ok(distance <= BRepTool::vertex_tolerance(first)?
        || distance <= BRepTool::vertex_tolerance(second)?)
}

/// `BRepTools::Compare(E1, E2)`: `E1.IsSame(E2)`; `compare_edges(&e1, &e2)`.
pub fn compare_edges(first: &Edge, second: &Edge) -> bool {
    first.is_same(second)
}

/// `BRepTools::OuterWire(F)`: the first wire of the face, replaced by each later wire
/// whose UV box contains the current one (within `Precision::PConfusion()`); a null
/// wire for a face without wire; `outer_wire(&face)?`.
pub fn outer_wire(face: &Face) -> Result<Wire, Failure> {
    let mut wires = Explorer::new(face, ShapeEnum::Wire, ShapeEnum::Shape)?;
    let Some(first) = Iterator::next(&mut wires) else {
        return Ok(Wire::default());
    };
    let mut result = topods::wire(&first)?;
    let Some(second) = Iterator::next(&mut wires) else {
        return Ok(result);
    };
    let mut outer = uv_bounds_of_wire(face, &result)?;
    for candidate in std::iter::once(second).chain(wires) {
        let wire = topods::wire(&candidate)?;
        let (umin, umax, vmin, vmax) = uv_bounds_of_wire(face, &wire)?;
        let tolerance = precision::p_confusion();
        if umin - outer.0 <= tolerance
            && umax - outer.1 >= -tolerance
            && vmin - outer.2 <= tolerance
            && vmax - outer.3 >= -tolerance
        {
            result = wire;
            outer = (umin, umax, vmin, vmax);
        }
    }
    Ok(result)
}

/// `BRepTools::Map3DEdges(S, M)`: adds the non-degenerated edges of `shape` to `map`;
/// `map_3d_edges(&shape, &mut map)?`.
pub fn map_3d_edges(shape: &Shape, map: &mut IndexedMapOfShape) -> Result<(), Failure> {
    for edge in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if !BRepTool::degenerated(&topods::edge(&edge)?)? {
            map.add(edge);
        }
    }
    Ok(())
}

/// `BRepTools::IsReallyClosed(E, F)`: the edge is a seam of the face and is met twice
/// when exploring the face's edges; `is_really_closed(&edge, &face)?`.
pub fn is_really_closed(edge: &Edge, face: &Face) -> Result<bool, Failure> {
    if !BRepTool::is_closed_on_face(edge, face)? {
        return Ok(false);
    }
    let mut occurrences = 0;
    for current in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if current.is_same(edge) {
            occurrences += 1;
        }
    }
    Ok(occurrences == 2)
}

/// `BRepTools::DetectClosedness(F, Uclosed, Vclosed)`: for each really closed seam of
/// the face, compares the two p-curves at the first parameter: a larger U than V gap
/// makes the face U-closed, else V-closed; `let (u, v) = detect_closedness(&face)?;`.
pub fn detect_closedness(face: &Face) -> Result<(bool, bool), Failure> {
    let (mut u_closed, mut v_closed) = (false, false);
    for current in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = topods::edge(&current)?;
        if !(BRepTool::is_closed_on_face(&edge, face)? && is_really_closed(&edge, face)?) {
            continue;
        }
        if is_u_iso_seam(&edge, face)? {
            u_closed = true;
        } else {
            v_closed = true;
        }
    }
    Ok((u_closed, v_closed))
}

/// Whether the two p-curves of a seam differ more in U than in V at the first
/// parameter of the reversed edge's p-curve (the native code reads both at the
/// range of the second `CurveOnSurface` call).
fn is_u_iso_seam(edge: &Edge, face: &Face) -> Result<bool, Failure> {
    let forward = BRepTool::curve_on_face(edge, face)?;
    let reversed = BRepTool::curve_on_face(&topods::edge(&edge.reversed())?, face)?;
    let first = live_pcurve(&forward.pcurve)?.value(reversed.first)?;
    let second = live_pcurve(&reversed.pcurve)?.value(reversed.first)?;
    Ok((first.x() - second.x()).abs() > (first.y() - second.y()).abs())
}

/// The p-curve object; a null p-curve raises Standard_NullObject (a crash natively).
fn live_pcurve(
    pcurve: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
) -> Result<&dyn rustycad_tkg2d::geom2d::Geom2dCurve, Failure> {
    pcurve.get().ok_or_else(|| {
        crate::brep::errors::null_object("BRepTools::DetectClosedness: null p-curve on a seam")
    })
}

/// `BRepTools::OriEdgeInFace(E, F)`: the orientation of `edge` in the FORWARD face;
/// Standard_ConstructionError when the face does not contain it;
/// `ori_edge_in_face(&edge, &face)?`.
pub fn ori_edge_in_face(edge: &Edge, face: &Face) -> Result<Orientation, Failure> {
    let forward = face.oriented(Orientation::Forward);
    for current in Explorer::new(&forward, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if current.is_same(edge) {
            return Ok(current.orientation());
        }
    }
    Err(Failure::with_type(
        "Standard_ConstructionError",
        "BRepTools::OriEdgeInFace: the edge is not in the face; expected an edge of the face",
        "",
    ))
}
