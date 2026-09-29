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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_Copy.hxx and
// BRepGraph_Copy.cxx (`Perform`, `CopyNode`) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_Copy`: deep copy of a whole graph or of one node's sub-graph.
//!
//! The native class has static members only; they are the free functions of this module.
//! A native call whose source and target are the same graph (the self-copy) cannot be
//! written with a `&BRepGraph` and a `&mut BRepGraph`, so it has its own entry point:
//! [`perform_self`] (the identity no-op) and [`copy_node_in_place`] (the sub-graph
//! duplicated inside one graph). C++ exceptions are [`Failure`] errors.
mod assembly;
mod containers;
mod context;
mod identity;
mod meshes;
mod pcurves;
mod policy;
mod topology;

pub use policy::{CachePolicy, GeomPolicy, MeshPolicy};

use crate::brep_graph::cache_registry::CacheRegistry;
use crate::brep_graph::copy_remap::Mode;
use crate::brep_graph::id_walk::full_ids;
use crate::brep_graph::layer_registry::LayerRegistry;
use crate::brep_graph::*;
use assembly::{append_root_products, ensure_occurrence, ensure_product};
use containers::{ensure_comp_solid, ensure_compound, ensure_node, ensure_shell, ensure_solid};
use context::{CopyContext, CopyGraphs};
use rustycad_tkernel::standard::Failure;
use topology::{ensure_edge, ensure_face, ensure_vertex, ensure_wire};

/// `Perform(theSourceGraph, theTargetGraph, theGeomPolicy, theMeshPolicy,
/// theCachePolicy)`: copies the whole source graph into the target. An empty source
/// fails (`false`). An empty target takes the identity fast path (the result ids equal
/// the source ids); a non-empty target receives every source entity appended, bottom
/// up, through the editor, with the root products rebuilt.
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// use rustycad_tkbrep::brep_graph::copy::{self, CachePolicy, GeomPolicy, MeshPolicy};
/// use rustycad_tkmath::gp::Point3;
/// let mut source = BRepGraph::new();
/// source.editor().vertices().add(Point3::new(1.0, 2.0, 3.0), 1e-7);
/// let mut target = BRepGraph::new();
/// let done = copy::perform(&source, &mut target, GeomPolicy::Copy, MeshPolicy::Copy,
///     CachePolicy::Drop).unwrap();
/// assert!(done && target.topo().vertices().nb() == 1);
/// ```
pub fn perform(
    source: &BRepGraph,
    target: &mut BRepGraph,
    geom: GeomPolicy,
    mesh: MeshPolicy,
    cache: CachePolicy,
) -> Result<bool, Failure> {
    if source.is_empty() {
        return Ok(false);
    }
    if target.is_empty() {
        return identity::copy_full_graph_identity(source, target, geom, mesh, cache);
    }
    let graphs = CopyGraphs::Distinct { source, target };
    let mut context = CopyContext::new(graphs, geom, mesh);
    copy_every_entity(&mut context)?;
    append_root_products(&mut context);
    finish_copy(context, cache)?;
    Ok(true)
}

/// `Perform(theGraph, theGraph, ...)`: the self-copy is the identity no-op and succeeds.
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// assert!(rustycad_tkbrep::brep_graph::copy::perform_self(&mut BRepGraph::new()));
/// ```
pub fn perform_self(_graph: &mut BRepGraph) -> bool {
    true
}

/// `CopyNode(theSourceGraph, theTargetGraph, theNodeId, ...)`: copies one node of any
/// kind with everything it references into another graph (entities the copy already
/// made are reused); the mapped root, or the invalid id for an empty source or an
/// unsupported node.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, NodeId};
/// use rustycad_tkbrep::brep_graph::copy::{self, CachePolicy, GeomPolicy, MeshPolicy};
/// use rustycad_tkmath::gp::Point3;
/// let mut source = BRepGraph::new();
/// let vertex = source.editor().vertices().add(Point3::new(0.0, 0.0, 0.0), 1e-7);
/// let mut target = BRepGraph::new();
/// let copied = copy::copy_node(&source, &mut target, vertex.into(), GeomPolicy::Copy,
///     MeshPolicy::Copy, CachePolicy::Drop).unwrap();
/// assert_eq!(copied, NodeId::from(vertex));
/// ```
pub fn copy_node(
    source: &BRepGraph,
    target: &mut BRepGraph,
    node: NodeId,
    geom: GeomPolicy,
    mesh: MeshPolicy,
    cache: CachePolicy,
) -> Result<NodeId, Failure> {
    if source.is_empty() {
        return Ok(NodeId::default());
    }
    copy_node_with(
        CopyGraphs::Distinct { source, target },
        node,
        geom,
        mesh,
        cache,
    )
}

/// `CopyNode(theGraph, theGraph, theNodeId, ...)`: the self-copy duplicates the node's
/// sub-graph with new ids inside the same graph; the caches are never copied.
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// use rustycad_tkbrep::brep_graph::copy::{self, CachePolicy, GeomPolicy, MeshPolicy};
/// use rustycad_tkmath::gp::Point3;
/// let mut graph = BRepGraph::new();
/// let vertex = graph.editor().vertices().add(Point3::new(0.0, 0.0, 0.0), 1e-7);
/// let copied = copy::copy_node_in_place(&mut graph, vertex.into(), GeomPolicy::Copy,
///     MeshPolicy::Copy, CachePolicy::Drop).unwrap();
/// assert_eq!(copied.index, 1);
/// ```
pub fn copy_node_in_place(
    graph: &mut BRepGraph,
    node: NodeId,
    geom: GeomPolicy,
    mesh: MeshPolicy,
    cache: CachePolicy,
) -> Result<NodeId, Failure> {
    if graph.is_empty() {
        return Ok(NodeId::default());
    }
    copy_node_with(CopyGraphs::Same(graph), node, geom, mesh, cache)
}

// The body of `CopyNode` after its empty-source check.
fn copy_node_with(
    graphs: CopyGraphs<'_>,
    node: NodeId,
    geom: GeomPolicy,
    mesh: MeshPolicy,
    cache: CachePolicy,
) -> Result<NodeId, Failure> {
    let mut context = CopyContext::new(graphs, geom, mesh);
    ensure_node(&mut context, node)?;
    append_root_products(&mut context);
    let mapped = context.mapped_node(node);
    finish_copy(context, cache)?;
    Ok(mapped)
}

// The bottom-up walk of `Perform` into a non-empty target: every source entity of every
// kind, removed ones included. The source is another graph, so its ids are collected
// kind by kind without changing what the native full iterators visit.
fn copy_every_entity(context: &mut CopyContext<'_>) -> Result<(), Failure> {
    ensure_all::<VertexTag, _>(context, ensure_vertex)?;
    ensure_all::<EdgeTag, _>(context, ensure_edge)?;
    ensure_all::<WireTag, _>(context, ensure_wire)?;
    ensure_all::<FaceTag, _>(context, ensure_face)?;
    ensure_all::<ShellTag, _>(context, ensure_shell)?;
    ensure_all::<SolidTag, _>(context, ensure_solid)?;
    ensure_all::<CompoundTag, _>(context, ensure_compound)?;
    ensure_all::<CompSolidTag, _>(context, ensure_comp_solid)?;
    ensure_all::<ProductTag, _>(context, ensure_product)?;
    ensure_all::<OccurrenceTag, _>(context, ensure_occurrence)
}

// One kind of the walk: `ensure` on every source id of the kind.
fn ensure_all<T: NodeTag, R>(
    context: &mut CopyContext<'_>,
    ensure: fn(&mut CopyContext<'_>, TypedId<T>) -> Result<R, Failure>,
) -> Result<(), Failure> {
    for id in full_ids::<T>(context.source()) {
        ensure(context, id)?;
    }
    Ok(())
}

// The tail of `Perform` and `CopyNode`: the UID reverse indexes marked dirty, the
// derived relations rebuilt (active counts kept), the layers copied through the item
// map, and the caches under `CopyFresh` (never for a self-copy).
fn finish_copy(context: CopyContext<'_>, cache: CachePolicy) -> Result<(), Failure> {
    let (graphs, items) = context.into_parts();
    match graphs {
        CopyGraphs::Distinct { source, target } => {
            prepare_target(target);
            LayerRegistry::copy_layers_to(source, target, &items, Mode::Copy)?;
            if cache == CachePolicy::CopyFresh {
                CacheRegistry::copy_fresh_caches_to(source, target, &items, Mode::Copy)?;
            }
            Ok(())
        }
        CopyGraphs::Same(graph) => {
            prepare_target(graph);
            LayerRegistry::copy_layers_in_place(graph, &items, Mode::Copy)
        }
    }
}

fn prepare_target(target: &mut BRepGraph) {
    let storage = target.inc_storage_mut();
    storage.mark_uid_reverse_indexes_dirty();
    storage.rebuild_derived_relations_preserving_active_counts();
}
