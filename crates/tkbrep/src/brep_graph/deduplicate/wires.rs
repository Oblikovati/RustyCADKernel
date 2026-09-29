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
// Rust translation of "Phase 3: Wire Merging" of `BRepGraph_Deduplicate::Perform`
// (ModelingData/TKBRep/BRepGraph/BRepGraph_Deduplicate.cxx) with its `WireHash` and
// `wiresEqual` at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The wire merge: wires with the same ordered coedge sequence (edge and orientation)
//! collapse onto the first one of their hash bucket.
use super::edges::hash_words;
use super::geometry::{RewriteMap, rewrite_map};
use super::redirect::{redirect_generic_parents, replace_and_record};
use super::{DeduplicateResult, Options};
use crate::brep_graph::editor::WireOps;
use crate::brep_graph::id_walk::full_ids;
use crate::brep_graph::layer_history::LayerHistory;
use crate::brep_graph::refs_iterator::CoEdgesOfWire;
use crate::brep_graph::{BRepGraph, EdgeId, FaceTag, WireId, WireTag};
use rustycad_tkernel::ncollection::{DataMap, DefaultHashPolicy};
use rustycad_tkernel::standard::hashing::hash;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;

// The golden-ratio constant of the `boost::hash_combine` step of `WireHash`.
const HASH_COMBINE_CONSTANT: usize = 0x9e37_79b9;

/// Phase 3: buckets the active wires by [`wire_hash`], binds every later wire of a
/// bucket equal to an unbound earlier one, then (unless analyzing) redirects the face
/// wire references, the compound and occurrence children, replaces each merged wire
/// and records a `Dedup:MergeWire`.
pub(super) fn merge(
    graph: &mut BRepGraph,
    history: &Handle<LayerHistory>,
    options: &Options,
    result: &mut DeduplicateResult,
) -> Result<(), Failure> {
    let canonical = canonical_wires(graph);
    if options.analyze_only {
        result.nb_merged_wires = canonical.len() as u32;
        return Ok(());
    }
    let label = AsciiString::new("Dedup:MergeWire");
    for (&old, &canon) in canonical.items() {
        redirect_face_wire_refs(graph, old, canon)?;
        redirect_generic_parents(graph, old.into(), canon.into())?;
        replace_and_record(graph, history, &label, old.into(), canon.into())?;
        result.nb_history_records += 1;
        result.nb_merged_wires += 1;
    }
    Ok(())
}

// The (edge, orientation) sequence of a wire's active coedges.
fn co_edge_sequence(graph: &BRepGraph, wire: WireId) -> Vec<(EdgeId, bool)> {
    CoEdgesOfWire::new(graph, wire)
        .map(|co_edge| {
            let def = graph.topo().co_edges().definition(co_edge);
            (def.child_edge_id, def.orientation.is_reversed)
        })
        .collect()
}

/// `WireHash`: per coedge, `hashBytes` of the edge-index and orientation hashes folded
/// in with the `boost::hash_combine` step (wrapping `size_t` arithmetic).
pub(super) fn wire_hash(graph: &BRepGraph, wire: WireId) -> usize {
    co_edge_sequence(graph, wire)
        .into_iter()
        .fold(0_usize, |accumulated, (edge, reversed)| {
            let entry = [
                hash::<usize, u32>(&edge.index),
                hash::<usize, i32>(&i32::from(reversed)),
            ];
            accumulated
                ^ hash_words(&entry)
                    .wrapping_add(HASH_COMBINE_CONSTANT)
                    .wrapping_add(accumulated << 6)
                    .wrapping_add(accumulated >> 2)
        })
}

// Old wire -> the first equal wire of its hash bucket.
fn canonical_wires(graph: &BRepGraph) -> RewriteMap<WireId> {
    let nb_wires = graph.topo().wires().nb();
    let mut buckets: DataMap<usize, Vec<WireId>, DefaultHashPolicy> =
        DataMap::with_hasher(DefaultHashPolicy, nb_wires.max(1) as usize);
    for wire in full_ids::<WireTag>(graph) {
        if wire.is_removed(graph) {
            continue;
        }
        let key = wire_hash(graph, wire);
        buckets.try_bind(key, Vec::new());
        if let Some(bucket) = buckets.change_seek(&key) {
            bucket.push(wire);
        }
    }
    let mut canonical = rewrite_map(nb_wires);
    for bucket in buckets.values() {
        bind_equal_wires(graph, bucket, &mut canonical);
    }
    canonical
}

// Within one bucket: every unbound base binds the later unbound wires equal to it.
fn bind_equal_wires(graph: &BRepGraph, bucket: &[WireId], canonical: &mut RewriteMap<WireId>) {
    for (position, &base) in bucket.iter().enumerate() {
        if canonical.is_bound(&base) {
            continue;
        }
        for &candidate in &bucket[position + 1..] {
            if canonical.is_bound(&candidate) {
                continue;
            }
            if co_edge_sequence(graph, base) == co_edge_sequence(graph, candidate) {
                canonical.bind(candidate, base);
            }
        }
    }
}

// Every active face's active wire references that point at `old` point at `canon`.
fn redirect_face_wire_refs(
    graph: &mut BRepGraph,
    old: WireId,
    canon: WireId,
) -> Result<(), Failure> {
    for face in full_ids::<FaceTag>(graph) {
        if face.is_removed(graph) {
            continue;
        }
        let refs = graph.topo().faces().relations(face).wire_ref_ids.clone();
        for reference in refs {
            let child = graph.refs().wires().entry(reference).child_wire_id;
            if graph.refs().generic().is_removed(reference.into()) || child != old {
                continue;
            }
            let mut guard = graph.editor().wires().mutate_ref(reference)?;
            WireOps::set_ref_child_wire_id_guarded(&mut guard, canon);
        }
    }
    Ok(())
}
