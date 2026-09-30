// Rust translation of snapshotCountForKind, detectTopologyRoot, populateUIDsIncremental,
// appendImpl, bindSourceShapeAlias, bindSourceShapeAliasesRecursive and
// bindSourceShapeAliases of ModelingData/TKBRep/BRepGraph/BRepGraph_ShapesView.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The population step of `Add`: the storage append, the UIDs of the new nodes and
//! references, the root detection and the source-shape aliases.
use super::route::shape_without_own_location;
use super::{AddStatus, Options};
use crate::brep_graph::{BRepGraph, NodeId, NodeKind, RefId, RefKind};
use crate::brep_graph_inc::populate::{self, BuildStatus};
use crate::topods::{Shape, ShapeIterator};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::toploc::Location;

// The node kind of a TopoDS shape type, `None` for SHAPE.
fn node_kind_of(shape_type: ShapeEnum) -> Option<NodeKind> {
    match shape_type {
        ShapeEnum::Compound => Some(NodeKind::COMPOUND),
        ShapeEnum::CompSolid => Some(NodeKind::COMP_SOLID),
        ShapeEnum::Solid => Some(NodeKind::SOLID),
        ShapeEnum::Shell => Some(NodeKind::SHELL),
        ShapeEnum::Face => Some(NodeKind::FACE),
        ShapeEnum::Wire => Some(NodeKind::WIRE),
        ShapeEnum::Edge => Some(NodeKind::EDGE),
        ShapeEnum::Vertex => Some(NodeKind::VERTEX),
        ShapeEnum::Shape => None,
    }
}

/// `snapshotCountForKind(theGraph, theShapeType)`: the definition count of the shape
/// type's kind, 0 for SHAPE.
pub(super) fn snapshot_count_for_kind(graph: &BRepGraph, shape_type: ShapeEnum) -> u32 {
    node_kind_of(shape_type).map_or(0, |kind| graph.topo().generic().nb(kind))
}

/// `detectTopologyRoot(theGraph, theShapeType, theOldCountOfShapeKind)`: population
/// appends in declaration order, so the first new node of the kind is the root.
pub(super) fn detect_topology_root(
    graph: &BRepGraph,
    shape_type: ShapeEnum,
    old_count: u32,
) -> NodeId {
    if snapshot_count_for_kind(graph, shape_type) <= old_count {
        return NodeId::default();
    }
    node_kind_of(shape_type).map_or_else(NodeId::default, |kind| NodeId::new(kind, old_count))
}

// The node kinds and reference kinds whose new entries get UIDs, in the native order.
const UID_NODE_KINDS: [NodeKind; 11] = [
    NodeKind::VERTEX,
    NodeKind::EDGE,
    NodeKind::CO_EDGE,
    NodeKind::WIRE,
    NodeKind::FACE,
    NodeKind::SHELL,
    NodeKind::SOLID,
    NodeKind::COMPOUND,
    NodeKind::COMP_SOLID,
    NodeKind::PRODUCT,
    NodeKind::OCCURRENCE,
];
const UID_REF_KINDS: [RefKind; 6] = [
    RefKind::SHELL,
    RefKind::FACE,
    RefKind::WIRE,
    RefKind::VERTEX,
    RefKind::SOLID,
    RefKind::CHILD,
];

// The counts before an append, in the order of the kind lists.
struct CountSnapshot {
    nodes: [u32; UID_NODE_KINDS.len()],
    refs: [u32; UID_REF_KINDS.len()],
}

impl CountSnapshot {
    fn take(graph: &BRepGraph) -> Self {
        Self {
            nodes: UID_NODE_KINDS.map(|kind| graph.topo().generic().nb(kind)),
            refs: UID_REF_KINDS.map(|kind| graph.refs().generic().nb(kind)),
        }
    }
}

/// `populateUIDsIncremental`: a UID for every node and reference appended since the
/// snapshot (removed ones included, as the full-traverse iterators visit them).
fn populate_uids_incremental(graph: &mut BRepGraph, old: &CountSnapshot) {
    for (kind, start) in UID_NODE_KINDS.into_iter().zip(old.nodes) {
        for index in start..graph.topo().generic().nb(kind) {
            graph.allocate_uid(NodeId::new(kind, index));
        }
    }
    for (kind, start) in UID_REF_KINDS.into_iter().zip(old.refs) {
        for index in start..graph.refs().generic().nb(kind) {
            graph.allocate_ref_uid(RefId::new(kind, index));
        }
    }
}

/// `appendImpl(theGraph, theShape, theOptions, theOutFlatRoots)`: the storage append
/// (flattened or full), the reconstructed-shape cache cleared, UIDs for the new entries.
/// The native post-append `ValidateMutationBoundary` is a debug-only assertion.
pub(super) fn append_impl(
    graph: &mut BRepGraph,
    shape: &Shape,
    options: &Options,
    flat_roots: &mut Vec<NodeId>,
) -> Result<AddStatus, Failure> {
    let old = CountSnapshot::take(graph);
    let status = if options.flatten {
        populate::append_flattened(
            graph,
            shape,
            options.parallel,
            flat_roots,
            &options.populate,
        )?
    } else {
        populate::append(graph, shape, options.parallel, &options.populate)?
    };
    graph.inc_storage().clear_current_shapes();
    populate_uids_incremental(graph, &old);
    Ok(match status {
        BuildStatus::Failed => AddStatus::Failed,
        BuildStatus::SuccessWithWarnings => AddStatus::SuccessWithWarnings,
        BuildStatus::Success => AddStatus::Success,
    })
}

// `bindSourceShapeAlias`: binds the source shape (and the source shape at the
// accumulated source location, when different) to the node of the populated shape at
// its definition location.
fn bind_source_shape_alias(
    graph: &BRepGraph,
    source: &Shape,
    source_location: &Location,
    populated: &Shape,
    definition_location: &Location,
) -> Result<(), Failure> {
    if source.is_null() || populated.is_null() {
        return Ok(());
    }
    let mut lookup = populated.clone();
    lookup.set_location(definition_location, false)?;
    let node = graph.shapes().find_node(&lookup);
    if !node.is_valid() {
        return Ok(());
    }
    let storage = graph.inc_storage();
    storage.set_definition_shape_binding(source, node);
    if !source.location().is_equal(source_location) {
        let mut alias = source.clone();
        alias.set_location(source_location, false)?;
        storage.set_definition_shape_binding(&alias, node);
    }
    Ok(())
}

// `bindSourceShapeAliasesRecursive`: the alias of the pair, then the children pairwise
// (compound children are definitions at the identity location).
fn bind_aliases_recursive(
    graph: &BRepGraph,
    source: &Shape,
    populated: &Shape,
    source_location: &Location,
    definition_location: &Location,
) -> Result<(), Failure> {
    bind_source_shape_alias(
        graph,
        source,
        source_location,
        populated,
        definition_location,
    )?;
    let compound_parent = populated.shape_type()? == ShapeEnum::Compound;
    let sources = ShapeIterator::new(source, false, false);
    let populated_children = ShapeIterator::new(populated, false, false);
    for (source_child, populated_child) in sources.zip(populated_children) {
        if source_child.is_null() || populated_child.is_null() {
            continue;
        }
        let child_source_location = (source_location * source_child.location())?;
        if compound_parent {
            let definition = shape_without_own_location(&populated_child)?;
            bind_aliases_recursive(
                graph,
                &source_child,
                &definition,
                &child_source_location,
                &Location::default(),
            )?;
        } else {
            let child_location = (definition_location * populated_child.location())?;
            bind_aliases_recursive(
                graph,
                &source_child,
                &populated_child,
                &child_source_location,
                &child_location,
            )?;
        }
    }
    Ok(())
}

/// `bindSourceShapeAliases(theGraph, theSourceShape, thePopulatedShape)`: the source
/// shape and its sub-shapes (at their source locations) resolve to the nodes built for
/// the populated shape.
pub(super) fn bind_source_shape_aliases(
    graph: &BRepGraph,
    source: &Shape,
    populated: &Shape,
) -> Result<(), Failure> {
    bind_aliases_recursive(
        graph,
        source,
        populated,
        source.location(),
        populated.location(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_detection_uses_the_first_new_node() {
        let mut graph = BRepGraph::new();
        assert_eq!(snapshot_count_for_kind(&graph, ShapeEnum::Shape), 0);
        graph.inc_storage_mut().append_face();
        assert_eq!(
            detect_topology_root(&graph, ShapeEnum::Face, 0),
            NodeId::new(NodeKind::FACE, 0)
        );
        assert!(!detect_topology_root(&graph, ShapeEnum::Face, 1).is_valid());
        assert!(!detect_topology_root(&graph, ShapeEnum::Shape, 0).is_valid());
    }
}
