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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_Layer.hxx and
// BRepGraph_Layer.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_Layer`: the base of the named attribute layers a graph notifies.
use crate::brep_graph::{BRepGraph, ItemDomain, ItemId, NodeId, NodeKind, RefId, RefKind};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::{
    Failure, Guid, Handle, RuntimeTyped, StandardTransient, Transient,
};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::atomic::{AtomicU64, Ordering};

/// The graph a registry hands to a layer callback: the attached graph, or `None` for a
/// detached layer (the native null `AttachedGraph()`), docs/topology-design.md 11.9.
pub type AttachedGraph<'g> = Option<&'g BRepGraph>;

/// The protected state of `BRepGraph_Layer`: the attachment (`myGraph`, kept as the
/// graph's attachment token, 0 when detached) and the revision counter.
///
/// ```
/// use rustycad_tkbrep::brep_graph::layer::LayerBase;
/// let base = LayerBase::new();
/// base.touch();
/// assert_eq!(base.revision(), 1);
/// assert!(!base.is_attached());
/// ```
#[derive(Debug, Default)]
pub struct LayerBase {
    graph_token: AtomicU64,
    revision: AtomicU64,
}

impl LayerBase {
    /// `BRepGraph_Layer()`: detached, revision 0.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Revision()`: bumped by every observable state change.
    pub fn revision(&self) -> u64 {
        self.revision.load(Ordering::Acquire)
    }

    /// `touch()` (protected): bump the revision; derived layers call it from their
    /// mutators.
    pub fn touch(&self) {
        self.revision.fetch_add(1, Ordering::AcqRel);
    }

    /// `IsAttached()` (protected): registered in a registry owned by a graph.
    pub fn is_attached(&self) -> bool {
        self.graph_token() != 0
    }

    /// `AttachedGraph()` (protected): `graph` when this layer is attached to it, `None`
    /// otherwise (the native null pointer of a detached layer).
    pub fn attached_graph<'g>(&self, graph: &'g BRepGraph) -> AttachedGraph<'g> {
        let token = self.graph_token();
        (token != 0 && token == graph.attachment_token()).then_some(graph)
    }

    /// `Graph()` (protected): the attached graph; Standard_ProgramError when the layer is
    /// detached (or attached to another graph than `graph`).
    pub fn graph<'g>(&self, graph: &'g BRepGraph) -> Result<&'g BRepGraph, Failure> {
        self.attached_graph(graph).ok_or_else(|| {
            Failure::with_type(
                "Standard_ProgramError",
                &format!(
                    "BRepGraph_Layer: layer is detached from graph (attachment token {}, \
                     expected the token {} of the given graph)",
                    self.graph_token(),
                    graph.attachment_token()
                ),
                "",
            )
        })
    }

    pub(crate) fn graph_token(&self) -> u64 {
        self.graph_token.load(Ordering::Acquire)
    }

    fn set_graph_token(&self, token: u64) {
        self.graph_token.store(token, Ordering::Release);
    }
}

/// `BRepGraph_Layer`: per-node and per-reference metadata under a unique GUID and
/// name, with the lifecycle callbacks the graph dispatches (removal, replacement,
/// modification). Every callback defaults to a no-op; `SubscribedKinds` and
/// `SubscribedRefKinds` default to 0 (no modification event). `CopyTo` waits for
/// `BRepGraph_CopyRemap` (docs/topology-design.md 11.9).
///
/// ```
/// use rustycad_tkbrep::brep_graph::layer::{BRepGraphLayer, kind_bit};
/// use rustycad_tkbrep::brep_graph::layer_history::LayerHistory;
/// use rustycad_tkbrep::brep_graph::NodeKind;
/// let history = LayerHistory::new();
/// assert!(*history.name() == "History");
/// assert_eq!(history.subscribed_kinds(), 0);
/// assert_eq!(kind_bit(NodeKind::FACE), 4);
/// ```
pub trait BRepGraphLayer: Transient {
    /// The embedded `BRepGraph_Layer` state.
    fn layer_base(&self) -> &LayerBase;

    /// `ID()`: the layer type identity, unique within a graph.
    fn id(&self) -> Guid;

    /// `Name()`: the layer display name.
    fn name(&self) -> &AsciiString;

    /// `OnNodeRemoved(theNode)`: a node was removed without replacement.
    fn on_node_removed(&self, _attached: AttachedGraph<'_>, _node: NodeId) {}

    /// `OnNodeReplaced(theOldNode, theNewNode)`: a node was removed and replaced.
    fn on_node_replaced(&self, _attached: AttachedGraph<'_>, _old: NodeId, _new: NodeId) {}

    /// `InvalidateAll()`: mark every cached value dirty.
    fn invalidate_all(&self, attached: AttachedGraph<'_>);

    /// `Clear()`: drop all stored data.
    fn clear(&self, attached: AttachedGraph<'_>);

    /// `SubscribedKinds()`: the bit mask of node kinds whose modifications this layer
    /// receives (constant for the layer's lifetime).
    fn subscribed_kinds(&self) -> i32 {
        0
    }

    /// `OnNodeModified(theNode)`: a subscribed node was modified (immediate mode).
    fn on_node_modified(&self, _attached: AttachedGraph<'_>, _node: NodeId) {}

    /// `OnNodesModified(theModifiedNodes)`: the nodes modified in a deferred scope.
    fn on_nodes_modified(&self, _attached: AttachedGraph<'_>, _nodes: &[NodeId]) {}

    /// `SubscribedRefKinds()`: the bit mask of reference kinds whose modifications this
    /// layer receives.
    fn subscribed_ref_kinds(&self) -> i32 {
        0
    }

    /// `OnRefRemoved(theRef)`: a reference was removed (dispatched to every layer).
    fn on_ref_removed(&self, _attached: AttachedGraph<'_>, _reference: RefId) {}

    /// `OnRefModified(theRef)`: a subscribed reference was modified (immediate mode).
    fn on_ref_modified(&self, _attached: AttachedGraph<'_>, _reference: RefId) {}

    /// `OnRefsModified(theModifiedRefs)`: the references modified in a deferred scope.
    fn on_refs_modified(&self, _attached: AttachedGraph<'_>, _references: &[RefId]) {}

    /// `OnAttached()` (protected): called after the registry attached this layer.
    fn on_attached(&self) {}

    /// `OnDetached()` (protected): called before the registry detaches this layer.
    fn on_detached(&self) {}

    /// `Revision()`: the monotonic revision counter.
    fn revision(&self) -> u64 {
        self.layer_base().revision()
    }

    /// `OnItemRemoved(theItem)`: the typed removal callback of a valid item.
    fn on_item_removed(&self, attached: AttachedGraph<'_>, item: ItemId) {
        if !item.is_valid() {
            return;
        }
        match item.item_domain() {
            ItemDomain::Node => self.on_node_removed(attached, item.node_id()),
            ItemDomain::Reference => self.on_ref_removed(attached, item.ref_id()),
            ItemDomain::None => {}
        }
    }

    /// `OnItemModified(theItem)`: the typed modification callback of a valid item.
    fn on_item_modified(&self, attached: AttachedGraph<'_>, item: ItemId) {
        if !item.is_valid() {
            return;
        }
        match item.item_domain() {
            ItemDomain::Node => self.on_node_modified(attached, item.node_id()),
            ItemDomain::Reference => self.on_ref_modified(attached, item.ref_id()),
            ItemDomain::None => {}
        }
    }
}
impl_transient_interface!(dyn BRepGraphLayer, "BRepGraph_Layer", StandardTransient);

/// A concrete layer class with a static type GUID (`T::GetID()`), which the registry's
/// typed `Find<T>` and `Ensure<T>` read; `Ensure` creates it with `Default`.
pub trait LayerType: BRepGraphLayer + RuntimeTyped + Default + Sized {
    /// `GetID()`: the fixed layer type GUID.
    fn get_id() -> Guid;
}

/// `KindBit(theKind)`: `1 << kind`; `kind_bit(NodeKind::EDGE) == 16`.
pub fn kind_bit(kind: NodeKind) -> i32 {
    1_i32.wrapping_shl(kind.0 as u32)
}

/// `RefKindBit(theKind)`: `1 << kind`; `ref_kind_bit(RefKind::FACE) == 2`.
pub fn ref_kind_bit(kind: RefKind) -> i32 {
    1_i32.wrapping_shl(kind.0 as u32)
}

/// `attachGraph(theGraph)` (friend of the registry): detach from a previous graph,
/// store the new token, then `OnAttached` for a graph (token 0 leaves it detached).
pub(crate) fn attach_graph(layer: &dyn BRepGraphLayer, token: u64) {
    if layer.layer_base().is_attached() {
        detach_context(layer);
    }
    layer.layer_base().set_graph_token(token);
    if token != 0 {
        layer.on_attached();
    }
}

/// `detachContext()` (friend of the registry): `OnDetached`, then forget the graph.
pub(crate) fn detach_context(layer: &dyn BRepGraphLayer) {
    if !layer.layer_base().is_attached() {
        return;
    }
    layer.on_detached();
    layer.layer_base().set_graph_token(0);
}

/// Views a handle of a concrete layer as the base layer handle the registry stores.
pub fn as_layer<T: BRepGraphLayer + RuntimeTyped>(layer: &Handle<T>) -> Handle<dyn BRepGraphLayer> {
    layer.cast::<dyn BRepGraphLayer>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_bits_shift_the_raw_kind() {
        assert_eq!(kind_bit(NodeKind::VERTEX), 32);
        assert_eq!(kind_bit(NodeKind::OCCURRENCE), 1 << 11);
        assert_eq!(
            ref_kind_bit(RefKind::OCCURRENCE),
            1 << RefKind::OCCURRENCE.0
        );
    }

    #[test]
    fn a_detached_base_refuses_the_graph() {
        let base = LayerBase::new();
        let graph = BRepGraph::new();
        assert!(base.attached_graph(&graph).is_none());
        let error = base.graph(&graph).unwrap_err();
        assert_eq!(error.exception_type(), "Standard_ProgramError");
        base.set_graph_token(graph.attachment_token());
        assert!(base.graph(&graph).is_ok());
    }
}
