// Rust translation of "Phase 2: Edge Merging" of `BRepGraph_Deduplicate::Perform`
// (ModelingData/TKBRep/BRepGraph/BRepGraph_Deduplicate.cxx) with its `EdgeKey` and
// `EdgeKeyHasher` at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edge merge: edges sharing a curve handle and (undirected) end vertices, with
//! matching parameter ranges and compatible tolerances, collapse onto the first edge.
use super::geometry::{RewriteMap, rewrite_map};
use super::redirect::{redirect_generic_parents, replace_and_record};
use super::{DeduplicateResult, Options};
use crate::brep_graph::id_walk::full_ids;
use crate::brep_graph::layer_history::LayerHistory;
use crate::brep_graph::tool;
use crate::brep_graph::{BRepGraph, EdgeId, EdgeTag, NodeId, VertexId, VertexRefId, WireId};
use rustycad_tkernel::ncollection::{DataMap, HashPolicy};
use rustycad_tkernel::standard::hashing::hash;
use rustycad_tkernel::standard::{Failure, Handle, hash_bytes};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::geom::GeomCurve;

// The tolerance ratio above which a candidate edge is not merged.
const EDGE_TOLERANCE_RATIO: f64 = 10.0;

/// `EdgeKey`: the curve handle and the end vertices, the smaller index first.
#[derive(Clone)]
struct EdgeKey {
    curve: Handle<dyn GeomCurve>,
    start: VertexId,
    end: VertexId,
}

impl std::fmt::Debug for EdgeKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EdgeKey")
            .field("curve", &self.curve.object_address())
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

/// `EdgeKeyHasher`: the handle address (`std::hash` of a handle), and the Murmur hash of
/// each vertex index, combined with `hashBytes`. The address makes the group order a
/// property of the process, natively too; the merge order only depends on it through
/// the collisions of the edge ids in the canonical map.
#[derive(Clone, Copy, Debug, Default)]
struct EdgeKeyHasher;

impl HashPolicy<EdgeKey> for EdgeKeyHasher {
    fn hash(&self, key: &EdgeKey) -> usize {
        let words = [
            handle_address(&key.curve),
            hash::<usize, u32>(&key.start.index),
            hash::<usize, u32>(&key.end.index),
        ];
        hash_words(&words)
    }

    fn equal(&self, first: &EdgeKey, second: &EdgeKey) -> bool {
        first.curve == second.curve && first.start == second.start && first.end == second.end
    }
}

/// `opencascade::hashBytes` of a `size_t` array: Murmur over its native bytes.
pub(super) fn hash_words(words: &[usize]) -> usize {
    let bytes: Vec<u8> = words.iter().flat_map(|word| word.to_ne_bytes()).collect();
    hash_bytes(&bytes)
}

/// `std::hash<occ::handle<T>>`: the address of the referenced object.
pub(super) fn handle_address<T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(
    handle: &Handle<T>,
) -> usize {
    handle.object_address() as usize
}

/// Phase 2: groups the active edges with a curve by [`EdgeKey`], finds the canonical
/// edge of each duplicate, then (unless analyzing) replaces it in its wires, redirects
/// the compound and occurrence children, replaces it and records a `Dedup:MergeEdge`.
pub(super) fn merge(
    graph: &mut BRepGraph,
    history: &Handle<LayerHistory>,
    options: &Options,
    result: &mut DeduplicateResult,
) -> Result<(), Failure> {
    let canonical = canonical_edges(graph, options);
    if options.analyze_only {
        result.nb_merged_edges = canonical.len() as u32;
        return Ok(());
    }
    let label = AsciiString::new("Dedup:MergeEdge");
    for (&old, &canon) in canonical.items() {
        let reversed = is_reversed(graph, canon, old);
        // ReplaceEdge rewrites the edge's wire list, so the wires are copied first.
        let wires: Vec<WireId> = graph.topo().edges().wires_of(old).collect();
        for wire in wires {
            graph
                .editor()
                .wires()
                .replace_edge(wire, old, canon, reversed)?;
        }
        redirect_generic_parents(graph, old.into(), canon.into())?;
        replace_and_record(graph, history, &label, old.into(), canon.into())?;
        result.nb_history_records += 1;
        result.nb_merged_edges += 1;
    }
    Ok(())
}

// The start and end references of an edge (`BRepGraph_Tool::Edge::Start/EndVertexId`)
// mapped through `child` when valid, `absent` otherwise.
fn edge_ends<V: Copy>(
    graph: &BRepGraph,
    edge: EdgeId,
    child: impl Fn(VertexId) -> V,
    absent: V,
) -> (V, V) {
    let resolve = |reference: VertexRefId| {
        if reference.is_valid() {
            child(graph.refs().vertices().entry(reference).child_vertex_id)
        } else {
            absent
        }
    };
    (
        resolve(tool::edge::start_vertex_id(graph, edge)),
        resolve(tool::edge::end_vertex_id(graph, edge)),
    )
}

// The child vertices of the edge's references, the invalid id without one.
fn end_vertices(graph: &BRepGraph, edge: EdgeId) -> (VertexId, VertexId) {
    edge_ends(graph, edge, |vertex| vertex, VertexId::invalid())
}

// The groups of equal keys, in full-iterator order within a group.
fn edge_groups(graph: &BRepGraph) -> DataMap<EdgeKey, Vec<EdgeId>, EdgeKeyHasher> {
    let buckets = graph.topo().edges().nb().max(1) as usize;
    let mut groups = DataMap::with_hasher(EdgeKeyHasher, buckets);
    for edge in full_ids::<EdgeTag>(graph) {
        if edge.is_removed(graph) || !tool::edge::has_curve(graph, edge) {
            continue;
        }
        let (mut start, mut end) = end_vertices(graph, edge);
        // Normalize: the smaller vertex index first, for undirected matching.
        if start > end {
            std::mem::swap(&mut start, &mut end);
        }
        let key = EdgeKey {
            curve: tool::edge::curve(graph, edge),
            start,
            end,
        };
        groups.try_bind(key.clone(), Vec::new());
        if let Some(group) = groups.change_seek(&key) {
            group.push(edge);
        }
    }
    groups
}

// Candidate -> first edge of its group, for candidates whose range matches within
// `CompTolerance` and whose tolerance is at most ten times the canonical one.
fn canonical_edges(graph: &BRepGraph, options: &Options) -> RewriteMap<EdgeId> {
    let groups = edge_groups(graph);
    let mut canonical = rewrite_map(graph.topo().edges().nb());
    for group in groups.values().filter(|group| group.len() >= 2) {
        let canon = group[0];
        let canon_tolerance = graph.topo().edges().definition(canon).tolerance;
        for &candidate in &group[1..] {
            let (canon_first, canon_last) = tool::edge::range(graph, canon);
            let (first, last) = tool::edge::range(graph, candidate);
            if (canon_first - first).abs() > options.comp_tolerance
                || (canon_last - last).abs() > options.comp_tolerance
            {
                continue;
            }
            let tolerance = graph.topo().edges().definition(candidate).tolerance;
            if tolerance > canon_tolerance * EDGE_TOLERANCE_RATIO {
                continue;
            }
            canonical.bind(candidate, canon);
        }
    }
    canonical
}

// A duplicate runs against its canonical edge when its ends are swapped; self-loops
// never count as reversed (after the vertex merge both ends may be one vertex). An end
// without reference is the default node id, a referenced one the node of its vertex.
fn is_reversed(graph: &BRepGraph, canon: EdgeId, old: EdgeId) -> bool {
    let node = |vertex: VertexId| NodeId::from(vertex);
    let (canon_start, canon_end) = edge_ends(graph, canon, node, NodeId::default());
    let (old_start, old_end) = edge_ends(graph, old, node, NodeId::default());
    if canon_start == canon_end || old_start == old_end {
        return false;
    }
    canon_start == old_end && canon_end == old_start
}
