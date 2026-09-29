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
// Rust translation of `ensureOccurrenceRef`, `ensureOccurrence`, `ensureProduct` and the
// root-product rebuild of `Perform`/`CopyNode` of
// ModelingData/TKBRep/BRepGraph/BRepGraph_Copy.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The memoised copies of the assembly nodes, written to the result storage directly
//! (no UID is allocated for them, as natively), and the root-product list.
use super::containers::ensure_node;
use super::context::CopyContext;
use super::topology::ref_item;
use crate::brep::errors::program_error;
use crate::brep_graph::id_walk::full_ids;
use crate::brep_graph::{
    BRepGraph, NodeKind, OccurrenceId, OccurrenceRefId, OccurrenceTag, ProductId, ProductTag,
};
use rustycad_tkernel::standard::Failure;
use std::collections::HashSet;

/// `ensureOccurrenceRef(theCtx, theSrcRefId, theDstParentProductId)`: the copy of an
/// occurrence reference under a copied product, with its child occurrence.
pub(crate) fn ensure_occurrence_ref(
    context: &mut CopyContext<'_>,
    source: OccurrenceRefId,
    parent: ProductId,
) -> Result<OccurrenceRefId, Failure> {
    if let Some(existing) = context.existing_occurrence_ref(source) {
        return Ok(existing);
    }
    let entry = context.src_storage().occurrence_ref(source);
    let (child, location) = (entry.child_occurrence_id, entry.local_location.clone());
    let mut new_occurrence = OccurrenceId::invalid();
    if child.is_valid() {
        new_occurrence = ensure_occurrence(context, child)?;
    }
    let copied =
        context
            .dst_storage()
            .attach_occurrence_to_product(parent, new_occurrence, &location);
    context.bind_occurrence_ref(source, copied);
    context.bind_item(ref_item(source), ref_item(copied));
    context.copy_removed(source, copied);
    Ok(copied)
}

/// `ensureOccurrence(theCtx, theSrcId)`: the copy of an occurrence, bound before its
/// child node is copied; the copy then points at the mapped child.
pub(crate) fn ensure_occurrence(
    context: &mut CopyContext<'_>,
    source: OccurrenceId,
) -> Result<OccurrenceId, Failure> {
    if let Some(existing) = context.existing(source) {
        return Ok(existing);
    }
    let child = context.src_storage().occurrence(source).child_node_id;
    let copied = context.dst_storage().append_occurrence();
    context.bind_node(source, copied);
    context.copy_removed(source, copied);
    if child.is_valid() {
        ensure_node(context, child)?;
        let mapped = context.mapped_node(child);
        context
            .dst_storage()
            .change_occurrence(copied)
            .child_node_id = mapped;
    }
    Ok(copied)
}

/// `ensureProduct(theCtx, theSrcId)`: the copy of a product, bound before its
/// occurrence references (assembly cycles short-circuit); a refused reference raises
/// Standard_ProgramError.
pub(crate) fn ensure_product(
    context: &mut CopyContext<'_>,
    source: ProductId,
) -> Result<ProductId, Failure> {
    if let Some(existing) = context.existing(source) {
        return Ok(existing);
    }
    let copied = context.dst_storage().append_product();
    context.copy_removed(source, copied);
    context.bind_node(source, copied);
    let refs = context
        .src_storage()
        .product_relations(source)
        .occurrence_ref_ids
        .clone();
    for reference in refs {
        let new_ref = ensure_occurrence_ref(context, reference, copied)?;
        if !new_ref.is_valid() {
            return Err(program_error(
                "BRepGraph_Copy: failed to copy occurrence ref",
                &format!("{reference:?} under {copied:?} (expected a valid OccurrenceRefId)"),
            ));
        }
    }
    Ok(copied)
}

/// The root-product rebuild of `Perform` and `CopyNode`, when products were copied:
/// every active product of the result that no active occurrence of the result
/// references is appended to the root list.
pub(crate) fn append_root_products(context: &mut CopyContext<'_>) {
    if !context.has_products() {
        return;
    }
    let target = context.target();
    let referenced = referenced_products(target);
    let roots: Vec<ProductId> = full_ids::<ProductTag>(target)
        .into_iter()
        .filter(|product| {
            !target.inc_storage().is_removed(*product) && !referenced.contains(product)
        })
        .collect();
    context
        .dst_storage()
        .change_root_product_ids()
        .extend(roots);
}

// The products an active occurrence of the graph points at.
fn referenced_products(graph: &BRepGraph) -> HashSet<ProductId> {
    let storage = graph.inc_storage();
    let mut referenced = HashSet::new();
    for occurrence in full_ids::<OccurrenceTag>(graph) {
        let child = storage.occurrence(occurrence).child_node_id;
        if storage.is_removed(occurrence) || child.kind != NodeKind::PRODUCT {
            continue;
        }
        let product = ProductId::from_node_id(child);
        if product.is_valid_in(&graph.topo().products()) {
            referenced.insert(product);
        }
    }
    referenced
}
