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
// Rust translation of "Phase 4: Face Merging" of `BRepGraph_Deduplicate::Perform`
// (ModelingData/TKBRep/BRepGraph/BRepGraph_Deduplicate.cxx) with its `FaceKey`,
// `FaceKeyHasher` and `faceWireHash` at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The face merge: faces sharing a surface handle and the same set of wires collapse
//! onto the first face of their group.
use super::edges::{handle_address, hash_words};
use super::geometry::{RewriteMap, rewrite_map};
use super::redirect::{redirect_generic_parents, replace_and_record};
use super::{DeduplicateResult, Options};
use crate::brep_graph::editor::{CoEdgeOps, FaceOps};
use crate::brep_graph::id_walk::full_ids;
use crate::brep_graph::layer_history::LayerHistory;
use crate::brep_graph::refs_iterator::RefsWireOfFace;
use crate::brep_graph::tool;
use crate::brep_graph::{BRepGraph, CoEdgeTag, FaceId, FaceRefId, FaceTag, NodeId, WireId};
use rustycad_tkernel::ncollection::{DataMap, HashPolicy};
use rustycad_tkernel::standard::hashing::hash;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::stl::sort_by;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::geom::GeomSurface;

// The tolerance ratio above which a candidate face is not merged.
const FACE_TOLERANCE_RATIO: f64 = 10.0;

/// `FaceKey`: the surface handle and the XOR of the face's wire hashes.
#[derive(Clone)]
struct FaceKey {
    surface: Handle<dyn GeomSurface>,
    wire_hash: usize,
}

impl std::fmt::Debug for FaceKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FaceKey")
            .field("surface", &self.surface.object_address())
            .field("wire_hash", &self.wire_hash)
            .finish()
    }
}

/// `FaceKeyHasher`: `hashBytes` of the handle address and the wire hash.
#[derive(Clone, Copy, Debug, Default)]
struct FaceKeyHasher;

impl HashPolicy<FaceKey> for FaceKeyHasher {
    fn hash(&self, key: &FaceKey) -> usize {
        hash_words(&[handle_address(&key.surface), key.wire_hash])
    }

    fn equal(&self, first: &FaceKey, second: &FaceKey) -> bool {
        first.surface == second.surface && first.wire_hash == second.wire_hash
    }
}

/// Phase 4: groups the active faces with a surface by [`FaceKey`], finds the canonical
/// face of each duplicate, then (unless analyzing) redirects the shell face references,
/// the coedge faces, the compound and occurrence children, replaces each merged face and
/// records a `Dedup:MergeFace`.
pub(super) fn merge(
    graph: &mut BRepGraph,
    history: &Handle<LayerHistory>,
    options: &Options,
    result: &mut DeduplicateResult,
) -> Result<(), Failure> {
    let canonical = canonical_faces(graph);
    if options.analyze_only {
        result.nb_merged_faces = canonical.len() as u32;
        return Ok(());
    }
    let label = AsciiString::new("Dedup:MergeFace");
    for (&old, &canon) in canonical.items() {
        redirect_shell_face_refs(graph, old, canon)?;
        // Before the removal: a coedge left on the old face would keep a live p-curve
        // under a dangling face after compaction.
        redirect_co_edge_faces(graph, old, canon)?;
        redirect_generic_parents(graph, old.into(), canon.into())?;
        replace_and_record(graph, history, &label, old.into(), canon.into())?;
        result.nb_history_records += 1;
        result.nb_merged_faces += 1;
    }
    Ok(())
}

// `faceWireHash`: the XOR of `opencascade::hash` of each active wire reference's child.
fn face_wire_hash(graph: &BRepGraph, face: FaceId) -> usize {
    face_wires(graph, face)
        .into_iter()
        .fold(0, |accumulated, wire| {
            accumulated ^ hash::<usize, u32>(&wire.index)
        })
}

// The children of the face's active wire references.
fn face_wires(graph: &BRepGraph, face: FaceId) -> Vec<WireId> {
    RefsWireOfFace::new(graph, face)
        .map(|reference| graph.refs().wires().entry(reference).child_wire_id)
        .collect()
}

// The groups of equal keys, in full-iterator order within a group.
fn face_groups(graph: &BRepGraph) -> DataMap<FaceKey, Vec<FaceId>, FaceKeyHasher> {
    let buckets = graph.topo().faces().nb().max(1) as usize;
    let mut groups = DataMap::with_hasher(FaceKeyHasher, buckets);
    for face in full_ids::<FaceTag>(graph) {
        if face.is_removed(graph) || !tool::face::has_surface(graph, face) {
            continue;
        }
        let key = FaceKey {
            surface: tool::face::surface(graph, face),
            wire_hash: face_wire_hash(graph, face),
        };
        groups.try_bind(key.clone(), Vec::new());
        if let Some(group) = groups.change_seek(&key) {
            group.push(face);
        }
    }
    groups
}

// Candidate -> first face of its group, for candidates whose tolerance is at most ten
// times the canonical one and whose active wires equal the canonical face's.
fn canonical_faces(graph: &BRepGraph) -> RewriteMap<FaceId> {
    let groups = face_groups(graph);
    let mut canonical = rewrite_map(graph.topo().faces().nb());
    for group in groups.values().filter(|group| group.len() >= 2) {
        let canon = group[0];
        let canon_tolerance = graph.topo().faces().definition(canon).tolerance;
        for &candidate in &group[1..] {
            let tolerance = graph.topo().faces().definition(candidate).tolerance;
            if tolerance > canon_tolerance * FACE_TOLERANCE_RATIO {
                continue;
            }
            if sorted_live_wires(graph, canon) == sorted_live_wires(graph, candidate) {
                canonical.bind(candidate, canon);
            }
        }
    }
    canonical
}

// The face's active, not removed wires sorted by index (libstdc++ `std::sort`), for an
// order-independent comparison.
fn sorted_live_wires(graph: &BRepGraph, face: FaceId) -> Vec<WireId> {
    let mut wires: Vec<WireId> = face_wires(graph, face)
        .into_iter()
        .filter(|wire| !wire.is_removed(graph))
        .collect();
    sort_by(&mut wires, |first, second| first.index < second.index);
    wires
}

// Every active face reference (of the full reference store) that points at `old`
// points at `canon`.
fn redirect_shell_face_refs(
    graph: &mut BRepGraph,
    old: FaceId,
    canon: FaceId,
) -> Result<(), Failure> {
    for index in 0..graph.refs().faces().nb() {
        let reference = FaceRefId::new(index);
        let child = graph.refs().faces().entry(reference).child_face_id;
        if graph.refs().generic().is_removed(reference.into()) || child != old {
            continue;
        }
        let mut guard = graph.editor().faces().mutate_ref(reference)?;
        FaceOps::set_ref_face_id_guarded(&mut guard, canon);
    }
    Ok(())
}

// Every active coedge bound to `old` is bound to `canon`.
fn redirect_co_edge_faces(
    graph: &mut BRepGraph,
    old: FaceId,
    canon: FaceId,
) -> Result<(), Failure> {
    for co_edge in full_ids::<CoEdgeTag>(graph) {
        if NodeId::from(co_edge).is_removed(graph) {
            continue;
        }
        if graph.topo().co_edges().definition(co_edge).face_id != old {
            continue;
        }
        let mut guard = graph.editor().co_edges().mutate(co_edge)?;
        CoEdgeOps::set_face_id_guarded(&mut guard, canon);
    }
    Ok(())
}
