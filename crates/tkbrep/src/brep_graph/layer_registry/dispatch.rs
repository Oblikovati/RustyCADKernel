// Rust translation of the Dispatch*, ClearAll and InvalidateAll members of
// ModelingData/TKBRep/BRepGraph/BRepGraph_LayerRegistry.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The event dispatch of the layer registry: every loop reads one slot at a time
//! under the lock and calls the layer without it, stopping at the first empty slot.
use super::LayerRegistry;
use crate::brep_graph::layer::{AttachedGraph, BRepGraphLayer, kind_bit, ref_kind_bit};
use crate::brep_graph::{BRepGraph, ItemDomain, ItemId, NodeId, RefId};

impl LayerRegistry {
    /// The graph the layers are attached to: `graph` when it owns this registry, `None`
    /// for a standalone registry or another graph (docs/topology-design.md 11.9).
    pub fn attached<'g>(&self, graph: Option<&'g BRepGraph>) -> AttachedGraph<'g> {
        graph.filter(|graph| self.graph_token != 0 && graph.attachment_token() == self.graph_token)
    }

    // Calls `visit` on every layer in slot order.
    fn for_each_layer(&self, mut visit: impl FnMut(&dyn BRepGraphLayer)) {
        for slot in 0_u32.. {
            let layer = self.layer_at(slot);
            let Some(layer) = layer.get() else {
                return;
            };
            visit(layer);
        }
    }

    /// `DispatchOnNodeRemoved(theNode)`: `OnNodeRemoved` on every layer.
    pub fn dispatch_on_node_removed(&self, graph: Option<&BRepGraph>, node: NodeId) {
        let attached = self.attached(graph);
        self.for_each_layer(|layer| layer.on_node_removed(attached, node));
    }

    /// `DispatchOnItemRemoved(theItem)`: `OnItemRemoved` on every layer (a no-op for an
    /// invalid item).
    pub fn dispatch_on_item_removed(&self, graph: Option<&BRepGraph>, item: ItemId) {
        if !item.is_valid() {
            return;
        }
        let attached = self.attached(graph);
        self.for_each_layer(|layer| layer.on_item_removed(attached, item));
    }

    /// `DispatchOnNodeReplaced(theOldNode, theNewNode)`: `OnNodeReplaced` on every layer.
    pub fn dispatch_on_node_replaced(&self, graph: Option<&BRepGraph>, old: NodeId, new: NodeId) {
        let attached = self.attached(graph);
        self.for_each_layer(|layer| layer.on_node_replaced(attached, old, new));
    }

    /// `DispatchNodeModified(theNode)`: `OnNodeModified` on the layers subscribed to the
    /// node's kind; nothing when no layer subscribes.
    pub fn dispatch_node_modified(&self, graph: Option<&BRepGraph>, node: NodeId) {
        if !self.has_modification_subscribers() {
            return;
        }
        let (attached, bit) = (self.attached(graph), kind_bit(node.kind));
        self.for_each_layer(|layer| {
            if layer.subscribed_kinds() & bit != 0 {
                layer.on_node_modified(attached, node);
            }
        });
    }

    /// `DispatchItemModified(theItem)`: the node or reference dispatch of a valid item.
    pub fn dispatch_item_modified(&self, graph: Option<&BRepGraph>, item: ItemId) {
        if !item.is_valid() {
            return;
        }
        match item.item_domain() {
            ItemDomain::Node => self.dispatch_node_modified(graph, item.node_id()),
            ItemDomain::Reference => self.dispatch_ref_modified(graph, item.ref_id()),
            ItemDomain::None => {}
        }
    }

    /// `DispatchNodesModified(theModifiedNodes, theModifiedKindsMask)`: `OnNodesModified`
    /// on the layers whose subscription meets the mask.
    pub fn dispatch_nodes_modified(&self, graph: Option<&BRepGraph>, nodes: &[NodeId], mask: i32) {
        if !self.has_modification_subscribers() || mask == 0 {
            return;
        }
        let attached = self.attached(graph);
        self.for_each_layer(|layer| {
            if layer.subscribed_kinds() & mask != 0 {
                layer.on_nodes_modified(attached, nodes);
            }
        });
    }

    /// `DispatchOnRefRemoved(theRef)`: `OnRefRemoved` on every layer (not filtered).
    pub fn dispatch_on_ref_removed(&self, graph: Option<&BRepGraph>, reference: RefId) {
        let attached = self.attached(graph);
        self.for_each_layer(|layer| layer.on_ref_removed(attached, reference));
    }

    /// `DispatchRefModified(theRef)`: `OnRefModified` on the layers subscribed to the
    /// reference's kind.
    pub fn dispatch_ref_modified(&self, graph: Option<&BRepGraph>, reference: RefId) {
        if !self.has_ref_modification_subscribers() {
            return;
        }
        let (attached, bit) = (self.attached(graph), ref_kind_bit(reference.kind));
        self.for_each_layer(|layer| {
            if layer.subscribed_ref_kinds() & bit != 0 {
                layer.on_ref_modified(attached, reference);
            }
        });
    }

    /// `DispatchRefsModified(theModifiedRefs, theModifiedRefKindsMask)`.
    pub fn dispatch_refs_modified(
        &self,
        graph: Option<&BRepGraph>,
        references: &[RefId],
        mask: i32,
    ) {
        if !self.has_ref_modification_subscribers() || mask == 0 {
            return;
        }
        let attached = self.attached(graph);
        self.for_each_layer(|layer| {
            if layer.subscribed_ref_kinds() & mask != 0 {
                layer.on_refs_modified(attached, references);
            }
        });
    }

    /// `ClearAll()`: `Clear` on every layer, keeping the registrations.
    pub fn clear_all(&self, graph: Option<&BRepGraph>) {
        let attached = self.attached(graph);
        self.for_each_layer(|layer| layer.clear(attached));
    }

    /// `InvalidateAll()`: `InvalidateAll` on every layer.
    pub fn invalidate_all(&self, graph: Option<&BRepGraph>) {
        let attached = self.attached(graph);
        self.for_each_layer(|layer| layer.invalidate_all(attached));
    }
}
