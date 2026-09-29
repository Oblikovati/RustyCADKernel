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
// Rust translation of BRepGraph_LayerDeferred::CopyTo of
// ModelingData/TKBRep/BRepGraph/BRepGraph_LayerDeferred.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Copying the deferred entries into another graph.
use super::{DeferredEntry, LayerDeferred};
use crate::brep_graph::ItemId;
use crate::brep_graph::copy_remap::CopyRemap;
use rustycad_tkernel::standard::Failure;

impl LayerDeferred {
    /// `CopyTo(theCopy)`: the entries (in map order) of mapped items are registered in
    /// the target graph's deferred layer (`Ensure`d) through the trusted direct path, in
    /// one bulk registration.
    pub(super) fn copy_entries_to(&self, copy: &CopyRemap<'_>) -> Result<(), Failure> {
        let entries: Vec<(ItemId, DeferredEntry)> = {
            let records = self.records.read();
            let items = records.entries.items();
            items.map(|(item, entry)| (*item, entry.clone())).collect()
        };
        if entries.is_empty() {
            return Ok(());
        }
        let graph = copy.target_graph_const();
        let target = graph.layer_registry().ensure::<LayerDeferred>()?;
        let Some(target) = target.get() else {
            return Ok(());
        };
        target.reserve_deferred_items(graph, entries.len())?;
        target.begin_bulk_registration();
        for (item, entry) in &entries {
            let target_item = copy.target_item(*item);
            if !target_item.is_valid() {
                continue;
            }
            let representations = entry.representations.as_slice();
            let (provider, key) = (&entry.provider, &entry.source_key);
            target.register_deferred_representations_direct(
                graph,
                target_item,
                provider,
                key,
                representations,
            )?;
        }
        target.end_bulk_registration(graph)
    }
}
