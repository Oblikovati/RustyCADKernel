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
// Rust translation of EditorView::ProductOps::Add, AppendDocumentRoot and Append of
// ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The product operations of the editor used by shape ingestion.
use super::{is_active_node, is_active_topology_node, require_mutable};
use crate::brep_graph::{BRepGraph, NodeId, OccurrenceId, OccurrenceRefId, ProductId};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::toploc::Location;

/// `EditorView::ProductOps`.
#[derive(Debug)]
pub struct ProductOps<'g> {
    pub(super) graph: &'g mut BRepGraph,
}

impl ProductOps<'_> {
    /// `Add(theShapeRoot, thePlacement)`: a part product with one occurrence of the
    /// active topology root, placed by `placement`; invalid for another root.
    ///
    /// ```
    /// use rustycad_tkbrep::brep_graph::{BRepGraph, NodeId};
    /// use rustycad_tkmath::toploc::Location;
    /// let mut graph = BRepGraph::new();
    /// let product = graph.editor().products().add(NodeId::default(), &Location::default());
    /// assert!(!product.is_valid());
    /// ```
    pub fn add(&mut self, shape_root: NodeId, placement: &Location) -> ProductId {
        if !is_active_topology_node(self.graph, shape_root) {
            return ProductId::default();
        }
        let product = self.graph.inc_storage_mut().append_product();
        self.graph.allocate_uid(product.into());
        let occurrence = self.graph.inc_storage_mut().append_occurrence();
        self.graph
            .inc_storage_mut()
            .change_occurrence(occurrence)
            .child_node_id = shape_root;
        // The native kind assertion cannot fail after the topology check above.
        self.graph.allocate_uid(occurrence.into());
        let storage = self.graph.inc_storage_mut();
        let occurrence_ref = storage.attach_occurrence_to_product(product, occurrence, placement);
        self.graph.allocate_ref_uid(occurrence_ref.into());
        self.graph.mark_modified(occurrence.into());
        self.graph.mark_ref_modified(occurrence_ref.into());
        product
    }

    /// `Add()`: an empty product.
    pub fn add_empty(&mut self) -> ProductId {
        let product = self.graph.inc_storage_mut().append_product();
        self.graph.allocate_uid(product.into());
        self.graph.mark_modified(product.into());
        product
    }

    /// `AppendDocumentRoot(theProductId)`: records an active product as a document root
    /// once; a locked or guarded product raises Standard_ProgramError.
    pub fn append_document_root(&mut self, product: ProductId) -> Result<(), Failure> {
        let storage = self.graph.inc_storage();
        if !product.is_valid_for(storage.nb_products()) || storage.is_removed(product) {
            return Ok(());
        }
        require_mutable(
            self.graph,
            product.into(),
            "BRepGraph::EditorView::AppendDocumentRoot(): locked product",
            "BRepGraph::EditorView::AppendDocumentRoot(): guard active on product",
        )?;
        let roots = self.graph.inc_storage_mut().change_root_product_ids();
        if roots.contains(&product) {
            return Ok(());
        }
        roots.push(product);
        self.graph.mark_modified(product.into());
        Ok(())
    }

    /// `Append(theParentProduct, theReferencedProduct, thePlacement,
    /// theParentOccurrence, theOutOccurrenceRefId)`: an occurrence of the referenced
    /// product under the parent (which leaves the document roots); the invalid ids when
    /// a product is inactive, the two are the same, or the parent occurrence does not
    /// instantiate the parent. Locked or guarded parents raise Standard_ProgramError.
    pub fn append(
        &mut self,
        parent: ProductId,
        referenced: ProductId,
        placement: &Location,
        parent_occurrence: OccurrenceId,
    ) -> Result<(OccurrenceId, OccurrenceRefId), Failure> {
        let invalid = (OccurrenceId::default(), OccurrenceRefId::default());
        if !self.can_append(parent, referenced, parent_occurrence) {
            return Ok(invalid);
        }
        self.require_append_parents(parent, parent_occurrence)?;
        let occurrence = self.graph.inc_storage_mut().append_occurrence();
        self.graph
            .inc_storage_mut()
            .change_occurrence(occurrence)
            .child_node_id = referenced.into();
        self.graph.allocate_uid(occurrence.into());
        let storage = self.graph.inc_storage_mut();
        let occurrence_ref = storage.attach_occurrence_to_product(parent, occurrence, placement);
        self.graph.allocate_ref_uid(occurrence_ref.into());
        self.graph
            .inc_storage_mut()
            .change_root_product_ids()
            .retain(|root| *root != referenced);
        self.graph.mark_modified(occurrence.into());
        self.graph.mark_ref_modified(occurrence_ref.into());
        self.graph.mark_modified(referenced.into());
        Ok((occurrence, occurrence_ref))
    }

    // The validity checks of `Append` before the lock checks.
    fn can_append(
        &self,
        parent: ProductId,
        referenced: ProductId,
        occurrence: OccurrenceId,
    ) -> bool {
        let graph = &*self.graph;
        if !is_active_node(graph, parent.into()) || !is_active_node(graph, referenced.into()) {
            return false;
        }
        if parent == referenced {
            return false;
        }
        if !occurrence.is_valid() {
            return true;
        }
        is_active_node(graph, occurrence.into())
            && graph.inc_storage().occurrence(occurrence).child_node_id == NodeId::from(parent)
    }

    fn require_append_parents(
        &self,
        parent: ProductId,
        occurrence: OccurrenceId,
    ) -> Result<(), Failure> {
        require_mutable(
            self.graph,
            parent.into(),
            "BRepGraph::EditorView::Append(): locked parent product",
            "BRepGraph::EditorView::Append(): guard active on parent product",
        )?;
        if !occurrence.is_valid() {
            return Ok(());
        }
        require_mutable(
            self.graph,
            occurrence.into(),
            "BRepGraph::EditorView::Append(): locked parent occurrence",
            "BRepGraph::EditorView::Append(): guard active on parent occurrence",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph::NodeKind;

    #[test]
    fn products_wrap_roots_and_nest() {
        let mut graph = BRepGraph::new();
        let solid = graph.inc_storage_mut().append_solid();
        let part = graph
            .editor()
            .products()
            .add(solid.into(), &Location::default());
        assert!(part.is_valid());
        graph
            .editor()
            .products()
            .append_document_root(part)
            .unwrap();
        graph
            .editor()
            .products()
            .append_document_root(part)
            .unwrap();
        assert_eq!(graph.root_product_ids(), &[part]);
        let assembly = graph.editor().products().add_empty();
        let (occurrence, reference) = graph
            .editor()
            .products()
            .append(
                assembly,
                part,
                &Location::default(),
                OccurrenceId::default(),
            )
            .unwrap();
        assert!(occurrence.is_valid() && reference.is_valid());
        assert!(graph.root_product_ids().is_empty());
        assert_eq!(
            graph
                .topo()
                .occurrences()
                .definition(occurrence)
                .child_node_id
                .kind,
            NodeKind::PRODUCT
        );
        let same = graph.editor().products().append(
            part,
            part,
            &Location::default(),
            OccurrenceId::default(),
        );
        assert!(!same.unwrap().0.is_valid());
        graph.inc_storage_mut().set_owned(assembly, true);
        let locked = graph.editor().products().append(
            assembly,
            part,
            &Location::default(),
            OccurrenceId::default(),
        );
        assert_eq!(
            locked.unwrap_err().exception_type(),
            "Standard_ProgramError"
        );
    }
}
