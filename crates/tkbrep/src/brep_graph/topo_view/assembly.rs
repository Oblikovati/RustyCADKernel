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
// Rust translation of the product and occurrence members of
// ModelingData/TKBRep/BRepGraph/BRepGraph_TopoView.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The assembly classification of the product and occurrence groups.
use super::{OccurrenceOps, ProductOps};
use crate::brep_graph::child_refs::active_occurrence_refs;
use crate::brep_graph::{NodeId, NodeKind, OccurrenceId, ProductId};
use crate::brep_graph_inc::OccurrenceDef;
use rustycad_tkmath::toploc::Location;

impl<'g> ProductOps<'g> {
    // The occurrence definitions reached through the product's active references with
    // active occurrences (the loop of `ShapeRoot`, `IsAssembly`, `IsPart`,
    // `ShapeRootNode`; only the product and the references are checked by range).
    fn active_occurrences(self, product: ProductId) -> Vec<&'g OccurrenceDef> {
        let storage = self.graph.inc_storage();
        if !product.is_valid_for(storage.nb_products()) || storage.is_removed(product) {
            return Vec::new();
        }
        storage
            .product_relations(product)
            .occurrence_ref_ids
            .iter()
            .filter(|ref_id| !storage.is_removed(**ref_id))
            .map(|ref_id| storage.occurrence_ref(*ref_id).child_occurrence_id)
            .filter(|occurrence| !storage.is_removed(*occurrence))
            .map(|occurrence| storage.occurrence(occurrence))
            .collect()
    }

    /// `ShapeRoot(theProduct)`: the first active topology root of the product's
    /// occurrences, invalid for an assembly.
    pub fn shape_root(self, product: ProductId) -> NodeId {
        let graph = self.graph;
        self.active_occurrences(product)
            .into_iter()
            .map(|occurrence| occurrence.child_node_id)
            .find(|child| {
                child.is_valid()
                    && !NodeKind::is_assembly_kind(child.kind)
                    && graph.topo_entity(*child).is_some()
                    && !child.is_removed(graph)
            })
            .unwrap_or_default()
    }

    /// `IsAssembly(theProduct)`: an active occurrence references a product.
    pub fn is_assembly(self, product: ProductId) -> bool {
        self.active_occurrences(product)
            .iter()
            .any(|occurrence| occurrence.child_node_id.kind == NodeKind::PRODUCT)
    }

    /// `IsPart(theProduct)`: an active occurrence references a topology node.
    pub fn is_part(self, product: ProductId) -> bool {
        self.active_occurrences(product).iter().any(|occurrence| {
            let child = occurrence.child_node_id;
            child.is_valid() && !NodeKind::is_assembly_kind(child.kind)
        })
    }

    /// `ShapeRootNode(theProduct)`: the first topology child of the active occurrences
    /// (without the entity checks of [`shape_root`](Self::shape_root)).
    pub fn shape_root_node(self, product: ProductId) -> NodeId {
        self.active_occurrences(product)
            .into_iter()
            .map(|occurrence| occurrence.child_node_id)
            .find(|child| child.is_valid() && !NodeKind::is_assembly_kind(child.kind))
            .unwrap_or_default()
    }

    /// `NbComponents(theProduct)`: the active child occurrences.
    pub fn nb_components(self, product: ProductId) -> u32 {
        let storage = self.graph.inc_storage();
        if !product.is_valid_for(storage.nb_products()) || storage.is_removed(product) {
            return 0;
        }
        active_occurrence_refs(self.graph, product).len() as u32
    }

    /// `Component(theProduct, theComponentIdx)`: the i-th active child occurrence.
    pub fn component(self, product: ProductId, component_index: i32) -> OccurrenceId {
        let storage = self.graph.inc_storage();
        if !product.is_valid_for(storage.nb_products()) || component_index < 0 {
            return OccurrenceId::default();
        }
        if storage.is_removed(product) {
            return OccurrenceId::default();
        }
        active_occurrence_refs(self.graph, product)
            .get(component_index as usize)
            .map(|ref_id| storage.occurrence_ref(*ref_id).child_occurrence_id)
            .unwrap_or_default()
    }
}

impl<'g> OccurrenceOps<'g> {
    /// `Product(theOccurrence)`: the active product an active occurrence instantiates.
    pub fn product(self, occurrence: OccurrenceId) -> ProductId {
        let storage = self.graph.inc_storage();
        if !occurrence.is_valid_for(storage.nb_occurrences()) || storage.is_removed(occurrence) {
            return ProductId::default();
        }
        let child = storage.occurrence(occurrence).child_node_id;
        if !child.is_valid() || child.kind != NodeKind::PRODUCT {
            return ProductId::default();
        }
        let product = ProductId::from_node_id(child);
        if !product.is_valid_for(storage.nb_products()) || storage.is_removed(product) {
            return ProductId::default();
        }
        product
    }

    /// `ParentProduct(theOccurrence)`: the first active parent product of an active
    /// occurrence.
    pub fn parent_product(self, occurrence: OccurrenceId) -> ProductId {
        let storage = self.graph.inc_storage();
        if !occurrence.is_valid_for(storage.nb_occurrences()) || storage.is_removed(occurrence) {
            return ProductId::default();
        }
        let refs = &storage
            .occurrence_relations(occurrence)
            .parent_occurrence_ref_ids;
        refs.iter()
            .filter(|ref_id| {
                ref_id.is_valid_for(storage.nb_occurrence_refs()) && !storage.is_removed(**ref_id)
            })
            .map(|ref_id| storage.occurrence_ref(*ref_id).parent_product_id)
            .find(|product| {
                product.is_valid_for(storage.nb_products()) && !storage.is_removed(*product)
            })
            .unwrap_or_default()
    }

    /// `OccurrenceLocation(theOccurrence)`: the local placement of the first active
    /// parent reference, identity otherwise (the occurrence's removal is not checked).
    pub fn occurrence_location(self, occurrence: OccurrenceId) -> Location {
        let storage = self.graph.inc_storage();
        if !occurrence.is_valid_for(storage.nb_occurrences()) {
            return Location::default();
        }
        let refs = &storage
            .occurrence_relations(occurrence)
            .parent_occurrence_ref_ids;
        refs.iter()
            .find(|ref_id| {
                ref_id.is_valid_for(storage.nb_occurrence_refs()) && !storage.is_removed(**ref_id)
            })
            .map(|ref_id| storage.occurrence_ref(*ref_id).local_location.clone())
            .unwrap_or_default()
    }
}
