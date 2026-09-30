// Rust translation of checkActiveCounts and checkDocumentRootProducts of
// ModelingData/TKBRep/BRepGraph/BRepGraph_Validate.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The active-count check (audit) and the document-root check (both modes).
//!
//! The per-kind node counts are the editor boundary's `cachedActiveByKind` and
//! `countActiveByKind`, which read the same stored counts and visit the same slots as
//! the native `NbActive()` and `countActive` of the validator.
use super::{Issue, Severity, report};
use crate::brep_graph::editor::{BOUNDARY_KINDS, actual_active, cached_active, kind_name};
use crate::brep_graph::*;
use std::collections::HashSet;

/// `checkActiveCounts`: every cached active count equals the count of live slots.
pub(super) fn check_active_counts(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    for kind in BOUNDARY_KINDS {
        let cached = cached_active(graph.inc_storage(), kind);
        verify(kind_name(kind), cached, actual_active(graph, kind), issues);
    }
    check_active_use_counts(graph, issues);
}

// The representation stores, in the native order.
fn check_active_use_counts(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let (geometry, poly) = (graph.topo().geometry(), graph.mesh().poly());
    let checks: [(&str, u32, i32); 7] = [
        (
            "FaceSurfaces",
            geometry.nb_active_face_surfaces(),
            live_uses::<FaceSurfaceTag>(graph),
        ),
        (
            "FaceTriangulations",
            poly.nb_active_triangulations(),
            live_uses::<FaceTriangulationTag>(graph),
        ),
        (
            "EdgeCurves3D",
            geometry.nb_active_edge_curves3d(),
            live_uses::<EdgeCurve3DTag>(graph),
        ),
        (
            "EdgePolygons3D",
            poly.nb_active_polygons3d(),
            live_uses::<EdgePolygon3DTag>(graph),
        ),
        (
            "CoEdgeCurves2D",
            geometry.nb_active_co_edge_curves2d(),
            live_uses::<CoEdgeCurve2DTag>(graph),
        ),
        (
            "CoEdgePolygons2D",
            poly.nb_active_polygons2d(),
            live_uses::<CoEdgePolygon2DTag>(graph),
        ),
        (
            "CoEdgePolygonsOnTri",
            poly.nb_active_polygons_on_tri(),
            live_uses::<CoEdgePolygonOnTriTag>(graph),
        ),
    ];
    for (name, cached, actual) in checks {
        verify(name, cached as i32, actual, issues);
    }
}

// `countActiveUses`: the slots of a representation store that are not removed.
fn live_uses<T: RepTag>(graph: &BRepGraph) -> i32 {
    (0..graph.inc_storage().nb::<T>())
        .filter(|index| !TypedId::<T>::new(*index).is_removed(graph))
        .count() as i32
}

// `verify`: a count mismatch is an error on no node.
fn verify(name: &str, cached: i32, actual: i32, issues: &mut Vec<Issue>) {
    if cached != actual {
        let text = format!("NbActive{name} mismatch: cached={cached} actual={actual}");
        report(issues, Severity::Error, NodeId::default(), &text);
    }
}

/// `checkDocumentRootProducts`: each root product is in range, live, listed once and
/// placed by no live occurrence.
pub(super) fn check_document_root_products(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut seen: HashSet<ProductId> = HashSet::new();
    for &root in graph.root_product_ids() {
        if !root.is_valid_in(&graph.topo().products()) {
            let text = "Document root Product is out of bounds";
            report(issues, Severity::Error, root, text);
            continue;
        }
        if root.is_removed(graph) {
            report(
                issues,
                Severity::Error,
                root,
                "Document root Product is removed",
            );
            continue;
        }
        if !seen.insert(root) {
            let text = "Document root Product is listed more than once";
            report(issues, Severity::Error, root, text);
        }
        if is_placed_by_live_occurrence(graph, root) {
            let text = "Document root Product is referenced by an active Occurrence";
            report(issues, Severity::Error, root, text);
        }
    }
}

// A live occurrence reference whose live occurrence places the product.
fn is_placed_by_live_occurrence(graph: &BRepGraph, product: ProductId) -> bool {
    let references = graph.refs().occurrences().ids_referencing(product.into());
    let nb_references = graph.refs().occurrences().nb();
    references.iter().any(|&reference| {
        if !reference.is_valid_for(nb_references) || reference.is_removed(graph) {
            return false;
        }
        let occurrence = graph
            .refs()
            .occurrences()
            .entry(reference)
            .child_occurrence_id;
        occurrence.is_valid_in(&graph.topo().occurrences()) && !occurrence.is_removed(graph)
    })
}
