// Rust translation of "Phase 1: Vertex Merging via KDTree range search" of
// `BRepGraph_Deduplicate::Perform` (ModelingData/TKBRep/BRepGraph/BRepGraph_Deduplicate.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The vertex merge: coincident vertices (within the larger tolerance) collapse onto
//! the vertex with the larger tolerance, searched through a KD-tree.
use super::geometry::{RewriteMap, rewrite_map};
use super::redirect::{redirect_generic_parents, replace_and_record};
use super::{DeduplicateResult, Options};
use crate::brep_graph::editor::VertexOps;
use crate::brep_graph::id_walk::full_ids;
use crate::brep_graph::layer_history::LayerHistory;
use crate::brep_graph::tool;
use crate::brep_graph::{BRepGraph, EdgeTag, VertexId, VertexRefId, VertexTag};
use rustycad_tkernel::ncollection::KDTree;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::stl::sort_by;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::Point3;

// `std::pair<gp_Pnt, BRepGraph_VertexId>` of the active vertex list.
type ActiveVertex = (Point3, VertexId);

// Hops of the canonical-chain resolution before giving up (the native cycle guard).
const MAX_CANONICAL_HOPS: usize = 64;

/// Phase 1: finds the canonical vertex of every mergeable vertex, then (unless
/// analyzing) redirects the edges' vertex references, the compound and occurrence
/// children, replaces each merged vertex and records a `Dedup:MergeVertex`.
pub(super) fn merge(
    graph: &mut BRepGraph,
    history: &Handle<LayerHistory>,
    options: &Options,
    result: &mut DeduplicateResult,
) -> Result<(), Failure> {
    let canonical = canonical_vertices(graph, options.hash_tolerance);
    if options.analyze_only {
        result.nb_merged_vertices = canonical.len() as u32;
        return Ok(());
    }
    let label = AsciiString::new("Dedup:MergeVertex");
    for (&old, &canon) in canonical.items() {
        redirect_edge_vertices(graph, old, canon)?;
        redirect_generic_parents(graph, old.into(), canon.into())?;
        replace_and_record(graph, history, &label, old.into(), canon.into())?;
        result.nb_history_records += 1;
        result.nb_merged_vertices += 1;
    }
    Ok(())
}

// The active vertices sorted by descending tolerance (libstdc++ `std::sort`, whose tie
// order the result follows): a high-tolerance base searches with a radius at least its
// own tolerance, which dominates the per-pair acceptance distance, so every mergeable
// pair is examined once with a sufficient radius.
fn sorted_active_vertices(graph: &BRepGraph) -> Vec<ActiveVertex> {
    let mut vertices: Vec<ActiveVertex> = full_ids::<VertexTag>(graph)
        .into_iter()
        .filter(|vertex| !vertex.is_removed(graph))
        .map(|vertex| (tool::vertex::pnt(graph, vertex), vertex))
        .collect();
    sort_by(&mut vertices, |first, second| {
        tool::vertex::tolerance(graph, first.1) > tool::vertex::tolerance(graph, second.1)
    });
    vertices
}

// Old vertex -> the base vertex it merges into: each unbound base searches the tree
// within `max(tolerance, base tolerance)`; a later candidate, resolved through earlier
// bindings, binds to the base when within the larger of the two tolerances.
fn canonical_vertices(graph: &BRepGraph, tolerance: f64) -> RewriteMap<VertexId> {
    let vertices = sorted_active_vertices(graph);
    let mut tree = KDTree::<[f64; 3], 3>::default();
    if !vertices.is_empty() {
        let points: Vec<[f64; 3]> = vertices
            .iter()
            .map(|(p, _)| [p.x(), p.y(), p.z()])
            .collect();
        tree.build(&points);
    }
    let mut canonical = rewrite_map(graph.topo().vertices().nb());
    for (local, &(_, base)) in vertices.iter().enumerate() {
        if canonical.is_bound(&base) {
            continue;
        }
        let base_point = tool::vertex::pnt(graph, base);
        let base_tolerance = tool::vertex::tolerance(graph, base);
        let query = [base_point.x(), base_point.y(), base_point.z()];
        tree.for_each_in_range(&query, tolerance.max(base_tolerance), |found| {
            let position = found - 1;
            if position <= local {
                return; // skip self and already-processed
            }
            let candidate = vertices[position].1;
            let effective = resolve_canonical(&canonical, candidate);
            if effective == base {
                return;
            }
            let limit = base_tolerance.max(tool::vertex::tolerance(graph, effective));
            if base_point.distance(&tool::vertex::pnt(graph, effective)) <= limit {
                canonical.bind(candidate, base);
            }
        });
    }
    canonical
}

// The end of a candidate's canonical chain (path compression with a cycle guard).
fn resolve_canonical(canonical: &RewriteMap<VertexId>, candidate: VertexId) -> VertexId {
    let mut effective = candidate;
    for _ in 0..MAX_CANONICAL_HOPS {
        let Some(&next) = canonical.seek(&effective) else {
            break;
        };
        if next == effective {
            break;
        }
        effective = next;
    }
    effective
}

// Every edge's vertex references that point at `old` point at `canon`. The edge guard
// is taken before the removed check, so a removed edge raises Standard_ProgramError
// ("invalid node id"), as natively.
fn redirect_edge_vertices(
    graph: &mut BRepGraph,
    old: VertexId,
    canon: VertexId,
) -> Result<(), Failure> {
    for edge in full_ids::<EdgeTag>(graph) {
        let mut guard = graph.editor().edges().mutate(edge)?;
        if edge.is_removed(guard.graph()) {
            continue;
        }
        let (start, end) = (guard.start_vertex_ref_id, guard.end_vertex_ref_id);
        redirect_vertex_ref(guard.graph_mut(), start, old, canon)?;
        redirect_vertex_ref(guard.graph_mut(), end, old, canon)?;
    }
    Ok(())
}

// One vertex reference of an edge, rewritten through its own guard when it points at
// `old`.
fn redirect_vertex_ref(
    graph: &mut BRepGraph,
    reference: VertexRefId,
    old: VertexId,
    canon: VertexId,
) -> Result<(), Failure> {
    if !reference.is_valid() {
        return Ok(());
    }
    let mut guard = graph.editor().vertices().mutate_ref(reference)?;
    if guard.child_vertex_id == old {
        VertexOps::set_ref_child_vertex_id_guarded(&mut guard, canon);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_canonical_cycle_stops_at_the_hop_limit() {
        let mut canonical = rewrite_map::<VertexId>(4);
        canonical.bind(VertexId::new(0), VertexId::new(1));
        canonical.bind(VertexId::new(1), VertexId::new(0));
        let end = resolve_canonical(&canonical, VertexId::new(0));
        assert_eq!(end, VertexId::new(0));
        canonical.bind(VertexId::new(2), VertexId::new(2));
        assert_eq!(
            resolve_canonical(&canonical, VertexId::new(2)),
            VertexId::new(2)
        );
    }
}
