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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_LayerTopoSupplement.hxx
// and BRepGraph_LayerTopoSupplement.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_LayerTopoSupplement`: runtime storage for the TopoDS fragments the core
//! model does not keep, attached to core owner nodes.
use crate::brep_graph::copy_remap::CopyRemap;
use crate::brep_graph::hooks::TopoSupplementHooks;
use crate::brep_graph::layer::{AttachedGraph, BRepGraphLayer, LayerBase, LayerType};
use crate::brep_graph::{BRepGraph, NodeId, NodeKind};
use crate::topods::Shape;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Guid, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::OnceLock;

mod copy;

pub use crate::brep_graph::hooks::AttachmentKind;

/// `BRepGraph_LayerTopoSupplement::Entry`: one stored attachment.
#[derive(Clone, Debug)]
pub struct SupplementEntry {
    /// `BaseOwner`: the core owner node.
    pub base_owner: NodeId,
    /// `LocalUid`: the layer-local uid (0 is never used).
    pub local_uid: u64,
    /// `Kind`: the semantic role.
    pub kind: AttachmentKind,
    /// `Shape`: the attached shape.
    pub shape: Shape,
}

impl Default for SupplementEntry {
    fn default() -> Self {
        Self {
            base_owner: NodeId::default(),
            local_uid: 0,
            kind: AttachmentKind::GenericSupplementShape,
            shape: Shape::default(),
        }
    }
}

// `myEntries`, `myOwnerToUids` and `myNextUid`.
struct SupplementRecords {
    entries: DataMap<u64, SupplementEntry>,
    owner_to_uids: DataMap<NodeId, Vec<u64>>,
    next_uid: u64,
}

impl Default for SupplementRecords {
    fn default() -> Self {
        Self {
            entries: DataMap::default(),
            owner_to_uids: DataMap::default(),
            next_uid: 1,
        }
    }
}

/// `BRepGraph_LayerTopoSupplement`: attachments keyed by a layer-local uid, with an
/// insertion-ordered uid list per owner. Not serialized; it keeps the live
/// `TopoDS -> Graph -> TopoDS` round trip.
///
/// ```
/// use rustycad_tkbrep::brep_graph::layer_topo_supplement::{AttachmentKind, LayerTopoSupplement};
/// use rustycad_tkbrep::brep_graph::{BRepGraph, NodeId, NodeKind};
/// use rustycad_tkbrep::topods::{Builder, Compound};
/// let graph = BRepGraph::new();
/// let layer = LayerTopoSupplement::new();
/// let mut fragment = Compound::default();
/// Builder.make_compound(&mut fragment);
/// let owner = NodeId::new(NodeKind::EDGE, 0);
/// let kind = AttachmentKind::GenericSupplementShape;
/// let uid = layer.add_attachment(&graph, owner, kind, &fragment);
/// assert_eq!(uid, 1);
/// assert_eq!(layer.attached_to(owner), vec![1]);
/// ```
#[derive(Default)]
pub struct LayerTopoSupplement {
    transient: StandardTransient,
    base: LayerBase,
    records: SharedState<SupplementRecords>,
}
impl_transient!(LayerTopoSupplement, "BRepGraph_LayerTopoSupplement", dyn BRepGraphLayer, transient,
    interfaces: [dyn BRepGraphLayer]);

/// `GetID()`: bd76a339-9958-4d92-a3de-1296bd67a992.
const LAYER_TOPO_SUPPLEMENT_ID: Guid = Guid::from_components(
    0xbd76a339_u32 as i32,
    0x9958,
    0x4d92,
    0xa3de,
    0x12,
    0x96,
    0xbd,
    0x67,
    0xa9,
    0x92,
);

impl LayerType for LayerTopoSupplement {
    fn get_id() -> Guid {
        LAYER_TOPO_SUPPLEMENT_ID
    }
}

impl LayerTopoSupplement {
    /// An empty detached layer; `LayerTopoSupplement::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `FindByUid(theUid)`: a copy of the entry, `None` when absent.
    pub fn find_by_uid(&self, uid: u64) -> Option<SupplementEntry> {
        self.records.read().entries.seek(&uid).cloned()
    }

    /// `AttachedTo(theOwner)`: the owner's uids in insertion order (a copy).
    pub fn attached_to(&self, owner: NodeId) -> Vec<u64> {
        let records = self.records.read();
        records
            .owner_to_uids
            .seek(&owner)
            .cloned()
            .unwrap_or_default()
    }

    /// `AddAttachment(theOwner, theKind, theShape)`: attaches under the next uid; the
    /// uid, or 0 when refused. `graph` is the attached graph whose cached reconstruction
    /// of the owner is cleared (ignored when this layer is not attached to it).
    pub fn add_attachment(
        &self,
        graph: &BRepGraph,
        owner: NodeId,
        kind: AttachmentKind,
        shape: &Shape,
    ) -> u64 {
        self.add_attachment_in(self.base.attached_graph(graph), owner, kind, shape)
    }

    fn add_attachment_in(
        &self,
        attached: AttachedGraph<'_>,
        owner: NodeId,
        kind: AttachmentKind,
        shape: &Shape,
    ) -> u64 {
        let uid = self.records.read().next_uid;
        if !self.add_attachment_with_uid_in(attached, owner, uid, kind, shape) {
            return 0;
        }
        uid
    }

    /// `AddAttachmentWithUid(theOwner, theUid, theKind, theShape)`: attaches under a
    /// preserved uid; false for an unsupported or incompatible owner, a null shape, uid
    /// 0 or a uid in use.
    pub fn add_attachment_with_uid(
        &self,
        graph: &BRepGraph,
        owner: NodeId,
        uid: u64,
        kind: AttachmentKind,
        shape: &Shape,
    ) -> bool {
        self.add_attachment_with_uid_in(self.base.attached_graph(graph), owner, uid, kind, shape)
    }

    fn add_attachment_with_uid_in(
        &self,
        attached: AttachedGraph<'_>,
        owner: NodeId,
        uid: u64,
        kind: AttachmentKind,
        shape: &Shape,
    ) -> bool {
        if !self.store_attachment(owner, uid, kind, shape) {
            return false;
        }
        if let Some(graph) = attached {
            graph.shapes().clear_cached(owner);
        }
        self.base.touch();
        true
    }

    // The validation and the bookkeeping of `AddAttachmentWithUid`, under the lock.
    fn store_attachment(
        &self,
        owner: NodeId,
        uid: u64,
        kind: AttachmentKind,
        shape: &Shape,
    ) -> bool {
        let mut records = self.records.write();
        let accepted = owner.is_valid()
            && is_supported_owner_kind(owner.kind)
            && !shape.is_null()
            && is_attachment_kind_compatible(owner.kind, kind)
            && uid != 0
            && !records.entries.is_bound(&uid);
        if !accepted {
            return false;
        }
        let entry = SupplementEntry {
            base_owner: owner,
            local_uid: uid,
            kind,
            shape: shape.clone(),
        };
        records.entries.bind(uid, entry);
        match records.owner_to_uids.change_seek(&owner) {
            Some(uids) => uids.push(uid),
            None => {
                records.owner_to_uids.bind(owner, vec![uid]);
            }
        }
        if uid >= records.next_uid {
            records.next_uid = uid + 1;
        }
        true
    }

    /// `RemoveAttachment(theUid)`: removes the attachment, keeping the owner's other
    /// uids in order; false when the uid is unknown.
    pub fn remove_attachment(&self, graph: &BRepGraph, uid: u64) -> bool {
        let Some(owner) = self.unlink_attachment(uid) else {
            return false;
        };
        if let Some(graph) = self.base.attached_graph(graph) {
            graph.shapes().clear_cached(owner);
        }
        self.records.write().entries.unbind(&uid);
        self.base.touch();
        true
    }

    // The owner-list part of `RemoveAttachment`: the owner of a known uid, removed
    // from its owner's list (the list dropped when empty).
    fn unlink_attachment(&self, uid: u64) -> Option<NodeId> {
        let mut records = self.records.write();
        let owner = records.entries.seek(&uid)?.base_owner;
        let now_empty = records.owner_to_uids.change_seek(&owner).map(|uids| {
            if let Some(index) = uids.iter().position(|&stored| stored == uid) {
                uids.remove(index);
            }
            uids.is_empty()
        });
        if now_empty == Some(true) {
            records.owner_to_uids.unbind(&owner);
        }
        Some(owner)
    }

    /// `Validate()`: Standard_ProgramError when an entry is invalid or its owner list
    /// misses it.
    pub fn validate(&self) -> Result<(), Failure> {
        let records = self.records.read();
        for (_, entry) in records.entries.items() {
            validate_entry(&records, entry)?;
        }
        Ok(())
    }

    /// `removeOwner(theOwner)` (private): drops every attachment of the owner.
    fn remove_owner(&self, owner: NodeId) {
        let mut records = self.records.write();
        let Some(uids) = records.owner_to_uids.seek(&owner).cloned() else {
            return;
        };
        for uid in uids {
            records.entries.unbind(&uid);
        }
        records.owner_to_uids.unbind(&owner);
        drop(records);
        self.base.touch();
    }

    // The migration of `OnNodeReplaced`: the old owner's uids appended to the new
    // owner's list and re-owned; false when the old owner has none.
    fn move_owner(&self, old: NodeId, new: NodeId) -> bool {
        let mut records = self.records.write();
        let Some(moved) = records.owner_to_uids.seek(&old).cloned() else {
            return false;
        };
        match records.owner_to_uids.change_seek(&new) {
            Some(list) => list.extend_from_slice(&moved),
            None => {
                records.owner_to_uids.bind(new, moved.clone());
            }
        }
        for uid in &moved {
            if let Some(entry) = records.entries.change_seek(uid) {
                entry.base_owner = new;
            }
        }
        records.owner_to_uids.unbind(&old);
        true
    }
}

// One entry of `Validate`.
fn validate_entry(records: &SupplementRecords, entry: &SupplementEntry) -> Result<(), Failure> {
    let owner = entry.base_owner;
    if entry.local_uid == 0
        || !owner.is_valid()
        || !is_supported_owner_kind(owner.kind)
        || entry.shape.is_null()
        || !is_attachment_kind_compatible(owner.kind, entry.kind)
    {
        return Err(validate_error("invalid entry", entry));
    }
    let Some(uids) = records.owner_to_uids.seek(&owner) else {
        return Err(validate_error("missing owner index", entry));
    };
    if !uids.contains(&entry.local_uid) {
        return Err(validate_error("missing owner linkage", entry));
    }
    Ok(())
}

fn validate_error(reason: &str, entry: &SupplementEntry) -> Failure {
    Failure::with_type(
        "Standard_ProgramError",
        &format!(
            "BRepGraph_LayerTopoSupplement::Validate() - {reason} (uid {}, owner {:?}, kind {:?})",
            entry.local_uid, entry.base_owner, entry.kind
        ),
        "",
    )
}

impl BRepGraphLayer for LayerTopoSupplement {
    fn layer_base(&self) -> &LayerBase {
        &self.base
    }

    fn id(&self) -> Guid {
        LAYER_TOPO_SUPPLEMENT_ID
    }

    fn name(&self) -> &AsciiString {
        static NAME: OnceLock<AsciiString> = OnceLock::new();
        NAME.get_or_init(|| AsciiString::new("LayerTopoSupplement"))
    }

    /// Drops the removed node's attachments.
    fn on_node_removed(&self, _attached: AttachedGraph<'_>, node: NodeId) {
        self.remove_owner(node);
    }

    /// Moves the attachments to a replacement of the same kind, else drops them.
    fn on_node_replaced(&self, _attached: AttachedGraph<'_>, old: NodeId, new: NodeId) {
        if !old.is_valid() {
            return;
        }
        if !new.is_valid() || old.kind != new.kind {
            self.remove_owner(old);
            return;
        }
        if self.move_owner(old, new) {
            self.base.touch();
        }
    }

    /// `Clear()`.
    fn copy_to(&self, copy: &CopyRemap<'_>) -> Result<(), Failure> {
        self.copy_attachments_to(copy)
    }

    fn invalidate_all(&self, attached: AttachedGraph<'_>) {
        self.clear(attached);
    }

    /// Removes every attachment and restarts the uids at 1.
    fn clear(&self, _attached: AttachedGraph<'_>) {
        let mut records = self.records.write();
        records.entries.clear(false);
        records.owner_to_uids.clear(false);
        records.next_uid = 1;
        drop(records);
        self.base.touch();
    }
}

impl TopoSupplementHooks for LayerTopoSupplement {
    fn add_attachment(
        &self,
        owner: NodeId,
        kind: crate::brep_graph::hooks::AttachmentKind,
        shape: &Shape,
    ) -> u64 {
        self.add_attachment_in(None, owner, kind, shape)
    }

    fn attached_shapes(&self, owner: NodeId) -> Vec<Shape> {
        let records = self.records.read();
        let Some(uids) = records.owner_to_uids.seek(&owner) else {
            return Vec::new();
        };
        uids.iter()
            .filter_map(|uid| records.entries.seek(uid))
            .filter(|entry| !entry.shape.is_null())
            .map(|entry| entry.shape.clone())
            .collect()
    }
}

// `isSupportedOwnerKind`: the topology kinds that can own attachments.
fn is_supported_owner_kind(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::VERTEX
            | NodeKind::EDGE
            | NodeKind::FACE
            | NodeKind::SHELL
            | NodeKind::SOLID
            | NodeKind::COMP_SOLID
            | NodeKind::COMPOUND
    )
}

// `isAttachmentKindCompatible`: each kind binds one owner kind; the generic kind any
// supported owner.
fn is_attachment_kind_compatible(owner: NodeKind, kind: AttachmentKind) -> bool {
    match kind {
        AttachmentKind::GenericSupplementShape => is_supported_owner_kind(owner),
        AttachmentKind::VertexSupplementShape => owner == NodeKind::VERTEX,
        AttachmentKind::EdgeInternalVertex => owner == NodeKind::EDGE,
        AttachmentKind::FaceDirectVertex => owner == NodeKind::FACE,
        AttachmentKind::SolidAuxShape => owner == NodeKind::SOLID,
        AttachmentKind::ShellAuxShape => owner == NodeKind::SHELL,
        AttachmentKind::CompSolidAuxShape => owner == NodeKind::COMP_SOLID,
        AttachmentKind::CompoundAuxShape => owner == NodeKind::COMPOUND,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topods::{Builder, Compound};

    // A non-null fragment (the kind checks read the owner, not the shape type).
    fn vertex() -> Shape {
        let mut fragment = Compound::default();
        Builder.make_compound(&mut fragment);
        fragment.into()
    }

    #[test]
    fn attachments_keep_owner_order_and_validate() {
        let graph = BRepGraph::new();
        let layer = LayerTopoSupplement::new();
        let edge = NodeId::new(NodeKind::EDGE, 2);
        let generic = AttachmentKind::GenericSupplementShape;
        assert_eq!(layer.add_attachment(&graph, edge, generic, &vertex()), 1);
        assert!(layer.add_attachment_with_uid(&graph, edge, 7, generic, &vertex()));
        assert!(!layer.add_attachment_with_uid(&graph, edge, 7, generic, &vertex()));
        assert_eq!(
            layer.add_attachment(&graph, edge, AttachmentKind::FaceDirectVertex, &vertex()),
            0
        );
        assert_eq!(layer.add_attachment(&graph, edge, generic, &vertex()), 8);
        assert!(layer.remove_attachment(&graph, 7));
        assert_eq!(layer.attached_to(edge), vec![1, 8]);
        assert!(layer.validate().is_ok());
        assert_eq!(layer.revision(), 4);
    }

    #[test]
    fn replacement_moves_or_drops_attachments() {
        let graph = BRepGraph::new();
        let layer = LayerTopoSupplement::new();
        let (a, b) = (
            NodeId::new(NodeKind::FACE, 0),
            NodeId::new(NodeKind::FACE, 1),
        );
        let kind = AttachmentKind::FaceDirectVertex;
        layer.add_attachment(&graph, a, kind, &vertex());
        layer.add_attachment(&graph, b, kind, &vertex());
        layer.on_node_replaced(None, a, b);
        assert_eq!(layer.attached_to(b), vec![2, 1]);
        assert_eq!(layer.find_by_uid(1).unwrap().base_owner, b);
        layer.on_node_replaced(None, b, NodeId::new(NodeKind::EDGE, 0));
        assert!(layer.attached_to(b).is_empty() && layer.find_by_uid(1).is_none());
        layer.clear(None);
        assert_eq!(layer.add_attachment(&graph, a, kind, &vertex()), 1);
    }
}
