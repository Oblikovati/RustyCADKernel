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
// Rust translation of ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Reconstruct.hxx and
// the Cache, Node and replaySupplementAttachments parts of BRepGraphInc_Reconstruct.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraphInc_Reconstruct`: rebuilds `TopoDS` shapes from the incidence storage.
//!
//! Every node becomes a new TShape; a [`Cache`] shared across calls makes shared
//! sub-shapes (vertices, edges, wires, faces) one TShape each. SameRange and
//! SameParameter are computed from the storage (the uncached values of the
//! `BRepGraph_CacheDerivedState` service, equal to the cached ones on an unchanged
//! graph).
mod edge;
mod face;

use crate::brep::BRepBuilder;
use crate::brep_graph::TypedId;
use crate::brep_graph::cache_derived_state;
use crate::brep_graph::hooks::TopoSupplementHooks;
use crate::brep_graph::{
    BRepGraph, CompSolidId, CompoundId, EdgeId, FaceId, NodeId, NodeKind, ShellId, SolidId,
    VertexId, WireId,
};
use crate::brep_graph_inc::{RepTag, Storage};
use crate::topods::{Builder, CompSolid, Compound, Shape, Shell, Solid, Vertex, Wire};
use rustycad_tkernel::standard::Failure;

pub use face::face_with_cache;

/// `BRepGraphInc_Reconstruct::Cache`: the reconstructed shape of each node, in one
/// dense vector per node kind (the reserved kind 9 included).
///
/// ```
/// use rustycad_tkbrep::brep_graph::{NodeId, NodeKind};
/// use rustycad_tkbrep::brep_graph_inc::reconstruct::Cache;
/// use rustycad_tkbrep::topods::Shape;
/// let cache = Cache::new();
/// assert!(!cache.is_bound(NodeId::new(NodeKind::FACE, 0)));
/// ```
#[derive(Clone, Debug)]
pub struct Cache {
    kinds: Vec<Vec<Shape>>,
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache {
    /// `THE_KIND_COUNT`.
    pub const KIND_COUNT: i32 = NodeId::KIND_COUNT;
    /// `THE_DEFAULT_INCREMENT`: the native block size of the per-kind vectors.
    pub const DEFAULT_INCREMENT: i32 = 32;

    /// `Cache()`: empty vectors for every kind; `Cache::new()`.
    pub fn new() -> Self {
        Self {
            kinds: vec![Vec::new(); Self::KIND_COUNT as usize],
        }
    }

    // The vector of a kind, `None` outside the dense kinds.
    fn slot(&self, kind: NodeKind) -> Option<&Vec<Shape>> {
        usize::try_from(kind.0)
            .ok()
            .and_then(|index| self.kinds.get(index))
    }

    /// `Seek(theNode)`: the cached non-null shape of the node;
    /// `cache.seek(NodeId::new(NodeKind::EDGE, 3))`.
    pub fn seek(&self, node: NodeId) -> Option<&Shape> {
        let shape = self.slot(node.kind)?.get(node.index as usize)?;
        (!shape.is_null()).then_some(shape)
    }

    /// `Bind(theNode, theShape)`: stores the shape, growing the vector with null
    /// shapes; ignored outside the dense kinds; `cache.bind(node, &shape)`.
    pub fn bind(&mut self, node: NodeId, shape: &Shape) {
        let Some(vector) = usize::try_from(node.kind.0)
            .ok()
            .and_then(|index| self.kinds.get_mut(index))
        else {
            return;
        };
        let index = node.index as usize;
        if index >= vector.len() {
            vector.resize(index + 1, Shape::default());
        }
        vector[index] = shape.clone();
    }

    /// `IsBound(theNode)`: a cached non-null shape exists; `cache.is_bound(node)`.
    pub fn is_bound(&self, node: NodeId) -> bool {
        self.seek(node).is_some()
    }
}

/// `Node(theGraph, theNode)`: the shape of a node with a fresh cache (shared
/// sub-shapes are shared within this call only); null for an invalid node, a coedge, a
/// product or an occurrence.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, NodeId};
/// use rustycad_tkbrep::brep_graph_inc::reconstruct;
/// let graph = BRepGraph::new();
/// assert!(reconstruct::node(&graph, NodeId::default()).unwrap().is_null());
/// ```
pub fn node(graph: &BRepGraph, node: NodeId) -> Result<Shape, Failure> {
    let mut cache = Cache::new();
    node_with_cache(graph, node, &mut cache)
}

/// `Node(theGraph, theNode, theCache)`: the cached shape of the node, else a new one
/// (with the supplement attachments replayed) bound in the cache. A node index out of
/// its kind's range is `Standard_OutOfRange` (undefined natively).
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, NodeId, NodeKind};
/// use rustycad_tkbrep::brep_graph_inc::reconstruct::{self, Cache};
/// let graph = BRepGraph::new();
/// let mut cache = Cache::new();
/// let error = reconstruct::node_with_cache(&graph, NodeId::new(NodeKind::EDGE, 0), &mut cache);
/// assert_eq!(error.unwrap_err().exception_type(), "Standard_OutOfRange");
/// ```
pub fn node_with_cache(
    graph: &BRepGraph,
    node: NodeId,
    cache: &mut Cache,
) -> Result<Shape, Failure> {
    if !node.is_valid() {
        return Ok(Shape::default());
    }
    if let Some(cached) = cache.seek(node) {
        return Ok(cached.clone());
    }
    let mut result = match node.kind {
        NodeKind::FACE => return face_with_cache(graph, FaceId::from_node_id(node), cache),
        NodeKind::CO_EDGE | NodeKind::PRODUCT | NodeKind::OCCURRENCE => return Ok(Shape::default()),
        _ => build_node(graph, node, cache)?,
    };
    replay_supplement(graph, node, &mut result)?;
    cache.bind(node, &result);
    Ok(result)
}

// The shape of a vertex, edge, wire or container node, checking the index range.
fn build_node(graph: &BRepGraph, node: NodeId, cache: &mut Cache) -> Result<Shape, Failure> {
    if node.index >= graph.topo().generic().nb(node.kind) {
        return Err(Failure::out_of_range(&format!(
            "BRepGraphInc_Reconstruct::Node: node (kind {}, index {}) is out of range",
            node.kind.0, node.index
        )));
    }
    match node.kind {
        NodeKind::VERTEX => Ok(make_vertex(graph, VertexId::from_node_id(node))?.into()),
        NodeKind::EDGE => {
            Ok(edge::build_edge(graph, EdgeId::from_node_id(node), cache, true)?.into())
        }
        NodeKind::WIRE => Ok(build_wire(graph, WireId::from_node_id(node), cache)?.into()),
        NodeKind::SHELL => Ok(build_shell(graph, ShellId::from_node_id(node), cache)?.into()),
        NodeKind::SOLID => Ok(build_solid(graph, SolidId::from_node_id(node), cache)?.into()),
        NodeKind::COMPOUND => {
            Ok(build_compound(graph, CompoundId::from_node_id(node), cache)?.into())
        }
        NodeKind::COMP_SOLID => {
            Ok(build_comp_solid(graph, CompSolidId::from_node_id(node), cache)?.into())
        }
        _ => Ok(Shape::default()),
    }
}

/// The record of an active representation: valid, in range and not removed (the
/// `IsValid() && IsValid(Nb*()) && !IsRemoved()` guard of every representation read).
pub(super) fn active_record<T: RepTag>(storage: &Storage, rep: TypedId<T>) -> Option<&T::Record> {
    let active = rep.is_valid_for(storage.nb::<T>()) && !storage.is_removed(rep);
    active.then(|| storage.record(rep))
}

/// `replaySupplementAttachments(theSupplement, theOwner, theOwnerShape)`: adds the
/// attached shapes of the owner to its reconstructed shape.
pub(super) fn replay_supplement(
    graph: &BRepGraph,
    owner: NodeId,
    owner_shape: &mut Shape,
) -> Result<(), Failure> {
    let layer = graph.topo_supplement();
    let Some(supplement) = layer.get() else {
        return Ok(());
    };
    if owner_shape.is_null() {
        return Ok(());
    }
    for attached in supplement.attached_shapes(owner) {
        Builder.add(owner_shape, &attached)?;
    }
    Ok(())
}

// The Vertex case: a new vertex at the point with the tolerance.
fn make_vertex(graph: &BRepGraph, vertex: VertexId) -> Result<Vertex, Failure> {
    let def = graph.inc_storage().vertex(vertex);
    let mut result = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut result, def.point, def.tolerance)?;
    Ok(result)
}

// The Wire case: the active coedges' edges in their orientation, closed when the
// coedges chain.
fn build_wire(graph: &BRepGraph, wire: WireId, cache: &mut Cache) -> Result<Wire, Failure> {
    let storage = graph.inc_storage();
    let mut result = Wire::default();
    Builder.make_wire(&mut result);
    for &co_edge_id in &storage.wire_relations(wire).co_edge_ids {
        if !co_edge_id.is_valid_for(storage.nb_co_edges()) || storage.is_removed(co_edge_id) {
            continue;
        }
        let co_edge = storage.co_edge(co_edge_id);
        if !co_edge.child_edge_id.is_valid_for(storage.nb_edges()) {
            continue;
        }
        add_oriented(
            &mut result,
            node_with_cache(graph, co_edge.child_edge_id.into(), cache)?,
            co_edge.orientation,
        )?;
    }
    if cache_derived_state::compute_wire_is_closed(graph, wire) {
        result.set_closed(true)?;
    }
    Ok(result)
}

// Adds a non-null child in the orientation (the common tail of the container cases).
fn add_oriented(
    owner: &mut Shape,
    mut child: Shape,
    orientation: crate::brep_graph_inc::ParityOrientation,
) -> Result<(), Failure> {
    if child.is_null() {
        return Ok(());
    }
    child.set_orientation(orientation.orientation());
    Builder.add(owner, &child)
}

// The Shell case: the active face references' faces, closed when every edge is shared.
fn build_shell(graph: &BRepGraph, shell: ShellId, cache: &mut Cache) -> Result<Shell, Failure> {
    let storage = graph.inc_storage();
    let mut result = Shell::default();
    Builder.make_shell(&mut result);
    for &ref_id in &storage.shell_relations(shell).face_ref_ids {
        if storage.is_removed(ref_id) {
            continue;
        }
        let reference = storage.face_ref(ref_id);
        if !reference.child_face_id.is_valid_for(storage.nb_faces()) {
            continue;
        }
        add_oriented(
            &mut result,
            face_with_cache(graph, reference.child_face_id, cache)?,
            reference.orientation,
        )?;
    }
    if cache_derived_state::compute_shell_is_closed(graph, shell) {
        result.set_closed(true)?;
    }
    Ok(result)
}

// The Solid case: the active shell references' shells.
fn build_solid(graph: &BRepGraph, solid: SolidId, cache: &mut Cache) -> Result<Solid, Failure> {
    let storage = graph.inc_storage();
    let mut result = Solid::default();
    Builder.make_solid(&mut result);
    for &ref_id in &storage.solid_relations(solid).shell_ref_ids {
        if storage.is_removed(ref_id) {
            continue;
        }
        let reference = storage.shell_ref(ref_id);
        if !reference.child_shell_id.is_valid_for(storage.nb_shells()) {
            continue;
        }
        add_oriented(
            &mut result,
            node_with_cache(graph, reference.child_shell_id.into(), cache)?,
            reference.orientation,
        )?;
    }
    Ok(result)
}

// The Compound case: the active child references' shapes, with their local location
// when it is not the identity.
fn build_compound(
    graph: &BRepGraph,
    compound: CompoundId,
    cache: &mut Cache,
) -> Result<Compound, Failure> {
    let storage = graph.inc_storage();
    let mut result = Compound::default();
    Builder.make_compound(&mut result);
    for &ref_id in &storage.compound_relations(compound).child_ref_ids {
        if storage.is_removed(ref_id) {
            continue;
        }
        let reference = storage.child_ref(ref_id);
        if !reference.child_node_id.is_valid() {
            continue;
        }
        let mut child = node_with_cache(graph, reference.child_node_id, cache)?;
        if child.is_null() {
            continue;
        }
        child.set_orientation(reference.orientation.orientation());
        if !reference.local_location.is_identity() {
            child.set_location(&reference.local_location, false)?;
        }
        Builder.add(&mut result, &child)?;
    }
    Ok(result)
}

// The CompSolid case: the active solid references' solids.
fn build_comp_solid(
    graph: &BRepGraph,
    comp_solid: CompSolidId,
    cache: &mut Cache,
) -> Result<CompSolid, Failure> {
    let storage = graph.inc_storage();
    let mut result = CompSolid::default();
    Builder.make_comp_solid(&mut result);
    for &ref_id in &storage.comp_solid_relations(comp_solid).solid_ref_ids {
        if storage.is_removed(ref_id) {
            continue;
        }
        let reference = storage.solid_ref(ref_id);
        if !reference.child_solid_id.is_valid_for(storage.nb_solids()) {
            continue;
        }
        add_oriented(
            &mut result,
            node_with_cache(graph, reference.child_solid_id.into(), cache)?,
            reference.orientation,
        )?;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_grows_and_ignores_null_shapes() {
        let mut cache = Cache::new();
        let node = NodeId::new(NodeKind::VERTEX, 4);
        cache.bind(node, &Shape::default());
        assert!(!cache.is_bound(node));
        let mut vertex = Vertex::default();
        BRepBuilder::default()
            .make_vertex_at(&mut vertex, Default::default(), 1e-7)
            .unwrap();
        cache.bind(node, &vertex);
        assert!(cache.is_bound(node));
        assert!(!cache.is_bound(NodeId::new(NodeKind::VERTEX, 3)));
        cache.bind(NodeId::new(NodeKind(42), 0), &vertex);
        assert!(cache.seek(NodeId::new(NodeKind(42), 0)).is_none());
    }
}
