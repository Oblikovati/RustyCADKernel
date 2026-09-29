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
// Rust translation of ModelingData/TKBRep/BRepGraph at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! BRepGraph: the graph facade over the [`brep_graph_inc`](crate::brep_graph_inc) backend.
//!
//! A [`BRepGraph`] owns its storage; the read views ([`BRepGraph::topo`],
//! [`BRepGraph::refs`], [`BRepGraph::uids`], [`BRepGraph::shapes`]) borrow it, and
//! [`BRepGraph::shapes_mut`] ingests shapes. The ids are values ([`NodeId`], [`RefId`],
//! [`RepId`], their typed aliases, [`ItemId`] and the UIDs). docs/topology-design.md
//! section 11 fixes the mapping and names the modules the second phase of the port adds.
pub mod cache;
pub mod cache_derived_state;
pub mod cache_mesh;
pub mod cache_registry;
pub mod child_explorer;
mod child_refs;
pub mod copy;
pub mod copy_remap;
pub mod deduplicate;
mod data;
pub mod deferred_scope;
pub mod defs_iterator;
pub mod editor;
mod explorer_kinds;
mod graph;
pub mod hooks;
mod id_hash_keys;
mod id_walk;
pub(crate) mod ids;
mod invalidate;
pub mod iterator;
pub mod layer;
pub mod layer_deferred;
pub mod layer_history;
pub mod layer_lock;
pub mod layer_parametric;
pub mod layer_registry;
pub mod layer_topo_supplement;
mod marks;
pub mod mesh_view;
pub mod mut_guard;
pub mod parallel_policy;
mod parent_cursor;
pub mod parent_explorer;
pub mod parent_traits;
pub mod refs_iterator;
pub mod refs_view;
pub mod registry_iterator;
pub mod related_iterator;
pub mod reverse_iterator;
pub mod shapes_view;
pub mod supplement_editor;
pub mod supplement_iterator;
pub mod tool;
pub mod topo_view;
pub mod uids_view;
mod usage_path;
pub mod validate;
mod version_stamp;

pub use data::Data;
pub use graph::BRepGraph;
pub use ids::*;
pub use registry_iterator::{CacheIterator, LayerIterator};
pub use usage_path::{UsagePath, UsageStep};
pub use version_stamp::{StampDomain, VersionStamp};

pub use crate::brep_graph_inc::{NodeTag, RefTag, RepTag, StoredTag};
