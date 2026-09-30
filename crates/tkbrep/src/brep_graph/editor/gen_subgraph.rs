// Rust translation of EditorView::GenOps::RemoveSubgraph and RemoveSubgraphScope of
// ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GenOps::RemoveSubgraph`: the ownership-aware cascading removal.
use super::gen_ops::GenOps;
use super::helpers::{
    has_any_active_usage, has_other_active_parent, node_item, product_has_occurrence_def,
    require_mutable,
};
use crate::brep_graph::child_explorer::{ChildExplorer, TraversalMode};
use crate::brep_graph::refs_iterator::RefsOccurrenceOfProduct;
use crate::brep_graph::*;
use rustycad_tkernel::standard::Failure;

impl GenOps<'_> {
    /// `RemoveSubgraph(theNode)`: removes the node and, first, every descendant that
    /// no other active parent uses (a product cascades into its occurrences; an
    /// occurrence into an otherwise unused topology child, detaching itself from its
    /// parent products and returning an orphaned child product to the document roots).
    /// A locked or guarded node raises Standard_ProgramError.
    ///
    /// ```
    /// use rustycad_tkbrep::brep_graph::BRepGraph;
    /// let mut graph = BRepGraph::new();
    /// let shell = graph.editor().shells().add();
    /// graph.editor().generic().remove_subgraph(shell.into()).unwrap();
    /// assert!(shell.is_removed(&graph));
    /// ```
    pub fn remove_subgraph(&mut self, node: NodeId) -> Result<(), Failure> {
        require_mutable(
            self.graph,
            node_item(node),
            "BRepGraph::EditorView::RemoveSubgraph(): locked node",
            "BRepGraph::EditorView::RemoveSubgraph(): guard active on node",
        )?;
        // `RemoveSubgraphScope`: the depth counter is restored on every exit, errors
        // included, as the native destructor does.
        self.graph
            .inc_storage_mut()
            .increment_remove_subgraph_depth();
        let result = self.remove_subgraph_in_scope(node);
        self.graph
            .inc_storage_mut()
            .decrement_remove_subgraph_depth();
        result
    }

    fn remove_subgraph_in_scope(&mut self, node: NodeId) -> Result<(), Failure> {
        match node.kind {
            NodeKind::PRODUCT => self.remove_product_occurrences(node)?,
            NodeKind::OCCURRENCE => self.remove_occurrence_subgraph(node)?,
            _ => self.remove_direct_children(node)?,
        }
        // The node is removed after its children, so the iterators still saw it as an
        // active parent during the traversal.
        self.remove_node(node)
    }

    // The product case: each occurrence (over a snapshot of their indices) cascades.
    fn remove_product_occurrences(&mut self, node: NodeId) -> Result<(), Failure> {
        let storage = self.graph.inc_storage();
        if !node.is_valid_for(storage.nb_products()) {
            return Ok(());
        }
        let references = RefsOccurrenceOfProduct::new(self.graph, ProductId::from_node_id(node));
        let occurrences: Vec<u32> = references
            .map(|reference| storage.occurrence_ref(reference).child_occurrence_id.index)
            .collect();
        for index in occurrences {
            self.remove_subgraph(OccurrenceId::new(index).into())?;
        }
        Ok(())
    }

    // The occurrence case.
    fn remove_occurrence_subgraph(&mut self, node: NodeId) -> Result<(), Failure> {
        if !node.is_valid_for(self.graph.inc_storage().nb_occurrences()) {
            return Ok(());
        }
        let occurrence = OccurrenceId::from_node_id(node);
        let child = self
            .graph
            .inc_storage()
            .occurrence(occurrence)
            .child_node_id;
        if child.is_valid()
            && NodeKind::is_topology_kind(child.kind)
            && !has_any_active_usage(self.graph, child)?
        {
            self.remove_subgraph(child)?;
        }
        self.detach_from_parent_products(occurrence);
        if child.is_valid() && child.kind == NodeKind::PRODUCT {
            let product = ProductId::from_node_id(child);
            let storage = self.graph.inc_storage();
            let alive = product.is_valid_for(storage.nb_products()) && !storage.is_removed(product);
            if alive && !has_any_active_usage(self.graph, child)? {
                self.graph
                    .inc_storage_mut()
                    .change_root_product_ids()
                    .push(product);
            }
        }
        Ok(())
    }

    // Every product placing the occurrence loses its first active reference to it.
    fn detach_from_parent_products(&mut self, occurrence: OccurrenceId) {
        let storage = self.graph.inc_storage();
        let parents: Vec<ProductId> = (0..storage.nb_products())
            .map(ProductId::new)
            .filter(|product| product_has_occurrence_def(storage, *product, occurrence))
            .collect();
        for product in parents {
            let storage = self.graph.inc_storage();
            if !product.is_valid_for(storage.nb_products()) || storage.is_removed(product) {
                continue;
            }
            let found = storage
                .product_relations(product)
                .occurrence_ref_ids
                .iter()
                .copied()
                .find(|reference| {
                    reference.is_valid_for(storage.nb_occurrence_refs())
                        && !storage.is_removed(*reference)
                        && storage.occurrence_ref(*reference).child_occurrence_id == occurrence
                });
            let Some(reference) = found else {
                continue;
            };
            let storage = self.graph.inc_storage_mut();
            storage.mark_removed_ref(reference.into());
            storage.detach_occurrence_from_product(product, reference);
            self.graph.mark_modified(product.into());
        }
    }

    // The topology case: each direct child without another active parent cascades.
    fn remove_direct_children(&mut self, node: NodeId) -> Result<(), Failure> {
        let mut children = Vec::new();
        let mut explorer =
            ChildExplorer::with_mode(self.graph, node, TraversalMode::DirectChildren)?;
        while explorer.more() {
            children.push(explorer.current().def_id);
            explorer.next()?;
        }
        for child in children {
            if !has_other_active_parent(self.graph, child, node)? {
                self.remove_subgraph(child)?;
            }
        }
        Ok(())
    }
}
