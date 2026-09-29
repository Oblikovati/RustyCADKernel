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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_LayerLock.hxx and
// BRepGraph_LayerLock.cxx (the layer, its queries and lifecycle) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_LayerLock`: the owner metadata of owned graph items.
//!
//! Ownership is root based: only the highest owned item of a group is stored, and every
//! descendant of a node root receives the fast owned bit of the storage. The bits are
//! written through the shared graph the layer is attached to
//! (docs/topology-design.md section 11.10).
use crate::brep_graph::copy_remap::CopyRemap;
use crate::brep_graph::layer::{AttachedGraph, BRepGraphLayer, LayerBase, LayerType};
use crate::brep_graph::{BRepGraph, ItemDomain, ItemId, NodeId, RefId};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::FlatDataMap;
use rustycad_tkernel::standard::{Failure, Guid, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::OnceLock;

mod copy;
mod ownership;
mod roots;
mod scoped_edit;

pub use scoped_edit::ScopedOwnerEdit;

/// `NCollection_FlatDataMap<BRepGraph_ItemId, Standard_GUID>`: the root entries of one
/// domain.
pub(crate) type OwnerMap = FlatDataMap<ItemId, Guid>;

// A snapshot of the root entries of one domain, in map order.
type RootEntries = Vec<(ItemId, Guid)>;

// `myNodeOwners` and `myRefOwners`.
#[derive(Default)]
struct RootOwners {
    nodes: OwnerMap,
    refs: OwnerMap,
}

impl RootOwners {
    // The map of an item's domain (references in `refs`, everything else in `nodes`).
    fn map_of(&self, item: ItemId) -> &OwnerMap {
        match item.item_domain() {
            ItemDomain::Reference => &self.refs,
            _ => &self.nodes,
        }
    }

    fn map_of_mut(&mut self, item: ItemId) -> &mut OwnerMap {
        match item.item_domain() {
            ItemDomain::Reference => &mut self.refs,
            _ => &mut self.nodes,
        }
    }
}

/// `BRepGraph_LayerLock`: root owner entries (a GUID per node or reference root) and
/// the owned bits they propagate. `HasOwner` reads the bit; `FindOwnerId` climbs to the
/// root entry. A root overlapping an ancestor root of another owner is rejected.
///
/// ```
/// use rustycad_tkbrep::brep_graph::layer_lock::LayerLock;
/// use rustycad_tkbrep::brep_graph::{BRepGraph, NodeId, NodeKind};
/// use rustycad_tkernel::standard::Guid;
/// let graph = BRepGraph::new();
/// let lock = LayerLock::new();
/// let face = NodeId::new(NodeKind::FACE, 0);
/// let owner = Guid::from_components(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
/// assert!(lock.set_owner_with_revision(&graph, face.into(), &owner, true).unwrap());
/// assert_eq!(lock.find_owner_id_node(&graph, face).unwrap(), Some(owner));
/// assert!(lock.has_owners() && !lock.has_owner_node(&graph, face));
/// ```
#[derive(Default)]
pub struct LayerLock {
    transient: StandardTransient,
    base: LayerBase,
    owners: SharedState<RootOwners>,
}
impl_transient!(LayerLock, "BRepGraph_LayerLock", dyn BRepGraphLayer, transient,
    interfaces: [dyn BRepGraphLayer]);

/// `GetID()`: 6f4f4d72-2e64-4ad9-8d02-d3833fe4b9d3.
const LAYER_LOCK_ID: Guid = Guid::from_components(
    0x6f4f4d72_u32 as i32,
    0x2e64,
    0x4ad9,
    0x8d02,
    0xd3,
    0x83,
    0x3f,
    0xe4,
    0xb9,
    0xd3,
);

impl LayerType for LayerLock {
    fn get_id() -> Guid {
        LAYER_LOCK_ID
    }
}

impl LayerLock {
    /// `BRepGraph_LayerLock()`: no owner, detached.
    pub fn new() -> Self {
        Self::default()
    }

    /// `FindOwnerId(theItem, theOwnerId)`: the direct entry of the item, else the entry
    /// of the root that covers it (found through the attached graph); `None` when there
    /// is none. The parent walks can raise as the explorers do.
    pub fn find_owner_id(&self, graph: &BRepGraph, item: ItemId) -> Result<Option<Guid>, Failure> {
        if !item.is_valid() || item.item_domain() == ItemDomain::None {
            return Ok(None);
        }
        if let Some(direct) = self.owners.read().map_of(item).seek(&item).copied() {
            return Ok(Some(direct));
        }
        let root = match item.item_domain() {
            ItemDomain::Reference => self.find_root_ref_id(graph, item.ref_id())?,
            _ => self.find_root_node_id(graph, item.node_id())?,
        };
        let owners = self.owners.read();
        Ok(root.and_then(|root| owners.map_of(root).seek(&root).copied()))
    }

    /// `FindOwnerId(theNode, theOwnerId)`.
    pub fn find_owner_id_node(
        &self,
        graph: &BRepGraph,
        node: NodeId,
    ) -> Result<Option<Guid>, Failure> {
        self.find_owner_id(graph, node.into())
    }

    /// `FindOwnerId(theRef, theOwnerId)`.
    pub fn find_owner_id_ref(
        &self,
        graph: &BRepGraph,
        reference: RefId,
    ) -> Result<Option<Guid>, Failure> {
        self.find_owner_id(graph, reference.into())
    }

    /// `HasOwner(theItem)`: the owned bit of the item in the attached graph (false when
    /// detached).
    pub fn has_owner(&self, graph: &BRepGraph, item: ItemId) -> bool {
        let Some(graph) = self.base.attached_graph(graph) else {
            return false;
        };
        if !item.is_valid() {
            return false;
        }
        match item.item_domain() {
            ItemDomain::Node => item.node_id().is_owned(graph),
            ItemDomain::Reference => item.ref_id().is_owned(graph),
            ItemDomain::None => false,
        }
    }

    /// `HasOwner(theNode)`.
    pub fn has_owner_node(&self, graph: &BRepGraph, node: NodeId) -> bool {
        self.has_owner(graph, node.into())
    }

    /// `HasOwner(theRef)`.
    pub fn has_owner_ref(&self, graph: &BRepGraph, reference: RefId) -> bool {
        self.has_owner(graph, reference.into())
    }

    /// `HasOwners()`: at least one root entry.
    pub fn has_owners(&self) -> bool {
        let owners = self.owners.read();
        !owners.nodes.is_empty() || !owners.refs.is_empty()
    }

    /// `ReserveOwners(theNbOwners)`: grows both root maps by the count.
    pub fn reserve_owners(&self, count: usize) {
        let mut owners = self.owners.write();
        let nodes = owners.nodes.extent() + count;
        owners.nodes.reserve(nodes);
        let refs = owners.refs.extent() + count;
        owners.refs.reserve(refs);
    }

    /// `TouchOwners()`: publishes a bulk update (one revision bump).
    pub fn touch_owners(&self) {
        self.base.touch();
    }

    // The root entries of both domains, copied in map order (nodes first).
    fn root_entries(&self) -> (RootEntries, RootEntries) {
        let owners = self.owners.read();
        let copy = |map: &OwnerMap| map.items().map(|(item, owner)| (*item, *owner)).collect();
        (copy(&owners.nodes), copy(&owners.refs))
    }
}

impl BRepGraphLayer for LayerLock {
    fn layer_base(&self) -> &LayerBase {
        &self.base
    }

    fn id(&self) -> Guid {
        LAYER_LOCK_ID
    }

    fn name(&self) -> &AsciiString {
        static NAME: OnceLock<AsciiString> = OnceLock::new();
        NAME.get_or_init(|| AsciiString::new("Lock"))
    }

    fn on_node_removed(&self, attached: AttachedGraph<'_>, node: NodeId) {
        self.drop_node_root(attached, node);
    }

    fn on_node_replaced(&self, attached: AttachedGraph<'_>, old: NodeId, new: NodeId) {
        self.move_node_root(attached, old, new);
    }

    fn on_ref_removed(&self, attached: AttachedGraph<'_>, reference: RefId) {
        self.drop_ref_root(attached, reference);
    }

    fn copy_to(&self, copy: &CopyRemap<'_>) -> Result<(), Failure> {
        self.copy_roots_to(copy)
    }

    /// `Clear()`.
    fn invalidate_all(&self, attached: AttachedGraph<'_>) {
        self.clear(attached);
    }

    /// Clears the bits of every root and its descendants, then drops the entries.
    fn clear(&self, attached: AttachedGraph<'_>) {
        self.clear_roots(attached);
    }
}
