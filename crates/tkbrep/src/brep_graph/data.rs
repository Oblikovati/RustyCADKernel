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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_Data.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_Data`: what a graph owns.
use super::cache_registry::CacheRegistry;
use super::layer_registry::LayerRegistry;
use crate::brep_graph_inc::Storage;
use std::sync::atomic::{AtomicU64, Ordering};

/// `BRepGraph_Data`: the incidence storage (the only source of truth) and the layer and
/// cache registries. The native view members are not stored: the Rust views borrow the
/// graph when asked for (docs/topology-design.md sections 11.3 and 11.9).
#[derive(Debug)]
pub struct Data {
    /// `myIncStorage`.
    pub(crate) inc_storage: Storage,
    /// `myLayerRegistry`, attached to this graph's token.
    pub(crate) layers: LayerRegistry,
    /// `myCacheRegistry`, attached to this graph's token.
    pub(crate) caches: CacheRegistry,
}

impl Default for Data {
    // `Attach(this)` of both registries: a fresh attachment token names this graph.
    fn default() -> Self {
        let token = next_attachment_token();
        Self {
            inc_storage: Storage::default(),
            layers: LayerRegistry::for_graph(token),
            caches: CacheRegistry::for_graph(token),
        }
    }
}

// The process-unique, non-zero identity a graph gives its registries (the native
// `BRepGraph*` a layer or cache stores), docs/topology-design.md 11.9.
fn next_attachment_token() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}
