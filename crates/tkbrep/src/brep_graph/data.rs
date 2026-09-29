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
use super::hooks::{CacheHooks, LayerHooks};
use crate::brep_graph_inc::Storage;

/// `BRepGraph_Data`: the incidence storage (the only source of truth) and the attachment
/// slots of the layer and cache registries. The native view members are not stored:
/// the Rust views borrow the graph when asked for (docs/topology-design.md section 11.3).
#[derive(Default)]
pub struct Data {
    /// `myIncStorage`.
    pub(crate) inc_storage: Storage,
    /// `myLayerRegistry`, attached by the layers area.
    pub(crate) layers: Option<Box<dyn LayerHooks>>,
    /// `myCacheRegistry`, attached by the caches area.
    pub(crate) caches: Option<Box<dyn CacheHooks>>,
}

impl std::fmt::Debug for Data {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Data")
            .field("inc_storage", &self.inc_storage)
            .field("layers", &self.layers.is_some())
            .field("caches", &self.caches.is_some())
            .finish()
    }
}
