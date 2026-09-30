// Rust translation of BRepGraph_CacheMesh::CopyFreshTo of
// ModelingData/TKBRep/BRepGraph/BRepGraph_CacheMesh.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Copying the fresh mesh entries into the mesh cache of a copied graph.
use super::access::ensure_size;
use super::entries::{CoEdgeMeshEntry, Slot};
use super::{CacheMesh, MeshSlots};
use crate::brep_graph::copy_remap::CopyRemap;
use crate::brep_graph::layer::remapped_node;
use crate::brep_graph::{BRepGraph, CoEdgeId, EdgeId, FaceId};
use rustycad_tkernel::standard::Failure;

// The two caches and graphs of one copy.
struct MeshCopy<'c> {
    copy: &'c CopyRemap<'c>,
    source: &'c CacheMesh,
    target: &'c CacheMesh,
}

impl MeshCopy<'_> {
    fn source_graph(&self) -> &BRepGraph {
        self.copy.source_graph()
    }

    fn target_graph(&self) -> &BRepGraph {
        self.copy.target_graph_const()
    }
}

impl CacheMesh {
    /// `CopyFreshTo(theCopy)`: the target graph's mesh cache (`Ensure`d) takes the
    /// active slot, then, per source slot, its driver and recipe; the target slot is
    /// cleared and receives the fresh face meshes (their mesh generation bumped), edge
    /// meshes and coedge polygons of the mapped entities, re-bound in the target graph.
    /// The source slots are read first, so a copy into this very cache cannot deadlock.
    pub(super) fn copy_slots_to(&self, copy: &CopyRemap<'_>) -> Result<(), Failure> {
        let target = copy
            .target_graph_const()
            .cache_registry()
            .ensure::<CacheMesh>()?;
        let Some(target) = target.get() else {
            return Ok(());
        };
        let (source_slots, active_slot): (Vec<Slot>, u32) = {
            let slots = self.slots.read();
            (slots.slots.clone(), slots.active_slot)
        };
        target.set_active_display_slot(active_slot);
        let ends = MeshCopy {
            copy,
            source: self,
            target,
        };
        let mut target_slots = target.slots.write();
        for source_slot in &source_slots {
            copy_slot(&ends, source_slot, &mut target_slots)?;
        }
        Ok(())
    }
}

// One source slot.
fn copy_slot(
    ends: &MeshCopy<'_>,
    source: &Slot,
    target_slots: &mut MeshSlots,
) -> Result<(), Failure> {
    let target = target_slots.change_slot(source.id);
    target.driver = source.driver.clone();
    target.recipe_hash = source.recipe_hash;
    target.clear();
    copy_faces(ends, source, target)?;
    copy_edges(ends, source, target)?;
    copy_co_edges(ends, source, target)
}

fn copy_faces(ends: &MeshCopy<'_>, source: &Slot, target: &mut Slot) -> Result<(), Failure> {
    let count = ends.target_graph().topo().faces().nb();
    for index in 0..source.face_meshes.len() as u32 {
        let (face, entry) = (FaceId::new(index), &source.face_meshes[index as usize]);
        if !ends
            .source
            .is_face_entry_fresh(ends.source_graph(), entry, source)
        {
            continue;
        }
        let target_face = remapped_node(ends.copy, face);
        if !target_face.is_valid_for(count) {
            continue;
        }
        let mut copied = ensure_size(&mut target.face_meshes, target_face.index)?.clone();
        copied.triangulation = entry.triangulation.clone();
        ends.target
            .bind_face_entry(ends.target_graph(), &mut copied, target_face, target);
        copied.mesh_generation = copied.mesh_generation.wrapping_add(1);
        target.face_meshes[target_face.index as usize] = copied;
    }
    Ok(())
}

fn copy_edges(ends: &MeshCopy<'_>, source: &Slot, target: &mut Slot) -> Result<(), Failure> {
    let count = ends.target_graph().topo().edges().nb();
    for index in 0..source.edge_meshes.len() as u32 {
        let (edge, entry) = (EdgeId::new(index), &source.edge_meshes[index as usize]);
        if !ends
            .source
            .is_edge_entry_fresh(ends.source_graph(), entry, source)
        {
            continue;
        }
        let target_edge = remapped_node(ends.copy, edge);
        if !target_edge.is_valid_for(count) {
            continue;
        }
        let mut copied = ensure_size(&mut target.edge_meshes, target_edge.index)?.clone();
        copied.polygon3d = entry.polygon3d.clone();
        ends.target
            .bind_edge_entry(ends.target_graph(), &mut copied, target_edge, target);
        target.edge_meshes[target_edge.index as usize] = copied;
    }
    Ok(())
}

fn copy_co_edges(ends: &MeshCopy<'_>, source: &Slot, target: &mut Slot) -> Result<(), Failure> {
    for index in 0..source.co_edge_meshes.len() as u32 {
        copy_co_edge(ends, source, target, CoEdgeId::new(index))?;
    }
    Ok(())
}

// Which sides of a source coedge entry are fresh (the 2D polygon, the polygons on
// triangulation).
#[derive(Clone, Copy)]
struct FreshSides {
    polygon2d: bool,
    on_tri: bool,
}

// One coedge: its fresh 2D polygon, and its fresh polygons on triangulation when the
// target coedge's face has a cached triangulation in the slot.
fn copy_co_edge(
    ends: &MeshCopy<'_>,
    source: &Slot,
    target: &mut Slot,
    co_edge: CoEdgeId,
) -> Result<(), Failure> {
    let Some(raw) = source.co_edge_raw(co_edge) else {
        return Ok(());
    };
    let Some(mut sides) = fresh_sides(ends, source, raw) else {
        return Ok(());
    };
    let target_graph = ends.target_graph();
    let target_co_edge = remapped_node(ends.copy, co_edge);
    if !target_co_edge.is_valid_for(target_graph.topo().co_edges().nb()) {
        return Ok(());
    }
    if sides.on_tri {
        let face = target_graph.inc_storage().co_edge(target_co_edge).face_id;
        sides.on_tri = face.is_valid() && target.face_raw(face).is_some();
    }
    let mut copied = ensure_size(&mut target.co_edge_meshes, target_co_edge.index)?.clone();
    merge_sides(&mut copied, raw, sides);
    if copied.is_present() {
        ends.target
            .bind_co_edge_entry(target_graph, &mut copied, target_co_edge, target);
    } else {
        copied.reset();
    }
    target.co_edge_meshes[target_co_edge.index as usize] = copied;
    Ok(())
}

// The fresh sides of a source entry; `None` when neither is fresh.
fn fresh_sides(ends: &MeshCopy<'_>, source: &Slot, raw: &CoEdgeMeshEntry) -> Option<FreshSides> {
    let graph = ends.source_graph();
    let sides = FreshSides {
        polygon2d: ends.source.is_co_edge_polygon2d_fresh(graph, raw, source),
        on_tri: ends
            .source
            .is_co_edge_polygon_on_tri_fresh(graph, raw, source),
    };
    (sides.polygon2d || sides.on_tri).then_some(sides)
}

// The fresh sides copied into the target entry (the non-null polygons on triangulation
// are appended).
fn merge_sides(copied: &mut CoEdgeMeshEntry, raw: &CoEdgeMeshEntry, sides: FreshSides) {
    if sides.polygon2d {
        copied.polygon2d = raw.polygon2d.clone();
    }
    if sides.on_tri {
        let polygons = raw
            .polygons_on_tri
            .iter()
            .filter(|polygon| polygon.is_some());
        copied.polygons_on_tri.extend(polygons.cloned());
    }
}
