// Rust translation of isForwardChildType, isCoreParityOrientation, enqueueFace, the
// traverse* functions, registerTopology, traverseFlattenedChildren and traverseTopology
// of ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Populate.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The hierarchy traversal of population (phase 1): containers become nodes with
//! references, faces are queued for extraction, free wires, edges and vertices are
//! registered directly.
use super::context::{
    BuildContext, TopologyBuildMode, definition_shape_key, shape_without_own_location,
};
use super::extract::extract_edge_definition;
use super::face::{ExtractedWire, FaceBuildData, order_extracted_wire};
use super::register::{
    append_wire_co_edge, append_wire_def, register_edge, register_or_reuse_vertex,
};
use super::supplement::{attach_non_core_container_child, attach_supplement};
use crate::brep_graph::*;
use crate::brep_graph_inc::ParityOrientation;
use crate::topods::{self, Shape, ShapeIterator};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::toploc::Location;

/// `isForwardChildType`: the child types the core model keeps under each owner type.
pub(super) fn is_forward_child_type(owner: ShapeEnum, child: ShapeEnum) -> bool {
    match owner {
        ShapeEnum::Compound => child != ShapeEnum::Shape,
        ShapeEnum::CompSolid => child == ShapeEnum::Solid,
        ShapeEnum::Solid => child == ShapeEnum::Shell,
        ShapeEnum::Shell => child == ShapeEnum::Face,
        ShapeEnum::Wire => child == ShapeEnum::Edge,
        _ => false,
    }
}

/// `isCoreParityOrientation`: FORWARD or REVERSED.
pub(super) fn is_core_parity_orientation(orientation: Orientation) -> bool {
    orientation == Orientation::Forward || orientation == Orientation::Reversed
}

// The children of a container as stored, with a flag telling whether the core model
// keeps each one (else it goes to the supplement layer).
fn container_children(container: &Shape, owner: ShapeEnum) -> Result<Vec<(Shape, bool)>, Failure> {
    let mut children = Vec::new();
    for child in ShapeIterator::new(container, false, false) {
        let core = is_forward_child_type(owner, child.shape_type()?)
            && is_core_parity_orientation(child.orientation());
        children.push((child, core));
    }
    Ok(children)
}

// The new container node of the start of every `traverse*`: bound with its original.
fn begin_container(
    build: &mut BuildContext<'_>,
    shape: &Shape,
    node: NodeId,
    definition_location: &Location,
) -> Result<(), Failure> {
    build.bind_located_node(shape, node, definition_location, true)?;
    let key = definition_shape_key(shape, definition_location)?;
    build.storage.bind_original(node, &key);
    Ok(())
}

/// `enqueueFace`: the existing face of the TShape at the location, else a new face
/// queued for extraction (bound, with its original and supplement).
fn enqueue_face(
    build: &mut BuildContext<'_>,
    face: &Shape,
    parent_location: &Location,
) -> Result<NodeId, Failure> {
    let definition_location = (parent_location * face.location())?;
    let existing = build.find_existing_node(face, NodeKind::FACE, &definition_location)?;
    if existing.is_valid() {
        return Ok(existing);
    }
    let face_id = build.storage.append_face();
    begin_container(build, face, face_id.into(), &definition_location)?;
    attach_supplement(build.supplement, face_id.into(), face)?;
    build.pending_faces.push(FaceBuildData {
        face: topods::Face::unchecked(face.clone()),
        face_id,
        definition_location,
        ..FaceBuildData::default()
    });
    Ok(face_id.into())
}

// `traverseCompound`: children are traversed at the identity location and referenced
// with their location composed with the compound's.
fn traverse_compound(
    build: &mut BuildContext<'_>,
    compound: &Shape,
    parent_location: &Location,
) -> Result<NodeId, Failure> {
    let definition_location = (parent_location * compound.location())?;
    let existing = build.find_existing_node(compound, NodeKind::COMPOUND, &definition_location)?;
    if existing.is_valid() {
        return Ok(existing);
    }
    let compound_id = build.storage.append_compound();
    begin_container(build, compound, compound_id.into(), &definition_location)?;
    for (child, core) in container_children(compound, ShapeEnum::Compound)? {
        if !core {
            attach_non_core_container_child(
                build.supplement,
                compound_id.into(),
                ShapeEnum::Compound,
                &child,
            );
            continue;
        }
        let child_node = traverse_topology(
            build,
            &shape_without_own_location(&child)?,
            &Location::default(),
        )?;
        if child_node.is_valid() {
            let location = (&definition_location * child.location())?;
            let orientation = ParityOrientation::new(child.orientation())?;
            build
                .storage
                .attach_child_to_compound(compound_id, child_node, &location, orientation);
        }
    }
    Ok(compound_id.into())
}

// The shared body of `traverseCompSolid` and `traverseSolid`: core children of the
// expected kind are traversed at the container location and referenced.
fn traverse_typed_children(
    build: &mut BuildContext<'_>,
    container: &Shape,
    owner: NodeId,
    owner_type: ShapeEnum,
    definition_location: &Location,
) -> Result<Vec<(NodeId, ParityOrientation)>, Failure> {
    let mut kept = Vec::new();
    for (child, core) in container_children(container, owner_type)? {
        if !core {
            attach_non_core_container_child(build.supplement, owner, owner_type, &child);
            continue;
        }
        let child_node = traverse_topology(build, &child, definition_location)?;
        kept.push((child_node, ParityOrientation::new(child.orientation())?));
    }
    Ok(kept)
}

// `traverseCompSolid`.
fn traverse_comp_solid(
    build: &mut BuildContext<'_>,
    comp_solid: &Shape,
    parent_location: &Location,
) -> Result<NodeId, Failure> {
    let definition_location = (parent_location * comp_solid.location())?;
    let existing =
        build.find_existing_node(comp_solid, NodeKind::COMP_SOLID, &definition_location)?;
    if existing.is_valid() {
        return Ok(existing);
    }
    let id = build.storage.append_comp_solid();
    begin_container(build, comp_solid, id.into(), &definition_location)?;
    for (child, orientation) in traverse_typed_children(
        build,
        comp_solid,
        id.into(),
        ShapeEnum::CompSolid,
        &definition_location,
    )? {
        if child.is_valid() && child.kind == NodeKind::SOLID {
            build
                .storage
                .attach_solid_to_comp_solid(id, SolidId::from_node_id(child), orientation);
        }
    }
    Ok(id.into())
}

// `traverseSolid`.
fn traverse_solid(
    build: &mut BuildContext<'_>,
    solid: &Shape,
    parent_location: &Location,
) -> Result<NodeId, Failure> {
    let definition_location = (parent_location * solid.location())?;
    let existing = build.find_existing_node(solid, NodeKind::SOLID, &definition_location)?;
    if existing.is_valid() {
        return Ok(existing);
    }
    let id = build.storage.append_solid();
    begin_container(build, solid, id.into(), &definition_location)?;
    for (child, orientation) in traverse_typed_children(
        build,
        solid,
        id.into(),
        ShapeEnum::Solid,
        &definition_location,
    )? {
        if child.is_valid() && child.kind == NodeKind::SHELL {
            build
                .storage
                .attach_shell_to_solid(id, ShellId::from_node_id(child), orientation);
        }
    }
    Ok(id.into())
}

// `traverseShell`: core faces are queued at the shell location and referenced.
fn traverse_shell(
    build: &mut BuildContext<'_>,
    shell: &Shape,
    parent_location: &Location,
) -> Result<NodeId, Failure> {
    let definition_location = (parent_location * shell.location())?;
    let existing = build.find_existing_node(shell, NodeKind::SHELL, &definition_location)?;
    if existing.is_valid() {
        return Ok(existing);
    }
    let id = build.storage.append_shell();
    begin_container(build, shell, id.into(), &definition_location)?;
    for (child, core) in container_children(shell, ShapeEnum::Shell)? {
        if !core {
            attach_non_core_container_child(build.supplement, id.into(), ShapeEnum::Shell, &child);
            continue;
        }
        let face = enqueue_face(build, &child, &definition_location)?;
        let orientation = ParityOrientation::new(child.orientation())?;
        build
            .storage
            .attach_face_to_shell(id, FaceId::from_node_id(face), orientation);
    }
    Ok(id.into())
}

// `traverseWire`: a free wire with coedges without face, in explorer order.
fn traverse_wire(
    build: &mut BuildContext<'_>,
    wire: &Shape,
    parent_location: &Location,
) -> Result<NodeId, Failure> {
    let definition_location = (parent_location * wire.location())?;
    let existing = build.find_existing_node(wire, NodeKind::WIRE, &definition_location)?;
    if existing.is_valid() {
        return Ok(existing);
    }
    let wire_shape = topods::Wire::unchecked(wire.clone());
    let wire_id = append_wire_def(build, &wire_shape, &definition_location, false)?;
    let mut data = ExtractedWire {
        shape: wire_shape,
        definition_location: definition_location.clone(),
        ..ExtractedWire::default()
    };
    for child in ShapeIterator::new(wire, false, false) {
        if is_forward_child_type(ShapeEnum::Wire, child.shape_type()?) {
            data.edges.push(extract_edge_definition(
                &topods::Edge::unchecked(child),
                &definition_location,
            )?);
        }
    }
    build.has_wire_order_warnings =
        build.has_wire_order_warnings || !order_extracted_wire(&mut data, None)?;
    for edge_data in &data.edges {
        let edge = register_edge(build, edge_data)?;
        if edge.is_valid() {
            append_wire_co_edge(build.storage, wire_id, edge, edge_data.orientation_in_wire)?;
        }
    }
    let status = build
        .storage
        .canonicalize_wire_co_edge_order_status(wire_id);
    build.record_wire_order_status(status);
    Ok(wire_id.into())
}

// `traverseEdge`: a free edge registered at the parent location.
fn traverse_edge(
    build: &mut BuildContext<'_>,
    edge: &Shape,
    parent_location: &Location,
) -> Result<NodeId, Failure> {
    let definition_location = (parent_location * edge.location())?;
    let existing = build.find_existing_node(edge, NodeKind::EDGE, &definition_location)?;
    if existing.is_valid() {
        return Ok(existing);
    }
    let data = extract_edge_definition(&topods::Edge::unchecked(edge.clone()), parent_location)?;
    Ok(register_edge(build, &data)?.into())
}

// `registerTopology`: the dispatch by shape type.
fn register_topology(
    build: &mut BuildContext<'_>,
    shape: &Shape,
    parent_location: &Location,
) -> Result<NodeId, Failure> {
    match shape.shape_type()? {
        ShapeEnum::Compound => traverse_compound(build, shape, parent_location),
        ShapeEnum::CompSolid => traverse_comp_solid(build, shape, parent_location),
        ShapeEnum::Solid => traverse_solid(build, shape, parent_location),
        ShapeEnum::Shell => traverse_shell(build, shape, parent_location),
        ShapeEnum::Face => enqueue_face(build, shape, parent_location),
        ShapeEnum::Wire => traverse_wire(build, shape, parent_location),
        ShapeEnum::Edge => traverse_edge(build, shape, parent_location),
        ShapeEnum::Vertex => {
            let location = (parent_location * shape.location())?;
            let vertex = topods::Vertex::unchecked(shape.clone());
            Ok(register_or_reuse_vertex(build, &vertex, &location)?.into())
        }
        ShapeEnum::Shape => Ok(NodeId::default()),
    }
}

// `traverseFlattenedChildren`: the core children of a flattened container.
fn traverse_flattened_children(
    build: &mut BuildContext<'_>,
    container: &Shape,
    parent_location: &Location,
) -> Result<(), Failure> {
    let definition_location = (parent_location * container.location())?;
    for (child, core) in container_children(container, container.shape_type()?)? {
        if core {
            traverse_topology(build, &child, &definition_location)?;
        }
    }
    Ok(())
}

/// `traverseTopology`: flattened containers pass their children through; any other
/// shape is registered (and becomes a root of a flattened append).
pub(super) fn traverse_topology(
    build: &mut BuildContext<'_>,
    shape: &Shape,
    parent_location: &Location,
) -> Result<NodeId, Failure> {
    if shape.is_null() {
        return Ok(NodeId::default());
    }
    let shape_type = shape.shape_type()?;
    let container = matches!(
        shape_type,
        ShapeEnum::Compound | ShapeEnum::CompSolid | ShapeEnum::Solid | ShapeEnum::Shell
    );
    if build.mode == TopologyBuildMode::Flattened && container {
        traverse_flattened_children(build, shape, parent_location)?;
        return Ok(NodeId::default());
    }
    let node = register_topology(build, shape, parent_location)?;
    if build.mode == TopologyBuildMode::Flattened {
        build.append_build_root(node);
    }
    Ok(node)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn child_types_follow_the_native_table() {
        assert!(is_forward_child_type(
            ShapeEnum::Compound,
            ShapeEnum::Vertex
        ));
        assert!(!is_forward_child_type(
            ShapeEnum::Compound,
            ShapeEnum::Shape
        ));
        assert!(is_forward_child_type(ShapeEnum::Solid, ShapeEnum::Shell));
        assert!(!is_forward_child_type(ShapeEnum::Solid, ShapeEnum::Face));
        assert!(!is_forward_child_type(ShapeEnum::Face, ShapeEnum::Wire));
        assert!(is_core_parity_orientation(Orientation::Reversed));
        assert!(!is_core_parity_orientation(Orientation::Internal));
    }
}
