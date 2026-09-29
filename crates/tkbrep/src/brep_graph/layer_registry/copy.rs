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
// Rust translation of BRepGraph_LayerRegistry::CopyLayersTo (both overloads) of
// ModelingData/TKBRep/BRepGraph/BRepGraph_LayerRegistry.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Copying every registered layer into another graph, or migrating the layers of a
//! compacted graph.
//!
//! The native members run on the source graph's registry with a `BRepGraph&` target,
//! and the compaction passes the source graph itself as the target. A Rust registry
//! cannot reach its graph, so the members are associated functions that take the
//! source graph (whose registry they use) and the target, and
//! [`LayerRegistry::copy_layers_in_place`] is the aliasing call (docs/topology-design.md
//! section 11.9).
use super::LayerRegistry;
use crate::brep_graph::BRepGraph;
use crate::brep_graph::copy_remap::{CopyRemap, ItemMap, MappingKind, Mode};
use crate::brep_graph::layer::{BRepGraphLayer, detach_context};
use rustycad_tkernel::standard::{Failure, Handle};
use std::sync::atomic::Ordering;

impl LayerRegistry {
    /// `CopyLayersTo(theTargetGraph, theItemRemap, theMode)` with a distinct target:
    /// `CopyTo` on every layer of `source` in slot order. In `Mode::Compact` the source
    /// registry is emptied first (every layer detached) and the old layers recreate
    /// their data in the target through `Ensure`.
    ///
    /// ```
    /// use rustycad_tkbrep::brep_graph::copy_remap::{ItemMap, Mode};
    /// use rustycad_tkbrep::brep_graph::layer_registry::LayerRegistry;
    /// use rustycad_tkbrep::brep_graph::BRepGraph;
    /// let (source, mut target) = (BRepGraph::new(), BRepGraph::new());
    /// LayerRegistry::copy_layers_to(&source, &mut target, &ItemMap::default(), Mode::Copy).unwrap();
    /// assert_eq!(target.layer_registry().nb_layers(), 1);
    /// ```
    pub fn copy_layers_to(
        source: &BRepGraph,
        target: &mut BRepGraph,
        items: &ItemMap,
        mode: Mode,
    ) -> Result<(), Failure> {
        let registry = source.layer_registry();
        if mode == Mode::Compact {
            let old_layers = registry.take_layers();
            let copy = CopyRemap::new(source, target, items, mode);
            return copy_each(&old_layers, &copy);
        }
        let copy = CopyRemap::new(source, target, items, mode);
        registry.copy_each_slot(&copy)
    }

    /// `CopyLayersTo(theTargetGraph, theMappingKind, theMode)`: `CopyTo` on every layer of
    /// `source` in slot order, with the identity (or an empty explicit) mapping.
    pub fn copy_layers_to_with_mapping(
        source: &BRepGraph,
        target: &mut BRepGraph,
        mapping_kind: MappingKind,
        mode: Mode,
    ) -> Result<(), Failure> {
        let copy = CopyRemap::with_mapping(source, target, mapping_kind, mode);
        source.layer_registry().copy_each_slot(&copy)
    }

    /// `CopyLayersTo(theGraph, theItemRemap, theMode)` called on the registry of
    /// `theGraph` itself (the compaction): in `Mode::Compact` the layers are detached
    /// and unregistered, then migrate into the same graph through the item map; in
    /// `Mode::Copy` each layer copies onto its own graph.
    pub fn copy_layers_in_place(
        graph: &mut BRepGraph,
        items: &ItemMap,
        mode: Mode,
    ) -> Result<(), Failure> {
        if mode == Mode::Compact {
            let old_layers = graph.layer_registry().take_layers();
            let copy = CopyRemap::in_place(graph, Some(items), MappingKind::Explicit, mode);
            return copy_each(&old_layers, &copy);
        }
        let copy = CopyRemap::in_place(graph, Some(items), MappingKind::Explicit, mode);
        copy.source_graph().layer_registry().copy_each_slot(&copy)
    }

    // The copy loop: one slot read per layer until the first empty slot.
    fn copy_each_slot(&self, copy: &CopyRemap<'_>) -> Result<(), Failure> {
        for slot in 0_u32.. {
            let layer = self.layer(slot);
            let Some(layer) = layer.get() else {
                return Ok(());
            };
            layer.copy_to(copy)?;
        }
        Ok(())
    }

    // The compaction's reset: the non-null layers are collected, every layer detached,
    // the slots, the GUID map and the masks cleared.
    fn take_layers(&self) -> Vec<Handle<dyn BRepGraphLayer>> {
        let mut slots = self.slots.write();
        let old_layers: Vec<_> = slots
            .layers
            .iter()
            .filter(|layer| layer.is_some())
            .cloned()
            .collect();
        for layer in slots.layers.iter().filter_map(Handle::get) {
            detach_context(layer);
        }
        slots.layers.clear();
        slots.guid_to_slot.clear();
        self.subscribed_kinds.store(0, Ordering::Relaxed);
        self.subscribed_ref_kinds.store(0, Ordering::Relaxed);
        old_layers
    }
}

// `CopyTo` on each collected layer, in order.
fn copy_each(layers: &[Handle<dyn BRepGraphLayer>], copy: &CopyRemap<'_>) -> Result<(), Failure> {
    for layer in layers.iter().filter_map(Handle::get) {
        layer.copy_to(copy)?;
    }
    Ok(())
}
