// Rust translation of ExtractedWire, FaceBuildData, findExtractedEdgeByExplorerEdge,
// orderExtractedWire, hasInfiniteRequiredBound and extractFaceData of
// ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Populate.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The per-face extraction of population: the independent work of the parallel phase.
use super::extract::{ExtractedEdge, FacePCurveContext, extract_edge_in_face};
use super::locations::{located_surface, located_triangulation};
use crate::brep::BRepTool;
use crate::brep_graph::FaceId;
use crate::brep_graph_inc::BitFlags;
use crate::brep_tools::WireExplorer;
use crate::topods::{self, Edge, Face, ShapeIterator, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::poly::{Triangulation, mesh_purpose};
use rustycad_tkmath::toploc::Location;

/// `ExtractedWire`: a wire use and its edge uses, in wire-explorer order when possible.
#[derive(Clone, Default)]
pub(super) struct ExtractedWire {
    pub(super) shape: Wire,
    pub(super) edges: Vec<ExtractedEdge>,
    pub(super) definition_location: Location,
    pub(super) is_generated: bool,
    pub(super) has_partial_explorer_order: bool,
}

/// `FaceBuildData`: a queued face and what its extraction found.
#[derive(Clone, Default)]
pub(super) struct FaceBuildData {
    pub(super) face: Face,
    pub(super) face_id: FaceId,
    pub(super) surface: Handle<dyn GeomSurface>,
    pub(super) raw_triangulation: Handle<Triangulation>,
    pub(super) active_triangulation: Handle<Triangulation>,
    pub(super) definition_location: Location,
    pub(super) tolerance: f64,
    pub(super) unsupported_natural_boundary: bool,
    pub(super) needs_synthesis: bool,
    pub(super) wires: Vec<ExtractedWire>,
}

// `findExtractedEdgeByExplorerEdge`: the first unused edge use that is the same edge
// (and has the same orientation when asked).
fn find_by_explorer_edge(
    wire: &ExtractedWire,
    used: &BitFlags,
    edge: &Edge,
    same_ori: bool,
) -> Option<usize> {
    (0..wire.edges.len()).find(|index| {
        let data = &wire.edges[*index];
        !used.test(*index as u32)
            && data.shape.is_same(edge)
            && (!same_ori || data.orientation_in_wire == edge.orientation())
    })
}

/// `orderExtractedWire(theWireData, theFace)`: reorders the edge uses in the order of
/// `BRepTools_WireExplorer` (on the face when given), the unmatched ones appended in
/// input order; false when the explorer order is incomplete (or matched nothing, which
/// keeps the input order).
pub(super) fn order_extracted_wire(
    wire: &mut ExtractedWire,
    face: Option<&Face>,
) -> Result<bool, Failure> {
    let count = wire.edges.len();
    if count < 2 {
        return Ok(true);
    }
    let mut explorer = match face {
        Some(face) if !face.is_null() => WireExplorer::with_wire_on_face(&wire.shape, face)?,
        _ => WireExplorer::with_wire(&wire.shape)?,
    };
    let mut used = BitFlags::new();
    used.resize(count);
    let mut order: Vec<usize> = Vec::with_capacity(count);
    while explorer.more() {
        let current = explorer.current().clone();
        let matched = find_by_explorer_edge(wire, &used, &current, true)
            .or_else(|| find_by_explorer_edge(wire, &used, &current, false));
        if let Some(index) = matched {
            used.set(index as u32);
            order.push(index);
        }
        explorer.next()?;
    }
    if order.is_empty() {
        return Ok(false);
    }
    let complete = order.len() == count;
    order.extend((0..count).filter(|index| !used.test(*index as u32)));
    wire.edges = order
        .iter()
        .map(|index| wire.edges[*index].clone())
        .collect();
    Ok(complete)
}

/// `hasInfiniteRequiredBound`: an infinite bound in a non-periodic direction.
pub(super) fn has_infinite_required_bound(surface: &dyn GeomSurface) -> bool {
    let (u1, u2, v1, v2) = surface.bounds();
    let infinite = |a: f64, b: f64| precision::is_infinite(a) || precision::is_infinite(b);
    (!surface.is_u_periodic() && infinite(u1, u2)) || (!surface.is_v_periodic() && infinite(v1, v2))
}

/// `extractFaceData(theData)`: the definition-frame surface and triangulation, the
/// tolerance, the wires of the FORWARD face with their edge uses in explorer order, and
/// the natural-restriction flags of a face without wires.
pub(super) fn extract_face_data(data: &mut FaceBuildData) -> Result<(), Failure> {
    let face = data.face.clone();
    let face_parent = (&data.definition_location * &face.location().inverted()?)?;
    let (raw_surface, surface_location) = BRepTool::surface_with_location(&face)?;
    data.surface = located_surface(&raw_surface, &(&face_parent * &surface_location)?)?;
    let (raw_triangulation, triangulation_location) =
        BRepTool::triangulation(&face, mesh_purpose::NONE)?;
    data.active_triangulation = located_triangulation(
        &raw_triangulation,
        &(&face_parent * &triangulation_location)?,
    )?;
    data.raw_triangulation = raw_triangulation;
    data.tolerance = BRepTool::face_tolerance(&face)?;
    let forward = topods::Face::unchecked(face.oriented(Orientation::Forward));
    let context = FacePCurveContext {
        raw_surface: raw_surface.clone(),
        surface_location,
        triangulation_location,
    };
    for child in ShapeIterator::new(&forward, false, false) {
        if child.shape_type()? != ShapeEnum::Wire {
            continue;
        }
        let wire = topods::Wire::unchecked(child);
        let explorer_face = raw_surface.is_some().then_some(&forward);
        let extracted = extract_wire_in_face(data, &wire, &forward, &context, explorer_face)?;
        data.wires.push(extracted);
    }
    flag_natural_boundary(data, &face)
}

// One wire of `extractFaceData`: its edge uses in the face, then the explorer order.
fn extract_wire_in_face(
    data: &FaceBuildData,
    wire: &Wire,
    forward: &Face,
    context: &FacePCurveContext,
    explorer_face: Option<&Face>,
) -> Result<ExtractedWire, Failure> {
    let mut extracted = ExtractedWire {
        shape: wire.clone(),
        definition_location: (&data.definition_location * wire.location())?,
        ..ExtractedWire::default()
    };
    for child in ShapeIterator::new(wire, false, false) {
        if child.shape_type()? != ShapeEnum::Edge {
            continue;
        }
        let edge = topods::Edge::unchecked(child);
        let location = extracted.definition_location.clone();
        let edge_data =
            extract_edge_in_face(&edge, forward, &data.raw_triangulation, context, &location)?;
        extracted.edges.push(edge_data);
    }
    extracted.has_partial_explorer_order = !order_extracted_wire(&mut extracted, explorer_face)?;
    Ok(extracted)
}

// A face without wires and with the natural-restriction flag needs a synthesised
// boundary, unless its surface is missing or unbounded (a build warning).
fn flag_natural_boundary(data: &mut FaceBuildData, face: &Face) -> Result<(), Failure> {
    if !data.wires.is_empty() || !BRepTool::natural_restriction(face)? {
        return Ok(());
    }
    match data.surface.get() {
        Some(surface) if !has_infinite_required_bound(surface) => data.needs_synthesis = true,
        _ => data.unsupported_natural_boundary = true,
    }
    Ok(())
}
