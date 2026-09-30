// Rust translation of the free-coedge mapping and the face/p-curve transfer of
// `BRepGraph_Compact::Perform` (ModelingData/TKBRep/BRepGraph/BRepGraph_Compact.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The coedges after the faces exist: the free coedges (outside any wire) matched by
//! edge and face, then every mapped coedge's face, orientation and p-curve.
use super::maps::NodeMaps;
use crate::brep_graph::editor::CoEdgeOps;
use crate::brep_graph::id_walk::active_ids;
use crate::brep_graph::tool;
use crate::brep_graph::*;
use rustycad_tkernel::standard::Failure;

/// Maps each unmapped active old coedge to the first active new coedge with the
/// remapped edge and face. The derived relations of the new graph are not rebuilt yet,
/// so the new coedges are scanned linearly (free coedges are rare). The scan skips a
/// new coedge whose id is bound as an old key, the native check.
pub(super) fn map_free_co_edges(graph: &BRepGraph, fresh: &BRepGraph, maps: &mut NodeMaps) {
    let new_co_edges = active_ids::<CoEdgeTag>(fresh);
    for old in active_ids::<CoEdgeTag>(graph) {
        if maps.co_edges.is_bound(&old) || graph.inc_storage().is_removed(old) {
            continue;
        }
        let def = graph.topo().co_edges().definition(old);
        let edge: EdgeId = maps.remap_typed(def.child_edge_id);
        let face: FaceId = maps.remap_typed(def.face_id);
        if !edge.is_valid() || !face.is_valid() {
            continue;
        }
        let found = new_co_edges.iter().copied().find(|&new| {
            let candidate = fresh.topo().co_edges().definition(new);
            !fresh.inc_storage().is_removed(new)
                && candidate.child_edge_id == edge
                && candidate.face_id == face
                && !maps.co_edges.is_bound(&new)
        });
        if let Some(new) = found {
            maps.co_edges.bind(old, new);
        }
    }
}

/// "Transfer face/PCurve attributes": each mapped coedge takes the remapped face and
/// the old orientation under its guard. A coedge whose face was dropped keeps only its
/// free-wire use (its face-bound representation is reset); otherwise it takes the old
/// p-curve with its range, or only the range without a p-curve.
pub(super) fn transfer_co_edge_attributes(
    graph: &BRepGraph,
    fresh: &mut BRepGraph,
    maps: &NodeMaps,
) -> Result<(), Failure> {
    for old in active_ids::<CoEdgeTag>(graph) {
        let Some(&new) = maps.co_edges.seek(&old) else {
            continue;
        };
        let def = graph.topo().co_edges().definition(old);
        let face: FaceId = maps.remap_typed(def.face_id);
        let mut guard = fresh.editor().co_edges().mutate(new)?;
        CoEdgeOps::set_face_id_guarded(&mut guard, face);
        CoEdgeOps::set_orientation_guarded(&mut guard, def.orientation);
        if !guard.face_id.is_valid() {
            CoEdgeOps::reset_pcurve_binding_guarded(&mut guard);
            continue;
        }
        if !def.curve2d_rep_id.is_valid() {
            let (first, last) = tool::co_edge::range(graph, old);
            CoEdgeOps::set_param_range_guarded(&mut guard, first, last);
            continue;
        }
        let pcurve = tool::co_edge::pcurve(graph, old);
        if !pcurve.is_null() {
            let (first, last) = tool::co_edge::range(graph, old);
            let editor = guard.graph_mut().editor();
            editor.co_edges().set_pcurve(new, &pcurve, first, last)?;
        }
    }
    Ok(())
}
