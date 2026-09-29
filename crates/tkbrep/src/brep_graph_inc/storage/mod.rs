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
// Rust translation of ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Storage.hxx, .lxx
// and .cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraphInc_Storage`: the incidence-table backend of a graph.
//!
//! The members are split by responsibility: the record accessors (`accessors`), the
//! flag planes (`tags`), the UIDs (`uids`), the relation edits (`attach`, `detach`,
//! `rebind`, `relation_sets`), the whole-table rebuilds (`rebuild`), the checks
//! (`validate`, `wire_order`), the shape bindings (`bindings`), the removal (`removal`)
//! and the indexed-load preparation (`prepare`). docs/topology-design.md section 11.2
//! explains the layout.
mod accessors;
mod attach;
mod bindings;
mod detach;
mod prepare;
mod rebind;
mod rebuild;
mod relation_sets;
mod removal;
mod stores;
mod tags;
mod uids;
mod validate;
mod wire_order;

pub use bindings::{CachedShape, ShapeBindings};
pub use stores::{EntityStore, FlagPlane, OwnershipPlanes, RemovalPlanes, RepStore};
pub use tags::{NodeTag, RefTag, RepTag, StoredTag};
pub use validate::WireCoEdgeOrderStatus;

use super::definition::{
    CoEdgeDef, CompSolidDef, CompoundDef, EdgeDef, FaceDef, OccurrenceDef, ProductDef, ShellDef,
    SolidDef, VertexDef, WireDef,
};
use super::reference::{ChildRef, FaceRef, OccurrenceRef, ShellRef, SolidRef, VertexRef, WireRef};
use super::relations::{
    CompSolidRelations, CompoundRelations, EdgeRelations, FaceRelations, OccurrenceRelations,
    ProductRelations, ShellRelations, SolidRelations, VertexRelations, WireRelations,
};
use super::representation::{
    CoEdgeCurve2DRep, CoEdgePolygon2DRep, CoEdgePolygonOnTriRep, EdgeCurve3DRep, EdgePolygon3DRep,
    FaceSurfaceRep, FaceTriangulationRep,
};
use crate::brep_graph::{ChildRefId, NodeId, OccurrenceRefId, ProductId, RefId, RefUid, Uid};
use rustycad_tkernel::standard::Guid;
use std::collections::{BTreeMap, HashMap};
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// `BRepGraphInc_Storage`: every definition, reference and representation table, the
/// relation tables, the UID indexes, the shape bindings and the reconstruction cache.
///
/// ```
/// use rustycad_tkbrep::brep_graph_inc::Storage;
/// let mut storage = Storage::new();
/// assert!(storage.is_empty());
/// let vertex = storage.append_vertex();
/// assert_eq!(storage.nb_vertices(), 1);
/// assert!(!storage.is_removed(vertex));
/// ```
#[derive(Default)]
pub struct Storage {
    pub(crate) root_product_ids: Vec<ProductId>,
    pub(crate) deferred_modified: Vec<NodeId>,
    pub(crate) deferred_ref_modified: Vec<RefId>,
    pub(crate) vertices: EntityStore<VertexDef>,
    pub(crate) edges: EntityStore<EdgeDef>,
    pub(crate) co_edges: EntityStore<CoEdgeDef>,
    pub(crate) wires: EntityStore<WireDef>,
    pub(crate) faces: EntityStore<FaceDef>,
    pub(crate) shells: EntityStore<ShellDef>,
    pub(crate) solids: EntityStore<SolidDef>,
    pub(crate) compounds: EntityStore<CompoundDef>,
    pub(crate) comp_solids: EntityStore<CompSolidDef>,
    pub(crate) products: EntityStore<ProductDef>,
    pub(crate) occurrences: EntityStore<OccurrenceDef>,
    pub(crate) shell_refs: EntityStore<ShellRef>,
    pub(crate) face_refs: EntityStore<FaceRef>,
    pub(crate) wire_refs: EntityStore<WireRef>,
    pub(crate) vertex_refs: EntityStore<VertexRef>,
    pub(crate) solid_refs: EntityStore<SolidRef>,
    pub(crate) child_refs: EntityStore<ChildRef>,
    pub(crate) occurrence_refs: EntityStore<OccurrenceRef>,
    pub(crate) face_relations: Vec<FaceRelations>,
    pub(crate) wire_relations: Vec<WireRelations>,
    pub(crate) edge_relations: Vec<EdgeRelations>,
    pub(crate) shell_relations: Vec<ShellRelations>,
    pub(crate) solid_relations: Vec<SolidRelations>,
    pub(crate) compound_relations: Vec<CompoundRelations>,
    pub(crate) comp_solid_relations: Vec<CompSolidRelations>,
    pub(crate) vertex_relations: Vec<VertexRelations>,
    pub(crate) product_relations: Vec<ProductRelations>,
    pub(crate) occurrence_relations: Vec<OccurrenceRelations>,
    pub(crate) node_to_compounds: BTreeMap<NodeId, Vec<ChildRefId>>,
    pub(crate) node_to_occurrences: BTreeMap<NodeId, Vec<OccurrenceRefId>>,
    pub(crate) edge_curves3d: RepStore<EdgeCurve3DRep>,
    pub(crate) edge_polygons3d: RepStore<EdgePolygon3DRep>,
    pub(crate) co_edge_curves2d: RepStore<CoEdgeCurve2DRep>,
    pub(crate) co_edge_polygons2d: RepStore<CoEdgePolygon2DRep>,
    pub(crate) co_edge_polygons_on_tri: RepStore<CoEdgePolygonOnTriRep>,
    pub(crate) face_surfaces: RepStore<FaceSurfaceRep>,
    pub(crate) face_triangulations: RepStore<FaceTriangulationRep>,
    pub(crate) uid_to_node_id: RwLock<HashMap<Uid, NodeId>>,
    pub(crate) ref_uid_to_ref_id: RwLock<HashMap<RefUid, RefId>>,
    pub(crate) uid_to_node_id_dirty: AtomicBool,
    pub(crate) ref_uid_to_ref_id_dirty: AtomicBool,
    pub(crate) shape_bindings: RwLock<ShapeBindings>,
    generation: AtomicU32,
    graph_guid: Guid,
    deferred_mode: AtomicBool,
    propagation_wave: AtomicU32,
    remove_subgraph_depth: u32,
    pub(crate) current_shapes: RwLock<HashMap<NodeId, CachedShape>>,
}

impl std::fmt::Debug for Storage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Storage")
            .field("counts", &self.counts())
            .field("active_counts", &self.active_counts())
            .field("generation", &self.generation())
            .finish_non_exhaustive()
    }
}

impl Storage {
    /// `BRepGraphInc_Storage()`: empty.
    pub fn new() -> Self {
        Self::default()
    }

    /// `RootProductIds()`: products no active occurrence references.
    pub fn root_product_ids(&self) -> &[ProductId] {
        &self.root_product_ids
    }

    /// `ChangeRootProductIds()`.
    pub fn change_root_product_ids(&mut self) -> &mut Vec<ProductId> {
        &mut self.root_product_ids
    }

    /// `DeferredModified()`: nodes accumulated during deferred invalidation.
    pub fn deferred_modified(&self) -> &[NodeId] {
        &self.deferred_modified
    }

    /// `ChangeDeferredModified()`.
    pub fn change_deferred_modified(&mut self) -> &mut Vec<NodeId> {
        &mut self.deferred_modified
    }

    /// `DeferredRefModified()`.
    pub fn deferred_ref_modified(&self) -> &[RefId] {
        &self.deferred_ref_modified
    }

    /// `ChangeDeferredRefModified()`.
    pub fn change_deferred_ref_modified(&mut self) -> &mut Vec<RefId> {
        &mut self.deferred_ref_modified
    }

    /// `IsEmpty()`: no node of any kind has been allocated.
    pub fn is_empty(&self) -> bool {
        self.vertices.nb() == 0
            && self.edges.nb() == 0
            && self.co_edges.nb() == 0
            && self.wires.nb() == 0
            && self.faces.nb() == 0
            && self.shells.nb() == 0
            && self.solids.nb() == 0
            && self.compounds.nb() == 0
            && self.comp_solids.nb() == 0
            && self.products.nb() == 0
            && self.occurrences.nb() == 0
    }

    /// `HasAnyGuard()`: any definition or reference has an active MutGuard.
    pub fn has_any_guard(&self) -> bool {
        let planes: [&super::BitFlags; 18] = [
            &self.vertices.guard,
            &self.edges.guard,
            &self.co_edges.guard,
            &self.wires.guard,
            &self.faces.guard,
            &self.shells.guard,
            &self.solids.guard,
            &self.compounds.guard,
            &self.comp_solids.guard,
            &self.products.guard,
            &self.occurrences.guard,
            &self.vertex_refs.guard,
            &self.shell_refs.guard,
            &self.face_refs.guard,
            &self.wire_refs.guard,
            &self.solid_refs.guard,
            &self.child_refs.guard,
            &self.occurrence_refs.guard,
        ];
        planes.iter().any(|plane| plane.has_any_bit_set())
    }

    /// `Generation()`: the graph generation of the version stamps.
    pub fn generation(&self) -> u32 {
        self.generation.load(Ordering::Relaxed)
    }

    /// `SetGeneration(theGeneration)`.
    pub fn set_generation(&self, generation: u32) {
        self.generation.store(generation, Ordering::Relaxed);
    }

    /// `IncrementGeneration()`.
    pub fn increment_generation(&self) {
        self.generation.fetch_add(1, Ordering::Relaxed);
    }

    /// `GraphGUID()`.
    pub fn graph_guid(&self) -> &Guid {
        &self.graph_guid
    }

    /// `SetGraphGUID(theGuid)`.
    pub fn set_graph_guid(&mut self, guid: Guid) {
        self.graph_guid = guid;
    }

    /// `DeferredMode()`.
    pub fn deferred_mode(&self) -> bool {
        self.deferred_mode.load(Ordering::Relaxed)
    }

    /// `SetDeferredMode(theEnabled)`.
    pub fn set_deferred_mode(&self, enabled: bool) {
        self.deferred_mode.store(enabled, Ordering::Relaxed);
    }

    /// `PropagationWave()`.
    pub fn propagation_wave(&self) -> u32 {
        self.propagation_wave.load(Ordering::Relaxed)
    }

    /// `AdvancePropagationWave()`: increments the wave and returns the new value.
    pub fn advance_propagation_wave(&self) -> u32 {
        self.propagation_wave
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1)
    }

    /// `IncrementPropagationWave()`.
    pub fn increment_propagation_wave(&self) {
        self.propagation_wave.fetch_add(1, Ordering::Relaxed);
    }

    /// `RemoveSubgraphDepth()`.
    pub fn remove_subgraph_depth(&self) -> u32 {
        self.remove_subgraph_depth
    }

    /// `IncrementRemoveSubgraphDepth()`.
    pub fn increment_remove_subgraph_depth(&mut self) {
        self.remove_subgraph_depth += 1;
    }

    /// `DecrementRemoveSubgraphDepth()`: never below zero.
    pub fn decrement_remove_subgraph_depth(&mut self) {
        self.remove_subgraph_depth = self.remove_subgraph_depth.saturating_sub(1);
    }

    /// `ClearDeferredQueues()`.
    pub fn clear_deferred_queues(&mut self) {
        self.deferred_modified = Vec::new();
        self.deferred_ref_modified = Vec::new();
    }

    /// `Clear()`: every table, the UID indexes, the shape cache and the deferred queues;
    /// the UID counters stay monotonic.
    pub fn clear(&mut self) {
        self.clear_storage_for_reuse();
        self.clear_uid_indexes();
        self.clear_shape_cache();
        self.clear_deferred_queues();
    }

    // `ClearStorageForReuse()`: the stores, the relation tables and the bindings.
    fn clear_storage_for_reuse(&mut self) {
        self.root_product_ids = Vec::new();
        self.clear_entity_stores();
        self.clear_relations();
        self.clear_rep_stores();
        self.clear_shape_bindings();
    }

    fn clear_entity_stores(&mut self) {
        self.vertices.clear();
        self.edges.clear();
        self.co_edges.clear();
        self.wires.clear();
        self.faces.clear();
        self.shells.clear();
        self.solids.clear();
        self.compounds.clear();
        self.comp_solids.clear();
        self.products.clear();
        self.occurrences.clear();
        self.shell_refs.clear();
        self.face_refs.clear();
        self.wire_refs.clear();
        self.vertex_refs.clear();
        self.solid_refs.clear();
        self.child_refs.clear();
        self.occurrence_refs.clear();
    }

    fn clear_rep_stores(&mut self) {
        self.edge_curves3d.clear();
        self.edge_polygons3d.clear();
        self.co_edge_curves2d.clear();
        self.co_edge_polygons2d.clear();
        self.co_edge_polygons_on_tri.clear();
        self.face_surfaces.clear();
        self.face_triangulations.clear();
    }

    // `ClearRelations()`: the relation tables and the sparse incoming maps.
    pub(crate) fn clear_relations(&mut self) {
        self.face_relations = Vec::new();
        self.wire_relations = Vec::new();
        self.edge_relations = Vec::new();
        self.shell_relations = Vec::new();
        self.solid_relations = Vec::new();
        self.compound_relations = Vec::new();
        self.comp_solid_relations = Vec::new();
        self.vertex_relations = Vec::new();
        self.product_relations = Vec::new();
        self.occurrence_relations = Vec::new();
        self.node_to_compounds.clear();
        self.node_to_occurrences.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_and_modes_round_trip() {
        let mut storage = Storage::new();
        storage.set_generation(4);
        storage.increment_generation();
        assert_eq!(storage.generation(), 5);
        assert_eq!(storage.advance_propagation_wave(), 1);
        storage.increment_propagation_wave();
        assert_eq!(storage.propagation_wave(), 2);
        storage.set_deferred_mode(true);
        assert!(storage.deferred_mode());
        storage.increment_remove_subgraph_depth();
        storage.decrement_remove_subgraph_depth();
        storage.decrement_remove_subgraph_depth();
        assert_eq!(storage.remove_subgraph_depth(), 0);
        storage.change_deferred_modified().push(NodeId::default());
        storage
            .change_deferred_ref_modified()
            .push(RefId::default());
        storage.clear_deferred_queues();
        assert!(storage.deferred_modified().is_empty());
        assert!(storage.deferred_ref_modified().is_empty());
        storage.change_root_product_ids().push(ProductId::new(0));
        assert_eq!(storage.root_product_ids().len(), 1);
        storage.set_graph_guid(Guid::new());
        assert_eq!(*storage.graph_guid(), Guid::new());
        assert!(!storage.has_any_guard());
    }
}
