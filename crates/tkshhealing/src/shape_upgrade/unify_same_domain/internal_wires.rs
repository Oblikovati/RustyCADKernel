//! Vertex-to-internal-edge map construction and wire assembly for same-domain face merges.
use super::face_wires;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Builder, Edge, Face, Shape, Wire};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Adds the original internal wires of both faces to the merged face once each.
pub(super) fn add_internal_wires(
    merged: &mut Shape,
    first: &Face,
    second: &Face,
) -> Result<(), Failure> {
    let edges = internal_edges(first, second)?;
    let mut vertex_edges = internal_vertex_edge_map(&edges)?;
    for wire in rebuild_internal_wires(edges, &mut vertex_edges)? {
        Builder.add(merged, &wire.into_shape())?;
    }
    Ok(())
}

/// Walks every unconsumed internal edge component into one wire.
fn rebuild_internal_wires(
    edges: Vec<Edge>,
    vertex_edges: &mut IndexedDataMapOfShapeListOfShape,
) -> Result<Vec<Wire>, Failure> {
    let mut pending = edges.clone();
    let mut wires = Vec::new();
    while !pending.is_empty() {
        let first_edge = pending.remove(0);
        remove_internal_edge_from_map(&first_edge, vertex_edges);
        wires.push(build_internal_wire(
            first_edge,
            &edges,
            &mut pending,
            vertex_edges,
        )?);
    }
    Ok(wires)
}

/// Starts one component and follows both free ends, bounded by its total edge count.
fn build_internal_wire(
    first: Edge,
    all_edges: &[Edge],
    pending: &mut Vec<Edge>,
    vertex_edges: &mut IndexedDataMapOfShapeListOfShape,
) -> Result<Wire, Failure> {
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    Builder.add(&mut wire, &first.clone().into_shape())?;
    let (start, end) = topexp::vertices(&first, false);
    let mut ends = [start, end];
    let mut last_edges = [first.clone(), first];
    grow_internal_wire(
        &mut wire,
        all_edges,
        &mut ends,
        &mut last_edges,
        pending,
        vertex_edges,
    )?;
    Ok(wire)
}

/// Extends the current internal wire until neither endpoint has an unused edge.
fn grow_internal_wire(
    wire: &mut Wire,
    all_edges: &[Edge],
    ends: &mut [topods::Vertex; 2],
    last_edges: &mut [Edge; 2],
    pending: &mut Vec<Edge>,
    vertex_edges: &mut IndexedDataMapOfShapeListOfShape,
) -> Result<(), Failure> {
    for _ in 0..all_edges.len() {
        if !append_next_internal_edge(wire, ends, last_edges, pending, vertex_edges)? {
            break;
        }
    }
    Ok(())
}

/// Appends one neighboring edge and removes it from both pending and vertex maps.
fn append_next_internal_edge(
    wire: &mut Wire,
    ends: &mut [topods::Vertex; 2],
    last_edges: &mut [Edge; 2],
    pending: &mut Vec<Edge>,
    vertex_edges: &mut IndexedDataMapOfShapeListOfShape,
) -> Result<bool, Failure> {
    let Some((index, edge)) = next_internal_edge(ends, last_edges, vertex_edges) else {
        return Ok(false);
    };
    pending.retain(|known| !known.is_same(&edge));
    remove_internal_edge_from_map(&edge, vertex_edges);
    Builder.add(wire, &edge.clone().into_shape())?;
    ends[index] = other_edge_vertex(&edge, &ends[index]);
    last_edges[index] = edge;
    Ok(true)
}

/// Collects unique edge occurrences from the direct internal wires of both faces.
fn internal_edges(first: &Face, second: &Face) -> Result<Vec<Edge>, Failure> {
    let mut edges = Vec::new();
    for face in [first, second] {
        for wire in face_wires(face)? {
            if wire.orientation() != rustycad_tkg3d::top_abs::Orientation::Internal {
                continue;
            }
            for edge in Explorer::new(&wire.into_shape(), ShapeEnum::Edge, ShapeEnum::Shape)? {
                let mut edge = topods::edge(&edge)?;
                edge.set_orientation(rustycad_tkg3d::top_abs::Orientation::Internal);
                if !edges.iter().any(|known: &Edge| known.is_same(&edge)) {
                    edges.push(edge);
                }
            }
        }
    }
    Ok(edges)
}

/// Builds OCCT's vertex-to-internal-edge adjacency map before wire assembly.
fn internal_vertex_edge_map(edges: &[Edge]) -> Result<IndexedDataMapOfShapeListOfShape, Failure> {
    let mut map = IndexedDataMapOfShapeListOfShape::default();
    for edge in edges {
        let shape = edge.clone().into_shape();
        topexp::map_shapes_and_ancestors(&shape, ShapeEnum::Vertex, ShapeEnum::Edge, &mut map)?;
    }
    Ok(map)
}

/// Removes a consumed edge from both endpoint lists, preventing stale traversal entries.
fn remove_internal_edge_from_map(edge: &Edge, map: &mut IndexedDataMapOfShapeListOfShape) {
    let (first, last) = topexp::vertices(edge, false);
    for vertex in [first, last] {
        if let Some(edges) = map.change_seek(&vertex.into_shape()) {
            edges.remove_value(&edge.clone().into_shape());
        }
    }
}

/// Chooses the next unconsumed edge at either open end of the current wire.
fn next_internal_edge(
    ends: &[topods::Vertex; 2],
    last_edges: &[Edge; 2],
    map: &IndexedDataMapOfShapeListOfShape,
) -> Option<(usize, Edge)> {
    for (index, vertex) in ends.iter().enumerate() {
        let Some(incident) = map.seek(&vertex.clone().into_shape()) else {
            continue;
        };
        if let Some(edge) = incident
            .iter()
            .find(|edge| !edge.is_same(&last_edges[index].clone().into_shape()))
        {
            return Some((index, topods::edge(edge).ok()?));
        }
    }
    None
}

/// Returns the opposite endpoint of an edge incident to `vertex`.
fn other_edge_vertex(edge: &Edge, vertex: &topods::Vertex) -> topods::Vertex {
    let (first, last) = topexp::vertices(edge, false);
    if first.is_same(vertex) { last } else { first }
}
