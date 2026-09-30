// Rust translation of the filter of BRepGraph_RefsIterator::RefsOfParent with
// OccurrenceOfProductTraits (ModelingData/TKBRep/BRepGraph/BRepGraph_RefsIterator.hxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The active occurrence references of a product, in relation order: what
//! `BRepGraph_RefsOccurrenceOfProduct` yields, collected for the phase-1 views and
//! `Shapes()` (the filter itself is [`RefsOccurrenceOfProduct`]).
use crate::brep_graph::refs_iterator::RefsOccurrenceOfProduct;
use crate::brep_graph::{BRepGraph, OccurrenceRefId, ProductId};

/// The references of an active product that are in range and active and whose
/// occurrence is in range and active; empty for an invalid or removed product.
pub(crate) fn active_occurrence_refs(
    graph: &BRepGraph,
    product: ProductId,
) -> Vec<OccurrenceRefId> {
    RefsOccurrenceOfProduct::new(graph, product).collect()
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
