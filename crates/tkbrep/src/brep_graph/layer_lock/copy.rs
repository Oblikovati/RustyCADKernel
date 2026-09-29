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
// Rust translation of BRepGraph_LayerLock::CopyTo of
// ModelingData/TKBRep/BRepGraph/BRepGraph_LayerLock.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Copying the root owners into another graph.
use super::LayerLock;
use crate::brep_graph::ItemId;
use crate::brep_graph::copy_remap::CopyRemap;
use rustycad_tkernel::standard::{Failure, Guid};

impl LayerLock {
    /// `CopyTo(theCopy)`: the root entries mapped to target items are registered in the
    /// target graph's lock layer (`Ensure`d), node roots first, in map order, with one
    /// revision update at the end when anything was stored.
    pub(super) fn copy_roots_to(&self, copy: &CopyRemap<'_>) -> Result<(), Failure> {
        let (nodes, refs) = self.root_entries();
        if nodes.is_empty() && refs.is_empty() {
            return Ok(());
        }
        let target = copy
            .target_graph_const()
            .layer_registry()
            .ensure::<LayerLock>()?;
        let Some(target) = target.get() else {
            return Ok(());
        };
        target.reserve_owners(nodes.len() + refs.len());
        let mut has_copied = false;
        for (item, owner) in nodes.iter().chain(refs.iter()) {
            has_copied |= copy_root(copy, target, *item, owner)?;
        }
        if has_copied {
            target.touch_owners();
        }
        Ok(())
    }
}

// One root entry of `CopyTo`: skipped when the source item has no target item.
fn copy_root(
    copy: &CopyRemap<'_>,
    target: &LayerLock,
    item: ItemId,
    owner: &Guid,
) -> Result<bool, Failure> {
    let target_item = copy.target_item(item);
    if !target_item.is_valid() {
        return Ok(false);
    }
    target.set_owner_with_revision(copy.target_graph_const(), target_item, owner, false)
}
