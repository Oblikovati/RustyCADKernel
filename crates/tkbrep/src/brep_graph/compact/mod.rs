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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_Compact.hxx and
// BRepGraph_Compact.cxx (`Perform`) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_Compact`: rebuilds a graph without its removed definitions, renumbering
//! the survivors densely and migrating the layers, caches and shape bindings.
//!
//! The native class has static members only: [`perform`] and [`perform_with`]. The
//! rebuild goes through a fresh graph's editor, bottom up, keeping the old generations
//! and UIDs; a rebuilt graph that fails the mutation boundary or
//! `BRepGraph_Validate` leaves the source untouched (docs/topology-design.md section
//! 11.12). C++ exceptions are [`Failure`] errors.
mod assembly;
mod co_edges;
mod containers;
mod maps;
mod swap;
mod topology;

use crate::brep_graph::layer_history::LayerHistory;
use crate::brep_graph::validate;
use crate::brep_graph::*;
use maps::{NodeMaps, RefMaps, bind_active_index_map, count_active};
use rustycad_tkernel::standard::Failure;
use std::collections::HashSet;

/// `BRepGraph_Compact::Options::CachePolicy`: the runtime cache migration policy.
///
/// ```
/// use rustycad_tkbrep::brep_graph::compact::CachePolicy;
/// assert_eq!(CachePolicy::default(), CachePolicy::Drop);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CachePolicy {
    /// `Drop`: keep the registered cache services but clear their transient entries.
    #[default]
    Drop,
    /// `CopyFresh`: copy the fresh, remappable entries into the compacted graph.
    CopyFresh,
}

/// `BRepGraph_Compact::Options`.
///
/// ```
/// use rustycad_tkbrep::brep_graph::compact::{CachePolicy, Options};
/// let options = Options::default();
/// assert!(options.history_mode && options.cache_mode == CachePolicy::Drop);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    /// `HistoryMode`: record the index remapping in the history layer.
    pub history_mode: bool,
    /// `CacheMode`: the runtime cache migration policy.
    pub cache_mode: CachePolicy,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            history_mode: true,
            cache_mode: CachePolicy::Drop,
        }
    }
}

/// `BRepGraph_Compact::Result`: the counters of a compaction.
///
/// ```
/// use rustycad_tkbrep::brep_graph::compact::CompactResult;
/// assert_eq!(CompactResult::default().nb_nodes_after, 0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompactResult {
    /// `NbRemovedVertices`.
    pub nb_removed_vertices: u32,
    /// `NbRemovedEdges`.
    pub nb_removed_edges: u32,
    /// `NbRemovedWires`.
    pub nb_removed_wires: u32,
    /// `NbRemovedFaces`.
    pub nb_removed_faces: u32,
    /// `NbRemovedShells`.
    pub nb_removed_shells: u32,
    /// `NbRemovedSolids`.
    pub nb_removed_solids: u32,
    /// `NbRemovedCompounds`.
    pub nb_removed_compounds: u32,
    /// `NbRemovedCompSolids`.
    pub nb_removed_comp_solids: u32,
    /// `NbRemovedSurfaces` (never written natively).
    pub nb_removed_surfaces: u32,
    /// `NbRemovedCurves` (never written natively).
    pub nb_removed_curves: u32,
    /// `NbNodesBefore`.
    pub nb_nodes_before: u32,
    /// `NbNodesAfter`.
    pub nb_nodes_after: u32,
    /// `NbUnmappedActiveDefs`: active definitions no remap holds (orphans and
    /// drop-outs), counted only with root products.
    pub nb_unmapped_active_defs: u32,
}

impl CompactResult {
    fn nb_removed(&self) -> u32 {
        self.nb_removed_vertices
            + self.nb_removed_edges
            + self.nb_removed_wires
            + self.nb_removed_faces
            + self.nb_removed_shells
            + self.nb_removed_solids
            + self.nb_removed_compounds
            + self.nb_removed_comp_solids
    }
}

/// `Perform(theGraph)`: [`perform_with`] and the default options.
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// let result = rustycad_tkbrep::brep_graph::compact::perform(&mut BRepGraph::new()).unwrap();
/// assert_eq!(result.nb_nodes_before, 0);
/// ```
pub fn perform(graph: &mut BRepGraph) -> Result<CompactResult, Failure> {
    perform_with(graph, &Options::default())
}

/// `Perform(theGraph, theOptions)`: an empty graph, or one without removed definitions,
/// is left alone. Otherwise the active definitions (those reachable from the root
/// products when there are some) are rebuilt densely into a fresh graph; a valid
/// rebuild replaces the source, whose layers migrate through the item remap. The
/// history's enabled state is restored on every return (not after a raise, as
/// natively).
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, VertexId};
/// use rustycad_tkmath::gp::Point3;
/// let mut graph = BRepGraph::new();
/// graph.editor().vertices().add(Point3::new(0.0, 0.0, 0.0), 1e-7);
/// graph.editor().vertices().add(Point3::new(0.0, 0.0, 0.0), 1e-7);
/// graph.editor().generic().remove_node(VertexId::new(0).into()).unwrap();
/// let options = rustycad_tkbrep::brep_graph::compact::Options::default();
/// let result = rustycad_tkbrep::brep_graph::compact::perform_with(&mut graph, &options).unwrap();
/// assert_eq!((result.nb_removed_vertices, result.nb_nodes_after), (1, 1));
/// ```
pub fn perform_with(graph: &mut BRepGraph, options: &Options) -> Result<CompactResult, Failure> {
    let mut result = CompactResult::default();
    if graph.is_empty() {
        return Ok(result);
    }
    count_removed(graph, &mut result);
    result.nb_nodes_before = graph.topo().generic().nb_nodes();
    if result.nb_removed() == 0 {
        result.nb_nodes_after = result.nb_nodes_before;
        return Ok(result);
    }
    let reachable = if graph.root_product_ids().is_empty() {
        None
    } else {
        Some(maps::reachable_from_root_products(graph)?)
    };
    let (mut nodes, mut refs) = (NodeMaps::new(graph), RefMaps::new(graph));
    bind_topology_maps(graph, &mut nodes, reachable.as_ref());
    if reachable.is_some() {
        result.nb_unmapped_active_defs = count_active_topology(graph) - nodes.nb_mapped_topology() as u32;
    }
    let history = graph.layer_registry().ensure::<LayerHistory>()?;
    let was_enabled = history.is_enabled();
    history.set_enabled(options.history_mode);
    let fresh = rebuild(graph, &mut nodes, &mut refs)?;
    if !is_valid_rebuild(&fresh) {
        result.nb_nodes_after = result.nb_nodes_before;
        history.set_enabled(was_enabled);
        return Ok(result);
    }
    let mut fresh = fresh;
    assembly::rebuild_products(graph, &mut fresh, &mut nodes, &mut refs, reachable.as_ref())?;
    assembly::restore_product_links(graph, &mut fresh, &mut nodes, &mut refs, reachable.as_ref())?;
    result.nb_nodes_after = fresh.topo().generic().nb_nodes();
    swap::finish_uids(&mut fresh, graph);
    swap::install(graph, fresh, &nodes, &refs, options)?;
    graph.editor().commit_mutation();
    // Re-acquired: the layer migration replaced the history instance.
    graph.layer_registry().ensure::<LayerHistory>()?.set_enabled(was_enabled);
    Ok(result)
}

// The removed definitions of each topology kind (slots minus active definitions).
fn count_removed(graph: &BRepGraph, result: &mut CompactResult) {
    let removed = |nb: u32, active: u32| nb - active;
    let topo = graph.topo();
    result.nb_removed_vertices = removed(topo.vertices().nb(), count_active::<VertexTag>(graph));
    result.nb_removed_edges = removed(topo.edges().nb(), count_active::<EdgeTag>(graph));
    result.nb_removed_wires = removed(topo.wires().nb(), count_active::<WireTag>(graph));
    result.nb_removed_faces = removed(topo.faces().nb(), count_active::<FaceTag>(graph));
    result.nb_removed_shells = removed(topo.shells().nb(), count_active::<ShellTag>(graph));
    result.nb_removed_solids = removed(topo.solids().nb(), count_active::<SolidTag>(graph));
    result.nb_removed_compounds = removed(topo.compounds().nb(), count_active::<CompoundTag>(graph));
    result.nb_removed_comp_solids =
        removed(topo.comp_solids().nb(), count_active::<CompSolidTag>(graph));
}

// The active topology definitions of every kind.
fn count_active_topology(graph: &BRepGraph) -> u32 {
    count_active::<VertexTag>(graph)
        + count_active::<EdgeTag>(graph)
        + count_active::<WireTag>(graph)
        + count_active::<FaceTag>(graph)
        + count_active::<ShellTag>(graph)
        + count_active::<SolidTag>(graph)
        + count_active::<CompoundTag>(graph)
        + count_active::<CompSolidTag>(graph)
}

// The dense old-to-new numbering of every topology kind but the coedges.
fn bind_topology_maps(graph: &BRepGraph, nodes: &mut NodeMaps, reachable: Option<&HashSet<NodeId>>) {
    bind_active_index_map(graph, &mut nodes.vertices, reachable);
    bind_active_index_map(graph, &mut nodes.edges, reachable);
    bind_active_index_map(graph, &mut nodes.wires, reachable);
    bind_active_index_map(graph, &mut nodes.faces, reachable);
    bind_active_index_map(graph, &mut nodes.shells, reachable);
    bind_active_index_map(graph, &mut nodes.solids, reachable);
    bind_active_index_map(graph, &mut nodes.compounds, reachable);
    bind_active_index_map(graph, &mut nodes.comp_solids, reachable);
}

// The fresh graph (with the source's generation and GUID) rebuilt bottom up, its
// derived relations rebuilt from the final incidence.
fn rebuild(graph: &BRepGraph, nodes: &mut NodeMaps, refs: &mut RefMaps) -> Result<BRepGraph, Failure> {
    let mut fresh = BRepGraph::new();
    let storage = graph.inc_storage();
    fresh.inc_storage().set_generation(storage.generation());
    fresh.inc_storage_mut().set_graph_guid(*storage.graph_guid());
    topology::rebuild_vertices(graph, &mut fresh, nodes)?;
    topology::rebuild_edges(graph, &mut fresh, nodes, refs)?;
    topology::rebuild_wires(graph, &mut fresh, nodes)?;
    topology::rebuild_faces(graph, &mut fresh, nodes, refs)?;
    co_edges::map_free_co_edges(graph, &fresh, nodes);
    co_edges::transfer_co_edge_attributes(graph, &mut fresh, nodes)?;
    containers::rebuild_shells(graph, &mut fresh, nodes, refs)?;
    containers::rebuild_solids(graph, &mut fresh, nodes, refs)?;
    containers::rebuild_compounds(graph, &mut fresh, nodes, refs)?;
    containers::rebuild_comp_solids(graph, &mut fresh, nodes, refs)?;
    fresh.inc_storage_mut().rebuild_derived_relations();
    Ok(fresh)
}

// A rebuilt graph that fails the mutation boundary or the lightweight validation keeps
// the source graph unchanged.
fn is_valid_rebuild(fresh: &BRepGraph) -> bool {
    fresh.editor_state().validate_mutation_boundary(None) && validate::perform(fresh).is_valid()
}
