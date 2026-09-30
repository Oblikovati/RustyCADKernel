// Rust translation of the assembly rebuild of `BRepGraph_Compact::Perform`
// (ModelingData/TKBRep/BRepGraph/BRepGraph_Compact.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The products and occurrences, which are not topology definitions: each kept product
//! is rebuilt as a part (over its remapped shape root, with the old part placement) or
//! an empty product, then the product-to-product links are restored.
use super::maps::{NodeMaps, RefMaps};
use crate::brep_graph::id_walk::full_ids;
use crate::brep_graph::*;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::toploc::Location;
use std::collections::HashSet;

// The old product is skipped: unreachable from the root products (when there are
// some) or removed.
fn is_skipped(graph: &BRepGraph, reachable: Option<&HashSet<NodeId>>, product: ProductId) -> bool {
    reachable.is_some_and(|nodes| !nodes.contains(&product.into()))
        || graph.inc_storage().is_removed(product)
}

// The first active occurrence reference of a product whose active occurrence has a
// topology child. The occurrence definition is read before its removed check, as
// natively (an invalid child occurrence id is out of range there and skipped here).
fn part_occurrence_ref(graph: &BRepGraph, product: ProductId) -> Option<OccurrenceRefId> {
    let storage = graph.inc_storage();
    graph
        .topo()
        .products()
        .relations(product)
        .occurrence_ref_ids
        .iter()
        .copied()
        .find(|&reference| {
            let occurrence = graph
                .refs()
                .occurrences()
                .entry(reference)
                .child_occurrence_id;
            if storage.is_removed(reference)
                || !occurrence.is_valid_in(&graph.topo().occurrences())
                || storage.is_removed(occurrence)
            {
                return false;
            }
            let child = graph
                .topo()
                .occurrences()
                .definition(occurrence)
                .child_node_id;
            child.is_valid() && NodeKind::is_topology_kind(child.kind)
        })
}

// The rebuilt product: a part over the remapped shape root placed like the old part
// occurrence, else an empty product; both become document roots.
fn rebuild_product(
    graph: &BRepGraph,
    fresh: &mut BRepGraph,
    maps: &NodeMaps,
    product: ProductId,
) -> Result<ProductId, Failure> {
    let root = graph.topo().products().shape_root_node(product);
    let mut new = ProductId::invalid();
    if root.is_valid() && NodeKind::is_topology_kind(root.kind) {
        let new_root = maps.remap(root);
        if new_root.is_valid() {
            let placement =
                part_occurrence_ref(graph, product).map_or_else(Location::default, |reference| {
                    graph
                        .refs()
                        .occurrences()
                        .entry(reference)
                        .local_location
                        .clone()
                });
            new = fresh.editor().products().add(new_root, &placement);
            fresh.editor().products().append_document_root(new)?;
        }
    }
    if !new.is_valid() {
        new = fresh.editor().products().add_empty();
        fresh.editor().products().append_document_root(new)?;
    }
    Ok(new)
}

/// The kept products in full-iterator order, each mapped with its part occurrence and
/// part occurrence reference (the first new occurrence with a topology child).
pub(super) fn rebuild_products(
    graph: &BRepGraph,
    fresh: &mut BRepGraph,
    maps: &mut NodeMaps,
    refs: &mut RefMaps,
    reachable: Option<&HashSet<NodeId>>,
) -> Result<(), Failure> {
    for old in full_ids::<ProductTag>(graph) {
        if is_skipped(graph, reachable, old) {
            continue;
        }
        let new = rebuild_product(graph, fresh, maps, old)?;
        if !new.is_valid() {
            continue;
        }
        maps.products.bind(old, new);
        let Some(old_ref) = part_occurrence_ref(graph, old) else {
            continue;
        };
        let old_occurrence = graph
            .refs()
            .occurrences()
            .entry(old_ref)
            .child_occurrence_id;
        if let Some(new_ref) = part_occurrence_ref(fresh, new) {
            let new_occurrence = fresh
                .refs()
                .occurrences()
                .entry(new_ref)
                .child_occurrence_id;
            maps.occurrences.bind(old_occurrence, new_occurrence);
            refs.occurrence_refs.bind(old_ref, new_ref);
        }
    }
    Ok(())
}

/// The product-to-product links of the kept parents, with their local placements; the
/// child occurrence is mapped once, the occurrence reference per parent slot.
pub(super) fn restore_product_links(
    graph: &BRepGraph,
    fresh: &mut BRepGraph,
    maps: &mut NodeMaps,
    refs: &mut RefMaps,
    reachable: Option<&HashSet<NodeId>>,
) -> Result<(), Failure> {
    for old_parent in full_ids::<ProductTag>(graph) {
        if is_skipped(graph, reachable, old_parent) {
            continue;
        }
        let Some(&new_parent) = maps.products.seek(&old_parent) else {
            continue;
        };
        let old_refs = graph
            .topo()
            .products()
            .relations(old_parent)
            .occurrence_ref_ids
            .clone();
        for old_ref in old_refs {
            restore_link(graph, fresh, maps, refs, new_parent, old_ref)?;
        }
    }
    Ok(())
}

fn restore_link(
    graph: &BRepGraph,
    fresh: &mut BRepGraph,
    maps: &mut NodeMaps,
    refs: &mut RefMaps,
    new_parent: ProductId,
    old_ref: OccurrenceRefId,
) -> Result<(), Failure> {
    let Some((old_occurrence, child, location)) = product_link(graph, old_ref) else {
        return Ok(());
    };
    let Some(&new_child) = maps.products.seek(&child) else {
        return Ok(());
    };
    let invalid = OccurrenceId::invalid();
    let mut products = fresh.editor().products();
    let (new_occurrence, _) = products.append(new_parent, new_child, &location, invalid)?;
    if !new_occurrence.is_valid() {
        return Ok(());
    }
    if !maps.occurrences.is_bound(&old_occurrence) {
        maps.occurrences.bind(old_occurrence, new_occurrence);
    }
    // Occurrence refs are usage records: the identity moves with the exact
    // parent-owned slot, not only with the occurrence definition.
    if let Some(new_ref) = slot_of(fresh, new_parent, new_occurrence) {
        refs.occurrence_refs.bind(old_ref, new_ref);
    }
    Ok(())
}

// The occurrence, child product and placement of an active occurrence reference whose
// active occurrence places a product.
fn product_link(
    graph: &BRepGraph,
    reference: OccurrenceRefId,
) -> Option<(OccurrenceId, ProductId, Location)> {
    let storage = graph.inc_storage();
    let entry = graph.refs().occurrences().entry(reference);
    let occurrence = entry.child_occurrence_id;
    if storage.is_removed(reference) || !occurrence.is_valid_in(&graph.topo().occurrences()) {
        return None;
    }
    let child = graph
        .topo()
        .occurrences()
        .definition(occurrence)
        .child_node_id;
    if storage.is_removed(occurrence) || !child.is_valid() || child.kind != NodeKind::PRODUCT {
        return None;
    }
    let location = entry.local_location.clone();
    Some((occurrence, ProductId::from_node_id(child), location))
}

// The parent's first occurrence reference to the occurrence.
fn slot_of(
    graph: &BRepGraph,
    parent: ProductId,
    occurrence: OccurrenceId,
) -> Option<OccurrenceRefId> {
    let refs = &graph.topo().products().relations(parent).occurrence_ref_ids;
    refs.iter().copied().find(|&reference| {
        graph
            .refs()
            .occurrences()
            .entry(reference)
            .child_occurrence_id
            == occurrence
    })
}
