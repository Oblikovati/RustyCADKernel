//! Repair pcurve endpoints according to topological adjacency.
use super::super::Deflection;
use super::*;
use crate::imesh_data::WireHandle;
use rustycad_tkernel::ncollection::Map;
pub(super) fn fix(face: &FaceHandle, parameters: &MeshParameters) -> Result<(), Failure> {
    for index in 0..face.wires_nb() {
        let wire = face.get_wire(index)?;
        Deflection::compute_wire(&wire, parameters)?;
        for edge in 0..wire.edges_nb() {
            fix_edge(face, &wire, edge)?;
        }
    }
    Deflection::compute_face(face, parameters)
}
fn fix_edge(face: &FaceHandle, wire: &WireHandle, index: i32) -> Result<(), Failure> {
    let previous = (index + wire.edges_nb() - 1) % wire.edges_nb();
    let next = (index + 1) % wire.edges_nb();
    let prev_edge = wire.get_edge(previous)?.upgrade();
    let curr_edge = wire.get_edge(index)?.upgrade();
    let next_edge = wire.get_edge(next)?.upgrade();
    let connected = !vertex::common(&curr_edge, &next_edge)?.is_null()
        && !vertex::common(&prev_edge, &curr_edge)?.is_null();
    if connected {
        connect_edges(face, wire, [previous, index, next])?;
    }
    if !connected || curr_edge.status().is_set(Status::Outdated) {
        face.status().set_status(Status::Outdated);
    }
    if !connected {
        wire.status().set_status(Status::OpenWire);
    }
    Ok(())
}
fn connect_edges(face: &FaceHandle, wire: &WireHandle, indices: [i32; 3]) -> Result<(), Failure> {
    let mut curves = Vec::with_capacity(3);
    for index in indices {
        curves.push(
            wire.get_edge(index)?
                .upgrade()
                .get_pcurve_for(&face.downgrade(), wire.get_edge_orientation(index)?)?,
        );
    }
    connect::connect(&curves[0], &curves[1], &curves[2])
}
pub(super) fn degenerate_edges(face: &FaceHandle) -> Result<Handle<FaceEdgeSet>, Failure> {
    if face.wires_nb() != 1 {
        return Ok(Handle::default());
    }
    let wire = face.get_wire(0)?;
    if wire.edges_nb() != 2 {
        return Ok(Handle::default());
    }
    degenerate_wire_edges(face, &wire)
}
fn degenerate_wire_edges(
    face: &FaceHandle,
    wire: &WireHandle,
) -> Result<Handle<FaceEdgeSet>, Failure> {
    let edge0 = wire.get_edge(0)?;
    let edge1 = wire.get_edge(1)?;
    let curve0 = edge0
        .upgrade()
        .get_pcurve_for(&face.downgrade(), wire.get_edge_orientation(0)?)?;
    let curve1 = edge1
        .upgrade()
        .get_pcurve_for(&face.downgrade(), wire.get_edge_orientation(1)?)?;
    if !both_two_points(&curve0, &curve1) {
        return Ok(Handle::default());
    }
    Ok(edge_pair_set(edge0, edge1))
}
fn edge_pair_set(
    edge0: crate::imesh_data::EdgePtr,
    edge1: crate::imesh_data::EdgePtr,
) -> Handle<FaceEdgeSet> {
    let mut edges = Map::default();
    edges.add(edge0);
    edges.add(edge1);
    Handle::new(Shared::new(SharedState::new(edges)))
}

fn both_two_points(
    first: &crate::imesh_data::PCurveHandle,
    second: &crate::imesh_data::PCurveHandle,
) -> bool {
    first.parameters_nb() == 2 && second.parameters_nb() == 2
}
