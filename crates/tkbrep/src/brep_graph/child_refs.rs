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
// Rust translation of the filter of BRepGraph_RefsIterator::RefsOfParent with
// OccurrenceOfProductTraits (ModelingData/TKBRep/BRepGraph/BRepGraph_RefsIterator.hxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The active occurrence references of a product, in relation order: what
//! `BRepGraph_RefsOccurrenceOfProduct` yields. The phase-1 views and `Shapes()` need
//! only this list; the iterators area ports the iterator class itself and may route
//! these callers through it.
use crate::brep_graph::{BRepGraph, OccurrenceRefId, ProductId};

/// The references of an active product that are in range and active and whose
/// occurrence is in range and active; empty for an invalid or removed product.
pub(crate) fn active_occurrence_refs(
    graph: &BRepGraph,
    product: ProductId,
) -> Vec<OccurrenceRefId> {
    let storage = graph.inc_storage();
    if !product.is_valid_for(storage.nb_products()) || product.is_removed(graph) {
        return Vec::new();
    }
    let (nb_refs, nb_occurrences) = (storage.nb_occurrence_refs(), storage.nb_occurrences());
    storage
        .product_relations(product)
        .occurrence_ref_ids
        .iter()
        .copied()
        .filter(|ref_id| ref_id.is_valid_for(nb_refs) && !ref_id.is_removed(graph))
        .filter(|ref_id| {
            let occurrence = storage.occurrence_ref(*ref_id).child_occurrence_id;
            occurrence.is_valid_for(nb_occurrences) && !occurrence.is_removed(graph)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkmath::toploc::Location;

    #[test]
    fn removed_occurrences_are_skipped() {
        let mut graph = BRepGraph::new();
        let storage = graph.inc_storage_mut();
        let (product, first, second) = (
            storage.append_product(),
            storage.append_occurrence(),
            storage.append_occurrence(),
        );
        let kept = storage.attach_occurrence_to_product(product, first, &Location::default());
        storage.attach_occurrence_to_product(product, second, &Location::default());
        storage.set_removed(second, true);
        assert_eq!(active_occurrence_refs(&graph, product), vec![kept]);
        assert!(active_occurrence_refs(&graph, ProductId::new(3)).is_empty());
    }
}
