// Rust translation of the private graph walks of BRepGraph_LayerLock (setItemOwned,
// expandOwnership, rebuildOwnedFlagsFromRoots, findRootNodeId, findRootRefId,
// parentNodeId, isCoveredByAncestor) of
// ModelingData/TKBRep/BRepGraph/BRepGraph_LayerLock.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The ownership bits and the upward searches for the root that covers an item.
use super::LayerLock;
use crate::brep_graph::child_explorer::ChildExplorer;
use crate::brep_graph::parent_explorer::{ParentExplorer, TraversalMode};
use crate::brep_graph::{
    BRepGraph, ChildRefId, FaceRefId, ItemDomain, ItemId, NodeId, NodeTag, NodeVisitor,
    OccurrenceRefId, RefId, RefKind, RefTag, RefVisitor, ShellRefId, SolidRefId, TypedId,
    VertexRefId, WireRefId,
};
use crate::brep_graph_inc::Storage;
use rustycad_tkernel::standard::{Failure, Guid};

impl LayerLock {
    /// `setItemOwned(theItem, theIsOwned)` (private): writes the item's owned bit in the
    /// attached graph; a detached layer writes nothing.
    pub(super) fn set_item_owned(&self, graph: Option<&BRepGraph>, item: ItemId, owned: bool) {
        let Some(graph) = graph.and_then(|graph| self.base.attached_graph(graph)) else {
            return;
        };
        match item.item_domain() {
            ItemDomain::Node => item.node_id().visit(OwnedWriter { graph, owned }),
            ItemDomain::Reference => item.ref_id().visit(OwnedWriter { graph, owned }),
            ItemDomain::None => {}
        }
    }

    /// `expandOwnership(theRoot, theIsOwned)` (private): the bit of every descendant of
    /// the root and of the reference used to reach it.
    pub(super) fn expand_ownership(
        &self,
        graph: Option<&BRepGraph>,
        root: NodeId,
        owned: bool,
    ) -> Result<(), Failure> {
        let Some(graph) = graph.and_then(|graph| self.base.attached_graph(graph)) else {
            return Ok(());
        };
        let mut explorer = ChildExplorer::new(graph, root)?;
        while explorer.more() {
            self.set_item_owned(Some(graph), explorer.current().def_id.into(), owned);
            let reference = explorer.current_ref();
            if reference.is_valid() {
                self.set_item_owned(Some(graph), reference.into(), owned);
            }
            explorer.next()?;
        }
        Ok(())
    }

    /// `rebuildOwnedFlagsFromRoots()` (private): sets the bits of every remaining root
    /// (and the descendants of the node roots) again, in map order.
    pub(super) fn rebuild_owned_flags_from_roots(
        &self,
        graph: Option<&BRepGraph>,
    ) -> Result<(), Failure> {
        let (nodes, refs) = self.root_entries();
        for (item, _) in nodes {
            self.set_item_owned(graph, item, true);
            self.expand_ownership(graph, item.node_id(), true)?;
        }
        for (item, _) in refs {
            self.set_item_owned(graph, item, true);
        }
        Ok(())
    }

    /// `findRootNodeId(theNode, theRootItem)` (private): an owned node whose direct
    /// parents are all unowned is its own root; otherwise the first ancestor, walking
    /// up, whose direct parents are all unowned (none once an unowned ancestor is met).
    pub(super) fn find_root_node_id(
        &self,
        graph: &BRepGraph,
        node: NodeId,
    ) -> Result<Option<ItemId>, Failure> {
        let Some(graph) = self.base.attached_graph(graph) else {
            return Ok(None);
        };
        if !node.is_owned(graph) {
            return Ok(None);
        }
        if !has_owned_direct_parent(graph, node)? {
            return Ok(Some(node.into()));
        }
        for ancestor in ParentExplorer::new(graph, node)? {
            let ancestor = ancestor?.def_id;
            if !ancestor.is_owned(graph) {
                break;
            }
            if !has_owned_direct_parent(graph, ancestor)? {
                return Ok(Some(ancestor.into()));
            }
        }
        Ok(None)
    }

    /// `findRootRefId(theRef, theRootItem)` (private): the reference's own entry, else
    /// the root that covers its parent node.
    pub(super) fn find_root_ref_id(
        &self,
        graph: &BRepGraph,
        reference: RefId,
    ) -> Result<Option<ItemId>, Failure> {
        let item = ItemId::from(reference);
        if self.owners.read().refs.is_bound(&item) {
            return Ok(Some(item));
        }
        let parent = self.parent_node_id(graph, reference);
        if !parent.is_valid() {
            return Ok(None);
        }
        self.find_root_node_id(graph, parent)
    }

    /// `parentNodeId(theRef)` (private): the parent node recorded in a reference in
    /// range (unchecked against the parent kind's range), invalid otherwise or when
    /// detached.
    pub(super) fn parent_node_id(&self, graph: &BRepGraph, reference: RefId) -> NodeId {
        let Some(graph) = self.base.attached_graph(graph) else {
            return NodeId::default();
        };
        let storage = graph.inc_storage();
        match reference.kind {
            RefKind::SHELL => parent_of(storage, ShellRefId::from_ref_id(reference), |e| {
                e.parent_solid_id.into()
            }),
            RefKind::FACE => parent_of(storage, FaceRefId::from_ref_id(reference), |e| {
                e.parent_shell_id.into()
            }),
            RefKind::WIRE => parent_of(storage, WireRefId::from_ref_id(reference), |e| {
                e.parent_face_id.into()
            }),
            RefKind::SOLID => parent_of(storage, SolidRefId::from_ref_id(reference), |e| {
                e.parent_comp_solid_id.into()
            }),
            RefKind::CHILD => parent_of(storage, ChildRefId::from_ref_id(reference), |e| {
                e.parent_compound_id.into()
            }),
            RefKind::OCCURRENCE => {
                parent_of(storage, OccurrenceRefId::from_ref_id(reference), |e| {
                    e.parent_product_id.into()
                })
            }
            RefKind::VERTEX => parent_of(storage, VertexRefId::from_ref_id(reference), |e| {
                e.parent_edge_id.into()
            }),
            _ => NodeId::default(),
        }
    }

    /// `isCoveredByAncestor(theItem, theRootOwnerId)` (private): the owner of the first
    /// ancestor root of a node (in parent-explorer order), or, for a reference, of its
    /// parent node or that node's ancestors.
    pub(super) fn is_covered_by_ancestor(
        &self,
        graph: &BRepGraph,
        item: ItemId,
    ) -> Result<Option<Guid>, Failure> {
        match item.item_domain() {
            ItemDomain::Node => self.ancestor_root_owner(graph, item.node_id()),
            ItemDomain::Reference => {
                let parent = self.parent_node_id(graph, item.ref_id());
                if !parent.is_valid() {
                    return Ok(None);
                }
                let direct = self.owners.read().nodes.seek(&parent.into()).copied();
                match direct {
                    Some(owner) => Ok(Some(owner)),
                    None => self.ancestor_root_owner(graph, parent),
                }
            }
            ItemDomain::None => Ok(None),
        }
    }

    // The node case of `isCoveredByAncestor`.
    fn ancestor_root_owner(
        &self,
        graph: &BRepGraph,
        node: NodeId,
    ) -> Result<Option<Guid>, Failure> {
        let Some(graph) = self.base.attached_graph(graph) else {
            return Ok(None);
        };
        for ancestor in ParentExplorer::new(graph, node)? {
            let ancestor = ItemId::from(ancestor?.def_id);
            if let Some(owner) = self.owners.read().nodes.seek(&ancestor).copied() {
                return Ok(Some(owner));
            }
        }
        Ok(None)
    }
}

// Whether any direct parent of the node carries the owned bit.
fn has_owned_direct_parent(graph: &BRepGraph, node: NodeId) -> Result<bool, Failure> {
    for parent in ParentExplorer::with_mode(graph, node, TraversalMode::DirectParents)? {
        if parent?.def_id.is_owned(graph) {
            return Ok(true);
        }
    }
    Ok(false)
}

// One typed branch of `parentNodeId`: the recorded parent of a reference in range.
fn parent_of<T: RefTag>(
    storage: &Storage,
    reference: TypedId<T>,
    parent: impl Fn(&T::Entry) -> NodeId,
) -> NodeId {
    if !reference.is_valid_for(storage.nb::<T>()) {
        return NodeId::default();
    }
    parent(storage.entry(reference))
}

// `aStorage.SetOwned(theTypedId, theIsOwned)` of `setItemOwned`.
struct OwnedWriter<'g> {
    graph: &'g BRepGraph,
    owned: bool,
}

impl NodeVisitor for OwnedWriter<'_> {
    type Output = ();
    fn visit<T: NodeTag>(self, id: TypedId<T>) {
        self.graph.inc_storage().set_owned(id, self.owned);
    }
}

impl RefVisitor for OwnedWriter<'_> {
    type Output = ();
    fn visit<T: RefTag>(self, id: TypedId<T>) {
        self.graph.inc_storage().set_owned(id, self.owned);
    }
}
