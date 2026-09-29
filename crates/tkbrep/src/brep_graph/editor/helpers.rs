// Copyright (c) 2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of the anonymous-namespace helpers and the private isOwned,
// requireUnlocked and requireNoActiveGuard members of
// ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx (and their copies in
// BRepGraph_EditorView_Mut.cxx and BRepGraph_EditorView_Setters.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The checks and relation queries that the editor operations share.
use crate::brep_graph::parent_explorer::{ParentExplorer, TraversalMode};
use crate::brep_graph::*;
use crate::brep_graph_inc::Storage;
use rustycad_tkernel::standard::Failure;

/// `isOwned(theItem)` (private natively): the item is in range and owned (locked).
pub(crate) fn is_owned(graph: &BRepGraph, item: ItemId) -> bool {
    match item.item_domain() {
        ItemDomain::Node => item.node_id().is_owned(graph),
        ItemDomain::Reference => item.ref_id().is_owned(graph),
        ItemDomain::None => false,
    }
}

/// `requireUnlocked(theItem, theOperation)` (private natively): an owned item raises
/// Standard_ProgramError with the operation text.
pub(crate) fn require_unlocked(
    graph: &BRepGraph,
    item: ItemId,
    operation: &str,
) -> Result<(), Failure> {
    if is_owned(graph, item) {
        return Err(program_error(operation, item));
    }
    Ok(())
}

/// `requireNoActiveGuard(theItem, theOperation)` (private natively): a guarded item
/// raises Standard_ProgramError with the operation text.
pub(crate) fn require_no_active_guard(
    graph: &BRepGraph,
    item: ItemId,
    operation: &str,
) -> Result<(), Failure> {
    if graph.inc_storage().is_item_guarded(item) {
        return Err(program_error(operation, item));
    }
    Ok(())
}

/// Both checks of an editor member on one item, in native order, with the native
/// `locked` and `guard active` texts.
pub(crate) fn require_mutable(
    graph: &BRepGraph,
    item: ItemId,
    locked: &str,
    guarded: &str,
) -> Result<(), Failure> {
    require_unlocked(graph, item, locked)?;
    require_no_active_guard(graph, item, guarded)
}

/// `requireUnlocked(theItem, "BRepGraph::EditorView: locked item")`: the check every
/// immediate setter runs.
pub(crate) fn require_item_unlocked(graph: &BRepGraph, item: ItemId) -> Result<(), Failure> {
    require_unlocked(graph, item, "BRepGraph::EditorView: locked item")
}

/// The checks of an immediate setter on a caller-given id: `requireUnlocked` (first,
/// as natively), then the range of the record the setter writes. The native setters
/// write `ChangeVertex(theVertex)`... without a range check (undefined behaviour past
/// the store); the port returns Standard_OutOfRange instead of panicking.
pub(crate) fn require_settable<T>(graph: &BRepGraph, id: TypedId<T>) -> Result<(), Failure>
where
    T: StoredTag,
    ItemId: From<<T::Family as IdFamily>::Untyped>,
{
    let item = ItemId::from(<T::Family as IdFamily>::untyped(T::RAW_KIND, id.index));
    require_item_unlocked(graph, item)?;
    graph.inc_storage().check_id(id).map(|_| ())
}

/// `BRepGraph_ItemId(theNode)` of a typed or untyped node id.
pub(crate) fn node_item(node: impl Into<NodeId>) -> ItemId {
    ItemId::from(node.into())
}

/// `BRepGraph_ItemId(theRef)` of a typed or untyped reference id.
pub(crate) fn ref_item(reference: impl Into<RefId>) -> ItemId {
    ItemId::from(reference.into())
}

/// A Standard_ProgramError whose message is the native operation text followed by
/// the offending item.
pub(crate) fn program_error(operation: &str, item: ItemId) -> Failure {
    Failure::with_type(
        "Standard_ProgramError",
        &format!(
            "{operation} (item domain {:?}, kind {}, index {})",
            item.item_domain(),
            item.raw_kind(),
            item.index()
        ),
        "",
    )
}

/// `isActiveNode` (private natively): in range and not removed.
pub(crate) fn is_active_node(graph: &BRepGraph, node: impl Into<NodeId>) -> bool {
    graph.topo().generic().is_active(node.into())
}

/// `isNodeIndexInRange` (private natively).
pub(crate) fn is_node_in_range(graph: &BRepGraph, node: impl Into<NodeId>) -> bool {
    graph.topo().generic().is_valid(node.into())
}

/// `isActiveTopologyNode` (private natively): an active node of a topology kind.
pub(crate) fn is_active_topology_node(graph: &BRepGraph, node: NodeId) -> bool {
    node.is_valid() && NodeKind::is_topology_kind(node.kind) && is_active_node(graph, node)
}

/// `isRefInRange` (private natively).
pub(crate) fn is_ref_in_range(graph: &BRepGraph, reference: RefId) -> bool {
    graph.refs().generic().is_valid(reference)
}

/// `isValidOccurrenceChildKind`: a product or a topology kind.
pub(crate) fn is_valid_occurrence_child_kind(kind: NodeKind) -> bool {
    kind == NodeKind::PRODUCT || NodeKind::is_topology_kind(kind)
}

/// `hasOtherActiveParent(theGraph, theChild, theExcludedParent)`: a direct parent
/// other than the excluded one (the parent explorer skips removed parents).
pub(crate) fn has_other_active_parent(
    graph: &BRepGraph,
    child: NodeId,
    excluded: NodeId,
) -> Result<bool, Failure> {
    let mut explorer = ParentExplorer::with_mode(graph, child, TraversalMode::DirectParents)?;
    while explorer.more() {
        if explorer.current().def_id != excluded {
            return Ok(true);
        }
        explorer.next()?;
    }
    Ok(false)
}

/// `hasAnyActiveUsage(theGraph, theChild)`: the child has a direct active parent
/// (through any reference kind or the structural edge and occurrence links).
pub(crate) fn has_any_active_usage(graph: &BRepGraph, child: NodeId) -> Result<bool, Failure> {
    if !child.is_valid() {
        return Ok(false);
    }
    let explorer = ParentExplorer::with_mode(graph, child, TraversalMode::DirectParents)?;
    Ok(explorer.more())
}

/// `isExpectedParentKindForRef(theParent, theRefKind)`.
pub(crate) fn is_expected_parent_kind_for_ref(parent: NodeId, kind: RefKind) -> bool {
    let expected = match kind {
        RefKind::VERTEX => NodeKind::EDGE,
        RefKind::WIRE => NodeKind::FACE,
        RefKind::FACE => NodeKind::SHELL,
        RefKind::SHELL => NodeKind::SOLID,
        RefKind::SOLID => NodeKind::COMP_SOLID,
        RefKind::CHILD => NodeKind::COMPOUND,
        RefKind::OCCURRENCE => NodeKind::PRODUCT,
        _ => return false,
    };
    parent.kind == expected
}

/// `productHasOccurrenceDef(theStorage, theProduct, theOccurrence)`: an active
/// reference of the active product places the occurrence.
pub(crate) fn product_has_occurrence_def(
    storage: &Storage,
    product: ProductId,
    occurrence: OccurrenceId,
) -> bool {
    if !product.is_valid_for(storage.nb_products())
        || !occurrence.is_valid_for(storage.nb_occurrences())
        || storage.is_removed(product)
    {
        return false;
    }
    let references = &storage.product_relations(product).occurrence_ref_ids;
    references.iter().any(|reference| {
        reference.is_valid_for(storage.nb_occurrence_refs())
            && !storage.is_removed(*reference)
            && storage.occurrence_ref(*reference).child_occurrence_id == occurrence
    })
}

/// `kindName(theKind)`: the plural kind name of the boundary issues.
pub(crate) fn kind_name(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::VERTEX => "Vertices",
        NodeKind::EDGE => "Edges",
        NodeKind::CO_EDGE => "CoEdges",
        NodeKind::WIRE => "Wires",
        NodeKind::FACE => "Faces",
        NodeKind::SHELL => "Shells",
        NodeKind::SOLID => "Solids",
        NodeKind::COMPOUND => "Compounds",
        NodeKind::COMP_SOLID => "CompSolids",
        NodeKind::PRODUCT => "Products",
        NodeKind::OCCURRENCE => "Occurrences",
        _ => "Unknown",
    }
}

/// `clearCoEdgeFaceScopedRepresentations(theStorage, theCoEdge)`: removes the
/// p-curve, 2D polygon and polygon-on-triangulation records in range and resets the
/// three ids.
pub(crate) fn clear_co_edge_face_scoped_representations(storage: &mut Storage, co_edge: CoEdgeId) {
    let def = storage.co_edge(co_edge).clone();
    if def
        .curve2d_rep_id
        .is_valid_for(storage.nb_co_edge_curves2d())
    {
        storage.mark_rep_removed(def.curve2d_rep_id.into());
    }
    if def
        .polygon2d_rep_id
        .is_valid_for(storage.nb_co_edge_polygons2d())
    {
        storage.mark_rep_removed(def.polygon2d_rep_id.into());
    }
    if def
        .polygon_on_tri_rep_id
        .is_valid_for(storage.nb_co_edge_polygons_on_tri())
    {
        storage.mark_rep_removed(def.polygon_on_tri_rep_id.into());
    }
    let target = storage.change_co_edge(co_edge);
    target.curve2d_rep_id = CoEdgeCurve2DRepId::default();
    target.polygon2d_rep_id = CoEdgePolygon2DRepId::default();
    target.polygon_on_tri_rep_id = CoEdgePolygonOnTriRepId::default();
}

/// Marks the three face-scoped representation records of a coedge removed (the ids
/// stay), as `RemoveNode(CoEdge)`, `RemoveWire` and `Split` do.
pub(crate) fn mark_co_edge_representations_removed(storage: &mut Storage, co_edge: CoEdgeId) {
    let def = storage.co_edge(co_edge);
    let reps: [RepId; 3] = [
        def.curve2d_rep_id.into(),
        def.polygon2d_rep_id.into(),
        def.polygon_on_tri_rep_id.into(),
    ];
    for rep in reps {
        storage.mark_rep_removed(rep);
    }
}

/// `removeFromRootProducts(theRoots, theProduct)`: drops every occurrence of the
/// product, keeping the order of the others.
pub(crate) fn remove_from_root_products(graph: &mut BRepGraph, product: ProductId) {
    graph
        .inc_storage_mut()
        .change_root_product_ids()
        .retain(|root| *root != product);
}
