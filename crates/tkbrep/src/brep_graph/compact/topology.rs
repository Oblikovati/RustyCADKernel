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
// Rust translation of the vertex, edge, wire and face rebuild of
// `BRepGraph_Compact::Perform` (ModelingData/TKBRep/BRepGraph/BRepGraph_Compact.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The bottom-up rebuild of the mapped vertices, edges, wires and faces into the fresh
//! graph, each kind followed by the copy of the old generations and UIDs.
use super::maps::{IdMap, NodeMaps, RefMaps};
use crate::brep::errors::program_error;
use crate::brep_graph::editor::{VertexOps, WireOps};
use crate::brep_graph::id_walk::active_ids;
use crate::brep_graph::refs_iterator::{CoEdgesOfWire, RefsWireOfFace};
use crate::brep_graph::tool;
use crate::brep_graph::*;
use crate::brep_graph_inc::HasBaseDef;
use rustycad_tkernel::standard::Failure;

/// "Copy OwnGen/SubtreeGen" of one kind: every mapped active node takes its old
/// `OwnGen`, `SubtreeGen` and `UID`.
pub(super) fn copy_generations<T: NodeTag>(graph: &BRepGraph, fresh: &mut BRepGraph, map: &IdMap<T>) {
    for old in active_ids::<T>(graph) {
        let Some(&new) = map.seek(&old) else {
            continue;
        };
        let source = graph.inc_storage().def(old).base().clone();
        let target = fresh.inc_storage_mut().change_def(new).base_mut();
        target.own_gen = source.own_gen;
        target.subtree_gen = source.subtree_gen;
        target.uid = source.uid;
    }
}

// The `Standard_ASSERT_RAISE` of a rebuilt id that differs from its precomputed one.
fn unexpected_id(what: &str, got: impl std::fmt::Debug, expected: impl std::fmt::Debug) -> Failure {
    program_error(
        &format!("BRepGraph_Compact: unexpected {what} id"),
        &format!("{got:?} (expected {expected:?})"),
    )
}

/// The vertices, in active order, at their precomputed ids.
pub(super) fn rebuild_vertices(graph: &BRepGraph, fresh: &mut BRepGraph, maps: &NodeMaps) -> Result<(), Failure> {
    for old in active_ids::<VertexTag>(graph) {
        let Some(&expected) = maps.vertices.seek(&old) else {
            continue;
        };
        let def = graph.topo().vertices().definition(old);
        let new = fresh.editor().vertices().add(def.point, def.tolerance);
        if new != expected {
            return Err(unexpected_id("vertex", new, expected));
        }
    }
    copy_generations(graph, fresh, &maps.vertices);
    Ok(())
}

// The remapped child vertex of an edge's start or end reference.
fn remapped_end(graph: &BRepGraph, maps: &NodeMaps, reference: VertexRefId) -> VertexId {
    if !reference.is_valid() {
        return VertexId::invalid();
    }
    let old = graph.refs().vertices().entry(reference).child_vertex_id;
    if !old.is_valid() {
        return VertexId::invalid();
    }
    maps.remap_typed(old)
}

/// The edges with their remapped vertices, shared curve and range; each new vertex
/// reference takes the old orientation through a reference guard (under the edge's
/// guard) and is mapped.
pub(super) fn rebuild_edges(
    graph: &BRepGraph,
    fresh: &mut BRepGraph,
    maps: &NodeMaps,
    refs: &mut RefMaps,
) -> Result<(), Failure> {
    for old in active_ids::<EdgeTag>(graph) {
        if maps.edges.seek(&old).is_none() {
            continue;
        }
        let def = graph.topo().edges().definition(old).clone();
        let start = remapped_end(graph, maps, tool::edge::start_vertex_id(graph, old));
        let end = remapped_end(graph, maps, tool::edge::end_vertex_id(graph, old));
        let curve = tool::edge::curve(graph, old);
        let (first, last) = tool::edge::range(graph, old);
        let new = fresh
            .editor()
            .edges()
            .add(start, end, &curve, first, last, def.tolerance);
        let mut guard = fresh.editor().edges().mutate(new)?;
        let new_refs = [guard.start_vertex_ref_id, guard.end_vertex_ref_id];
        let old_refs = [def.start_vertex_ref_id, def.end_vertex_ref_id];
        for (old_ref, new_ref) in old_refs.into_iter().zip(new_refs) {
            if !old_ref.is_valid() || !new_ref.is_valid() {
                continue;
            }
            let orientation = graph.refs().vertices().entry(old_ref).orientation;
            let mut ref_guard = guard.graph_mut().editor().vertices().mutate_ref(new_ref)?;
            VertexOps::set_ref_orientation_guarded(&mut ref_guard, orientation);
            drop(ref_guard);
            refs.vertex_refs.bind(old_ref, new_ref);
        }
    }
    copy_generations(graph, fresh, &maps.edges);
    Ok(())
}

/// The wires, each created empty at its precomputed id, then a coedge use per coedge
/// whose edge is mapped (face unset); the coedges are mapped pairwise.
pub(super) fn rebuild_wires(graph: &BRepGraph, fresh: &mut BRepGraph, maps: &mut NodeMaps) -> Result<(), Failure> {
    for old in active_ids::<WireTag>(graph) {
        let Some(&expected) = maps.wires.seek(&old) else {
            continue;
        };
        let new = fresh.editor().wires().add(&[]);
        if new != expected {
            return Err(unexpected_id("wire", new, expected));
        }
        let mut pairs = Vec::new();
        for co_edge in CoEdgesOfWire::new(graph, old) {
            let def = graph.topo().co_edges().definition(co_edge);
            let edge: EdgeId = maps.remap_typed(def.child_edge_id);
            if !edge.is_valid() {
                continue;
            }
            let storage = fresh.inc_storage_mut();
            let use_id = storage.create_co_edge_use(new, edge, FaceId::invalid(), def.orientation);
            fresh.allocate_uid(use_id.into());
            pairs.push((co_edge, use_id));
        }
        for (old_co_edge, new_co_edge) in pairs {
            maps.co_edges.bind(old_co_edge, new_co_edge);
        }
    }
    copy_generations(graph, fresh, &maps.wires);
    Ok(())
}

// The remapped wires of a face's active references, the outer one first, with the old
// references they come from.
fn remapped_face_wires(graph: &BRepGraph, maps: &NodeMaps, face: FaceId) -> (WireId, Vec<WireId>, Vec<WireRefId>) {
    let mut outer = WireId::invalid();
    let (mut inner, mut old_refs) = (Vec::new(), Vec::new());
    for reference in RefsWireOfFace::new(graph, face) {
        let child = graph.refs().wires().entry(reference).child_wire_id;
        let wire: WireId = maps.remap_typed(child);
        if !wire.is_valid() {
            continue;
        }
        if !outer.is_valid() {
            outer = wire;
        } else {
            inner.push(wire);
        }
        old_refs.push(reference);
    }
    (outer, inner, old_refs)
}

/// The faces with their shared surface and remapped wires; each new wire reference
/// takes the old orientation through a reference guard (under the face's guard) and is
/// mapped.
pub(super) fn rebuild_faces(
    graph: &BRepGraph,
    fresh: &mut BRepGraph,
    maps: &NodeMaps,
    refs: &mut RefMaps,
) -> Result<(), Failure> {
    for old in active_ids::<FaceTag>(graph) {
        if maps.faces.seek(&old).is_none() {
            continue;
        }
        let tolerance = graph.topo().faces().definition(old).tolerance;
        let surface = tool::face::surface(graph, old);
        let (outer, inner, old_refs) = remapped_face_wires(graph, maps, old);
        let new = fresh.editor().faces().add(&surface, outer, &inner, tolerance);
        let mut guard = fresh.editor().faces().mutate(new)?;
        let new_refs = guard.graph().topo().faces().relations(new).wire_ref_ids.clone();
        for (new_ref, old_ref) in new_refs.into_iter().zip(old_refs) {
            let orientation = graph.refs().wires().entry(old_ref).orientation;
            let mut ref_guard = guard.graph_mut().editor().wires().mutate_ref(new_ref)?;
            WireOps::set_ref_orientation_guarded(&mut ref_guard, orientation);
            drop(ref_guard);
            refs.wire_refs.bind(old_ref, new_ref);
        }
    }
    copy_generations(graph, fresh, &maps.faces);
    Ok(())
}
