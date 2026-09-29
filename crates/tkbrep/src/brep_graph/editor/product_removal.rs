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
// Rust translation of EditorView::ProductOps::RemoveOccurrence, RemoveOccurrences and
// RemoveShapeRoot of ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The removals of `EditorView::ProductOps`.
use super::helpers::{has_any_active_usage, is_active_node};
use super::products::ProductOps;
use super::removals::detach_ordered_parent_ref;
use crate::brep_graph::*;
use crate::brep_graph_inc::Storage;
use rustycad_tkernel::standard::Failure;

impl ProductOps<'_> {
    /// `RemoveOccurrence(theProductDefId, theOccurrenceRefId)`: detaches one active
    /// occurrence use of an active product (removing the occurrence subgraph when it
    /// has no other use); a child product left without an active parent becomes a
    /// document root again (appended, even when already listed, as natively). False
    /// for an inactive use (and for ids out of range, the release behaviour of the
    /// native assertions).
    pub fn remove_occurrence(
        &mut self,
        product: ProductId,
        reference: OccurrenceRefId,
    ) -> Result<bool, Failure> {
        let storage = self.graph.inc_storage();
        if !product.is_valid_for(storage.nb_products())
            || !reference.is_valid_for(storage.nb_occurrence_refs())
            || !is_active_node(self.graph, product)
        {
            return Ok(false);
        }
        let occurrence = storage.occurrence_ref(reference).child_occurrence_id;
        let listed = storage
            .product_relations(product)
            .occurrence_ref_ids
            .contains(&reference);
        if storage.is_removed(reference)
            || !listed
            || !occurrence.is_valid()
            || !is_active_node(self.graph, occurrence)
        {
            return Ok(false);
        }
        if !detach_ordered_parent_ref(
            self.graph,
            reference.into(),
            listed,
            occurrence.into(),
            true,
        )? {
            return Ok(false);
        }
        self.restore_root_of_child(occurrence)?;
        self.graph.mark_modified(product.into());
        Ok(true)
    }

    // A product child of the occurrence without any active use is a root again.
    fn restore_root_of_child(&mut self, occurrence: OccurrenceId) -> Result<(), Failure> {
        let storage = self.graph.inc_storage();
        let child = storage.occurrence(occurrence).child_node_id;
        if !child.is_valid() || child.kind != NodeKind::PRODUCT {
            return Ok(());
        }
        let product = ProductId::from_node_id(child);
        let alive = product.is_valid_for(storage.nb_products()) && !storage.is_removed(product);
        if alive && !has_any_active_usage(self.graph, child)? {
            self.graph
                .inc_storage_mut()
                .change_root_product_ids()
                .push(product);
        }
        Ok(())
    }

    /// `RemoveOccurrences(theProductId, theOccurrenceRefs)`: all or nothing.
    pub fn remove_occurrences(
        &mut self,
        product: ProductId,
        references: &[OccurrenceRefId],
    ) -> Result<bool, Failure> {
        if !is_active_node(self.graph, product) {
            return Ok(false);
        }
        let storage = self.graph.inc_storage();
        let valid = |reference: &OccurrenceRefId| {
            reference.is_valid_for(storage.nb_occurrence_refs())
                && !storage.is_removed(*reference)
                && storage
                    .product_relations(product)
                    .occurrence_ref_ids
                    .contains(reference)
        };
        if !references.iter().all(valid) {
            return Ok(false);
        }
        for reference in references {
            if !self.remove_occurrence(product, *reference)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `RemoveShapeRoot(theProductDefId)`: detaches the first active occurrence of the
    /// product whose child is a topology kind and removes the root subgraph when no
    /// other use remains; false when the first such occurrence has no active root (or
    /// the product is inactive or out of range).
    pub fn remove_shape_root(&mut self, product: ProductId) -> Result<bool, Failure> {
        let storage = self.graph.inc_storage();
        if !product.is_valid_for(storage.nb_products()) || !is_active_node(self.graph, product) {
            return Ok(false);
        }
        let Some((reference, root)) = find_shape_root(storage, product) else {
            return Ok(false);
        };
        if !root.is_valid() || !is_active_node(self.graph, root) {
            return Ok(false);
        }
        let occurrence = storage.occurrence_ref(reference).child_occurrence_id;
        // The native call ignores the detachment result.
        detach_ordered_parent_ref(self.graph, reference.into(), true, occurrence.into(), true)?;
        if !has_any_active_usage(self.graph, root)? {
            self.graph.editor().generic().remove_subgraph(root)?;
        }
        self.graph.mark_modified(product.into());
        Ok(true)
    }
}

// The first active reference of the product whose active occurrence has a child of a
// topology kind (the invalid child id has the Solid kind, so it stops the scan too).
fn find_shape_root(storage: &Storage, product: ProductId) -> Option<(OccurrenceRefId, NodeId)> {
    for reference in &storage.product_relations(product).occurrence_ref_ids {
        let occurrence = storage.occurrence_ref(*reference).child_occurrence_id;
        if storage.is_removed(*reference) {
            continue;
        }
        let child = storage.occurrence(occurrence).child_node_id;
        if !storage.is_removed(occurrence) && NodeKind::is_topology_kind(child.kind) {
            return Some((*reference, child));
        }
    }
    None
}
