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
// Rust translation of BRepGraph_ReconstructionContext, reconstructOccurrenceLocal,
// reconstructProductLocal and reconstructShape of
// ModelingData/TKBRep/BRepGraph/BRepGraph_ShapesView.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reconstruction of any node, the assembly nodes included: a product is its
//! topology root (at the root reference's location) or a compound of its occurrences;
//! an occurrence is its child at its global location.
use crate::brep_graph::child_refs::active_occurrence_refs;
use crate::brep_graph::{BRepGraph, NodeId, NodeKind, OccurrenceId, ProductId};
use crate::brep_graph_inc::reconstruct::{self, Cache};
use crate::topods::{Builder, Compound, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::toploc::Location;
use std::collections::HashSet;

/// `BRepGraph_ReconstructionContext`: the graph, a shared reconstruction cache and the
/// products being reconstructed (the cycle guard).
pub(super) struct ReconstructionContext<'g> {
    graph: &'g BRepGraph,
    cache: Cache,
    active_products: HashSet<ProductId>,
}

impl<'g> ReconstructionContext<'g> {
    /// `makeReconstructionContext(theGraph, theStorage)`.
    pub(super) fn new(graph: &'g BRepGraph) -> Self {
        Self {
            graph,
            cache: Cache::new(),
            active_products: HashSet::new(),
        }
    }

    /// `reconstructShape(theContext, theNode)`: null for an invalid node.
    pub(super) fn shape(&mut self, node: NodeId) -> Result<Shape, Failure> {
        if !node.is_valid() {
            return Ok(Shape::default());
        }
        match node.kind {
            NodeKind::PRODUCT => self.product(ProductId::from_node_id(node)),
            NodeKind::OCCURRENCE => self.occurrence_global(OccurrenceId::from_node_id(node)),
            _ => reconstruct::node_with_cache(self.graph, node, &mut self.cache),
        }
    }

    // The child of an active occurrence (a product reconstructed locally, else the
    // topology node); null for an inactive occurrence or one without child.
    fn occurrence_child(&mut self, occurrence: OccurrenceId) -> Result<Shape, Failure> {
        let storage = self.graph.inc_storage();
        if !occurrence.is_valid_for(storage.nb_occurrences()) {
            return Ok(Shape::default());
        }
        let child = storage.occurrence(occurrence).child_node_id;
        if storage.is_removed(occurrence) || !child.is_valid() {
            return Ok(Shape::default());
        }
        if child.kind == NodeKind::PRODUCT {
            return self.product(ProductId::from_node_id(child));
        }
        reconstruct::node_with_cache(self.graph, child, &mut self.cache)
    }

    // The Occurrence case of `reconstructShape`: moved by the occurrence's global
    // location.
    fn occurrence_global(&mut self, occurrence: OccurrenceId) -> Result<Shape, Failure> {
        let mut shape = self.occurrence_child(occurrence)?;
        if !shape.is_null() {
            let global = self
                .graph
                .topo()
                .occurrences()
                .occurrence_location(occurrence);
            move_unless_identity(&mut shape, &global)?;
        }
        Ok(shape)
    }

    // `reconstructOccurrenceLocal`: moved by the occurrence reference's local location.
    fn occurrence_local(
        &mut self,
        occurrence: OccurrenceId,
        local: &Location,
    ) -> Result<Shape, Failure> {
        let mut shape = self.occurrence_child(occurrence)?;
        if !shape.is_null() {
            move_unless_identity(&mut shape, local)?;
        }
        Ok(shape)
    }

    // `reconstructProductLocal`: the cached product, else (unless removed or already
    // being reconstructed) its reconstruction, cached when not null.
    fn product(&mut self, product: ProductId) -> Result<Shape, Failure> {
        let storage = self.graph.inc_storage();
        if !product.is_valid_for(storage.nb_products()) {
            return Ok(Shape::default());
        }
        if let Some(cached) = self.cache.seek(product.into()) {
            return Ok(cached.clone());
        }
        if storage.is_removed(product) || self.active_products.contains(&product) {
            return Ok(Shape::default());
        }
        self.active_products.insert(product);
        let result = self.product_shape(product);
        self.active_products.remove(&product);
        let result = result?;
        if !result.is_null() {
            self.cache.bind(product.into(), &result);
        }
        Ok(result)
    }

    // The shape root of the first occurrence with a topology child, at its reference
    // location; else a compound of the occurrences at their local locations.
    fn product_shape(&mut self, product: ProductId) -> Result<Shape, Failure> {
        let storage = self.graph.inc_storage();
        let references = active_occurrence_refs(self.graph, product);
        let root = references.iter().find_map(|&ref_id| {
            let reference = storage.occurrence_ref(ref_id);
            let child = storage
                .occurrence(reference.child_occurrence_id)
                .child_node_id;
            (child.is_valid() && child.kind != NodeKind::PRODUCT)
                .then(|| (child, reference.local_location.clone()))
        });
        if let Some((root, location)) = root {
            let mut shape = reconstruct::node_with_cache(self.graph, root, &mut self.cache)?;
            if !shape.is_null() {
                move_unless_identity(&mut shape, &location)?;
            }
            return Ok(shape);
        }
        self.occurrences_compound(&references)
    }

    // The compound of the active occurrences' local shapes.
    fn occurrences_compound(
        &mut self,
        references: &[crate::brep_graph::OccurrenceRefId],
    ) -> Result<Shape, Failure> {
        let storage = self.graph.inc_storage();
        let mut compound = Compound::default();
        Builder.make_compound(&mut compound);
        for &ref_id in references {
            let reference = storage.occurrence_ref(ref_id);
            let occurrence = reference.child_occurrence_id;
            if !occurrence.is_valid_for(storage.nb_occurrences()) || storage.is_removed(occurrence)
            {
                continue;
            }
            let child = self.occurrence_local(occurrence, &reference.local_location.clone())?;
            if !child.is_null() {
                Builder.add(&mut compound, &child)?;
            }
        }
        Ok(compound.into())
    }
}

// `Move(theLocation)` unless the location is the identity.
fn move_unless_identity(shape: &mut Shape, location: &Location) -> Result<(), Failure> {
    if location.is_identity() {
        return Ok(());
    }
    shape.move_by(location, false)
}
