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
// Rust translation of the tail of `BRepGraph_Compact::Perform` (UID counters, registry
// save, swap, layer migration, shape bindings and the remap history record) of
// ModelingData/TKBRep/BRepGraph/BRepGraph_Compact.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Installing the rebuilt graph in place of the compacted one.
//!
//! Natively the source graph's registries are moved out, the rebuilt graph is moved
//! into the source object, and the saved registries are moved back and re-attached. A
//! Rust graph keeps its registries in place and swaps its storage with the rebuilt
//! graph's, replaying the registry callbacks of the native moves (docs/topology-design.md
//! section 11.12).
use super::maps::{NodeMaps, RefMaps, item_remap};
use super::{CachePolicy, Options};
use crate::brep_graph::cache_registry::CacheRegistry;
use crate::brep_graph::copy_remap::Mode;
use crate::brep_graph::layer_history::{HistoryKind, LayerHistory};
use crate::brep_graph::layer_registry::LayerRegistry;
use crate::brep_graph::*;
use crate::brep_graph_inc::{HasBaseDef, HasBaseRef, Storage};
use crate::topods::Shape;
use rustycad_tkernel::ncollection::{DataMap, DefaultHashPolicy};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::AsciiString;

// The largest UID counter of a node kind's slots (removed ones included).
fn set_node_counter<T: NodeTag>(storage: &Storage) {
    let max = (0..storage.nb::<T>())
        .map(|index| storage.def(TypedId::<T>::new(index)).base().uid)
        .max()
        .unwrap_or(0);
    storage.set_next_node_uid_counter(NodeKind(T::RAW_KIND), max + 1);
}

// The largest UID counter of a reference kind's slots.
fn set_ref_counter<T: RefTag>(storage: &Storage) {
    let max = (0..storage.nb::<T>())
        .map(|index| storage.entry(TypedId::<T>::new(index)).base().uid)
        .max()
        .unwrap_or(0);
    storage.set_next_ref_uid_counter(RefKind(T::RAW_KIND), max + 1);
}

/// The rebuilt graph's UID state: every per-kind counter set to the largest UID of its
/// slots plus one (the UIDs are the old inline ones), the old generation and GUID, and
/// the reverse indexes rebuilt from the inline UIDs.
pub(super) fn finish_uids(fresh: &mut BRepGraph, graph: &BRepGraph) {
    let storage = fresh.inc_storage_mut();
    set_node_counter::<VertexTag>(storage);
    set_node_counter::<EdgeTag>(storage);
    set_node_counter::<CoEdgeTag>(storage);
    set_node_counter::<WireTag>(storage);
    set_node_counter::<FaceTag>(storage);
    set_node_counter::<ShellTag>(storage);
    set_node_counter::<SolidTag>(storage);
    set_node_counter::<CompoundTag>(storage);
    set_node_counter::<CompSolidTag>(storage);
    set_node_counter::<ProductTag>(storage);
    set_node_counter::<OccurrenceTag>(storage);
    set_ref_counter::<ShellRefTag>(storage);
    set_ref_counter::<FaceRefTag>(storage);
    set_ref_counter::<WireRefTag>(storage);
    set_ref_counter::<VertexRefTag>(storage);
    set_ref_counter::<SolidRefTag>(storage);
    set_ref_counter::<ChildRefTag>(storage);
    set_ref_counter::<OccurrenceRefTag>(storage);
    // Preserve the graph generation so inline UIDs remain valid after compaction.
    storage.set_generation(graph.inc_storage().generation());
    storage.set_graph_guid(*graph.inc_storage().graph_guid());
    storage.rebuild_uid_reverse_indexes();
}

// The shape-to-node and node-to-original-shape bindings of the compacted storage.
struct Bindings {
    shapes: Vec<(Shape, NodeId)>,
    originals: Vec<(NodeId, Shape)>,
}

fn save_bindings(storage: &Storage) -> Bindings {
    let mut bindings = Bindings {
        shapes: Vec::new(),
        originals: Vec::new(),
    };
    storage.for_each_shape_binding(|shape, node| bindings.shapes.push((shape.clone(), node)));
    storage.for_each_original_binding(|node, shape| bindings.originals.push((node, shape.clone())));
    bindings
}

// The bindings of the surviving nodes, their ids remapped; dropped nodes lose theirs.
fn restore_bindings(storage: &Storage, bindings: &Bindings, remap: &DataMap<NodeId, NodeId, DefaultHashPolicy>) {
    for (shape, old) in &bindings.shapes {
        if let Some(new) = remap.seek(old) {
            storage.set_definition_shape_binding(shape, *new);
        }
    }
    for (old, shape) in &bindings.originals {
        if let Some(new) = remap.seek(old) {
            storage.bind_original(*new, shape);
        }
    }
}

/// Installs the rebuilt graph: the caches follow the policy (copied fresh into the
/// rebuilt graph and adopted, or kept and cleared), the storages swap, the layers are
/// re-attached and migrated through the item remap, the bindings are remapped and, in
/// history mode, one `Compact:Remap` batch records every node pair.
pub(super) fn install(
    graph: &mut BRepGraph,
    mut fresh: BRepGraph,
    nodes: &NodeMaps,
    refs: &RefMaps,
    options: &Options,
) -> Result<(), Failure> {
    let bindings = save_bindings(graph.inc_storage());
    let items = item_remap(nodes, refs);
    if options.cache_mode == CachePolicy::CopyFresh {
        CacheRegistry::copy_fresh_caches_to(graph, &mut fresh, &items, Mode::Compact)?;
        graph.cache_registry().clear();
    }
    std::mem::swap(graph.inc_storage_mut(), fresh.inc_storage_mut());
    if options.cache_mode == CachePolicy::CopyFresh {
        graph.cache_registry().adopt(fresh.cache_registry());
    }
    // The rebuilt graph's own layers (and, when dropped, its services) are detached.
    drop(fresh);
    graph.layer_registry().reattach();
    if options.cache_mode == CachePolicy::Drop {
        graph.cache_registry().clear_all();
    }
    LayerRegistry::copy_layers_in_place(graph, &items, Mode::Compact)?;
    let remap = node_remap(graph, nodes);
    restore_bindings(graph.inc_storage(), &bindings, &remap);
    if options.history_mode {
        record_remap(graph, &remap)?;
    }
    Ok(())
}

// `aRemapMap`: the node pairs bound kind by kind into a map sized with the new node
// count; its bucket order is the order of the history batch.
fn node_remap(graph: &BRepGraph, nodes: &NodeMaps) -> DataMap<NodeId, NodeId, DefaultHashPolicy> {
    let size = graph.topo().generic().nb_nodes() as usize;
    let mut remap = DataMap::with_hasher(DefaultHashPolicy, size);
    for (old, new) in nodes.pairs() {
        remap.bind(old, new);
    }
    remap
}

// The history layer after the migration (a new instance), enabled, records the pairs.
fn record_remap(graph: &BRepGraph, remap: &DataMap<NodeId, NodeId, DefaultHashPolicy>) -> Result<(), Failure> {
    let history = graph.layer_registry().ensure::<LayerHistory>()?;
    history.set_enabled(true);
    let (originals, replacements): (Vec<NodeId>, Vec<NodeId>) =
        remap.items().map(|(old, new)| (*old, *new)).unzip();
    if originals.is_empty() {
        return Ok(());
    }
    history.record_batch(
        graph,
        &AsciiString::new("Compact:Remap"),
        &originals,
        &replacements,
        &AsciiString::default(),
        HistoryKind::Modified,
    )
}
