// Rust translation of the owner registration members of BRepGraph_LayerLock
// (SetOwner, UnsetOwner, OnNodeRemoved, OnNodeReplaced, OnRefRemoved, Clear,
// removeRootEntry) of ModelingData/TKBRep/BRepGraph/BRepGraph_LayerLock.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Registering and removing root owners.
use super::LayerLock;
use crate::brep_graph::child_explorer::ChildExplorer;
use crate::brep_graph::layer::AttachedGraph;
use crate::brep_graph::{BRepGraph, ItemDomain, ItemId, NodeId, RefId};
use rustycad_tkernel::standard::{Failure, Guid};

// What `SetOwner` finds in the subtree of a new node root.
enum DescendantRoots {
    // Every descendant root has the same owner: they are absorbed by the new root.
    Absorbed(Vec<ItemId>),
    // A descendant root has another owner: the new root is rejected.
    Conflict,
}

impl LayerLock {
    /// `SetOwner(theItem, theOwnerId)`: registers a root owner and publishes it;
    /// Standard_ProgramError (`Standard_ASSERT_RAISE`, active in release builds too)
    /// when nothing was stored (invalid item, re-registration, rejected overlap).
    pub fn set_owner(&self, graph: &BRepGraph, item: ItemId, owner: &Guid) -> Result<(), Failure> {
        if self.set_owner_with_revision(graph, item, owner, true)? {
            return Ok(());
        }
        Err(Failure::with_type(
            "Standard_ProgramError",
            &format!(
                "*** ERROR: ASSERT: BRepGraph_LayerLock::SetOwner: rejected or invalid item \
                 {item:?} (expected a valid item not covered by another owner)"
            ),
            "",
        ))
    }

    /// `SetOwner(theNode, theOwnerId)`.
    pub fn set_owner_node(
        &self,
        graph: &BRepGraph,
        node: NodeId,
        owner: &Guid,
    ) -> Result<(), Failure> {
        self.set_owner(graph, node.into(), owner)
    }

    /// `SetOwner(theRef, theOwnerId)`.
    pub fn set_owner_ref(
        &self,
        graph: &BRepGraph,
        reference: RefId,
        owner: &Guid,
    ) -> Result<(), Failure> {
        self.set_owner(graph, reference.into(), owner)
    }

    /// `SetOwner(theItem, theOwnerId, theToUpdateRevision)`: true when a new root entry
    /// was stored. A root of the same owner (direct or ancestor) only re-sets the item's
    /// bit; another owner's root rejects the item, as does a descendant root of another
    /// owner; same-owner descendant roots are absorbed. A node root propagates its bit to
    /// its descendants. The revision update can be postponed by bulk callers.
    pub fn set_owner_with_revision(
        &self,
        graph: &BRepGraph,
        item: ItemId,
        owner: &Guid,
        update_revision: bool,
    ) -> Result<bool, Failure> {
        if !item.is_valid() {
            return Ok(false);
        }
        if let Some(covering) = self.covering_owner(graph, item)? {
            self.reaffirm_if_same(graph, item, covering == *owner, update_revision);
            return Ok(false);
        }
        let DescendantRoots::Absorbed(absorbed) = self.descendant_roots(graph, item, owner)? else {
            return Ok(false);
        };
        self.store_root(graph, item, owner, &absorbed)?;
        if update_revision {
            self.base.touch();
        }
        Ok(true)
    }

    // The owner of the item's own root entry, else of an ancestor root covering it.
    fn covering_owner(&self, graph: &BRepGraph, item: ItemId) -> Result<Option<Guid>, Failure> {
        let existing = self.owners.read().map_of(item).seek(&item).copied();
        if existing.is_some() {
            return Ok(existing);
        }
        self.is_covered_by_ancestor(graph, item)
    }

    // A new root: the absorbed roots unbound, the root bound, its bit and (for a node)
    // its subtree's bits set.
    fn store_root(
        &self,
        graph: &BRepGraph,
        item: ItemId,
        owner: &Guid,
        absorbed: &[ItemId],
    ) -> Result<(), Failure> {
        self.bind_root(item, owner, absorbed);
        self.set_item_owned(Some(graph), item, true);
        if item.item_domain() == ItemDomain::Node {
            self.expand_ownership(Some(graph), item.node_id(), true)?;
        }
        Ok(())
    }

    // An idempotent re-registration by the same owner re-sets the bit.
    fn reaffirm_if_same(&self, graph: &BRepGraph, item: ItemId, same: bool, update_revision: bool) {
        if !same {
            return;
        }
        self.set_item_owned(Some(graph), item, true);
        if update_revision {
            self.base.touch();
        }
    }

    // The roots already stored below a new node root (none for a reference or a
    // detached layer), in child-explorer order.
    fn descendant_roots(
        &self,
        graph: &BRepGraph,
        item: ItemId,
        owner: &Guid,
    ) -> Result<DescendantRoots, Failure> {
        let mut absorbed = Vec::new();
        let attached = self.base.attached_graph(graph);
        let (Some(graph), ItemDomain::Node) = (attached, item.item_domain()) else {
            return Ok(DescendantRoots::Absorbed(absorbed));
        };
        let mut explorer = ChildExplorer::new(graph, item.node_id())?;
        while explorer.more() {
            let reference = explorer.current_ref();
            let node = Some(ItemId::from(explorer.current().def_id));
            let link = reference.is_valid().then(|| ItemId::from(reference));
            for candidate in [node, link].into_iter().flatten() {
                if !self.absorb_if_same(candidate, owner, &mut absorbed) {
                    return Ok(DescendantRoots::Conflict);
                }
            }
            explorer.next()?;
        }
        Ok(DescendantRoots::Absorbed(absorbed))
    }

    // A descendant item's root entry: none, the same owner (kept for absorption), or
    // another owner (false: the new root is rejected).
    fn absorb_if_same(&self, candidate: ItemId, owner: &Guid, absorbed: &mut Vec<ItemId>) -> bool {
        match self.owners.read().map_of(candidate).seek(&candidate) {
            Some(found) if found != owner => false,
            Some(_) => {
                absorbed.push(candidate);
                true
            }
            None => true,
        }
    }

    // Unbinds the absorbed descendant roots, then binds the new root.
    fn bind_root(&self, item: ItemId, owner: &Guid, absorbed: &[ItemId]) {
        let mut owners = self.owners.write();
        for root in absorbed {
            owners.map_of_mut(*root).unbind(root);
        }
        owners.map_of_mut(item).bind(item, *owner);
    }

    /// `UnsetOwner(theItem)`: removes a root entry, clears its bits (a node root clears
    /// its descendants) and sets the bits of the remaining roots again.
    pub fn unset_owner(&self, graph: &BRepGraph, item: ItemId) -> Result<(), Failure> {
        if !item.is_valid() || item.item_domain() == ItemDomain::None {
            return Ok(());
        }
        if !self.owners.write().map_of_mut(item).unbind(&item) {
            return Ok(());
        }
        self.set_item_owned(Some(graph), item, false);
        if item.item_domain() == ItemDomain::Node {
            self.expand_ownership(Some(graph), item.node_id(), false)?;
        }
        self.rebuild_owned_flags_from_roots(Some(graph))?;
        self.base.touch();
        Ok(())
    }

    /// `UnsetOwner(theNode)`.
    pub fn unset_owner_node(&self, graph: &BRepGraph, node: NodeId) -> Result<(), Failure> {
        self.unset_owner(graph, node.into())
    }

    /// `UnsetOwner(theRef)`.
    pub fn unset_owner_ref(&self, graph: &BRepGraph, reference: RefId) -> Result<(), Failure> {
        self.unset_owner(graph, reference.into())
    }

    /// `UnsetOwner(theItem, theOwnerId)`: `UnsetOwner` when the item's own root entry
    /// has this owner.
    pub fn unset_owner_matching(
        &self,
        graph: &BRepGraph,
        item: ItemId,
        owner: &Guid,
    ) -> Result<(), Failure> {
        if !item.is_valid() || item.item_domain() == ItemDomain::None {
            return Ok(());
        }
        let stored = self.owners.read().map_of(item).seek(&item).copied();
        if stored != Some(*owner) {
            return Ok(());
        }
        self.unset_owner(graph, item)
    }

    /// `OnNodeRemoved`: drops a node root and its bits. The walks cannot report an
    /// error from the noexcept callback (natively the process terminates); the port
    /// stops walking.
    pub(super) fn drop_node_root(&self, attached: AttachedGraph<'_>, node: NodeId) {
        let item = ItemId::from(node);
        if !self.owners.write().nodes.unbind(&item) {
            return;
        }
        self.set_item_owned(attached, item, false);
        let _walk = self.expand_ownership(attached, node, false);
        let _rebuild = self.rebuild_owned_flags_from_roots(attached);
        self.base.touch();
    }

    /// `OnNodeReplaced`: moves a node root to the new node (only the two nodes' own
    /// bits change).
    pub(super) fn move_node_root(&self, attached: AttachedGraph<'_>, old: NodeId, new: NodeId) {
        let (old_item, new_item) = (ItemId::from(old), ItemId::from(new));
        {
            let mut owners = self.owners.write();
            let Some(owner) = owners.nodes.seek(&old_item).copied() else {
                return;
            };
            owners.nodes.unbind(&old_item);
            owners.nodes.bind(new_item, owner);
        }
        self.set_item_owned(attached, old_item, false);
        self.set_item_owned(attached, new_item, true);
        self.base.touch();
    }

    /// `OnRefRemoved`: drops a reference root and its bit.
    pub(super) fn drop_ref_root(&self, attached: AttachedGraph<'_>, reference: RefId) {
        let item = ItemId::from(reference);
        if !self.owners.write().refs.unbind(&item) {
            return;
        }
        self.set_item_owned(attached, item, false);
        self.base.touch();
    }

    /// `Clear()`: clears the bits of every node root (and descendants) and reference
    /// root in map order, then drops the entries and their buckets.
    pub(super) fn clear_roots(&self, attached: AttachedGraph<'_>) {
        let (nodes, refs) = self.root_entries();
        for (item, _) in nodes {
            self.set_item_owned(attached, item, false);
            let _walk = self.expand_ownership(attached, item.node_id(), false);
        }
        self.owners.write().nodes.clear(true);
        for (item, _) in refs {
            self.set_item_owned(attached, item, false);
        }
        self.owners.write().refs.clear(true);
        self.base.touch();
    }

    /// `removeRootEntry(theItem)` (private): drops one root entry and its own bit.
    /// Declared and defined natively but called nowhere; ported for completeness.
    #[allow(dead_code)]
    pub(super) fn remove_root_entry(&self, attached: AttachedGraph<'_>, item: ItemId) {
        if item.item_domain() == ItemDomain::None {
            return;
        }
        if self.owners.write().map_of_mut(item).unbind(&item) {
            self.set_item_owned(attached, item, false);
            self.base.touch();
        }
    }
}
