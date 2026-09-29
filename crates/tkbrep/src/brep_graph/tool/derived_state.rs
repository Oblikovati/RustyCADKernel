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
// Rust translation of the answers of BRepGraph_CacheDerivedState::IsDegenerated,
// IsClosed, SameRange, SameParameter, GetWireIsClosed and IsShellClosed that
// BRepGraph_Tool reads through `derivedStateCache` (ModelingData/TKBRep/BRepGraph/
// BRepGraph_CacheDerivedState.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The derived-state reads of `BRepGraph_Tool`, routed through the graph's memoising
//! [`CacheDerivedState`] service (`derivedStateCache`: `CacheRegistry().Ensure`), so
//! a repeated read is answered from the cache while the entity's OwnGen is unchanged.
//!
//! The service's `ensure*Entry` refuses an entity out of range or removed (and
//! `BindOwnGen` refuses an inactive one), so each read answers false for those, even
//! where the static computation says true (an inactive coedge is same-range there).
//! The service registered by the graph is attached to it, so its `Graph()` never
//! raises here; when another class holds the service GUID (natively a null
//! dereference) the reads fall back to the uncached computations.
use crate::brep_graph::cache_derived_state::{
    CacheDerivedState, compute_edge_properties, compute_same_parameter, compute_same_range,
    compute_shell_is_closed, compute_wire_is_closed,
};
use crate::brep_graph::{BRepGraph, CoEdgeId, EdgeId, ShellId, WireId};
use rustycad_tkernel::standard::{Failure, Handle};

/// `derivedStateCache(theGraph)`: the graph's service, registered on first use; `None`
/// when it cannot be obtained (see the module notes).
pub(crate) fn derived_state_cache(graph: &BRepGraph) -> Option<Handle<CacheDerivedState>> {
    let cache = graph.cache_registry().ensure::<CacheDerivedState>().ok()?;
    cache.is_some().then_some(cache)
}

/// `IsDegenerated(theEdge)`: false for an inactive edge.
pub(crate) fn edge_is_degenerated(graph: &BRepGraph, edge: EdgeId) -> bool {
    let cached =
        derived_state_cache(graph).and_then(|cache| cache.is_degenerated(graph, edge).ok());
    cached.unwrap_or_else(|| {
        compute_edge_properties(graph, edge).is_some_and(|(degenerated, _)| degenerated)
    })
}

/// `IsClosed(theEdge)`: false for an inactive edge.
pub(crate) fn edge_is_closed(graph: &BRepGraph, edge: EdgeId) -> bool {
    let cached = derived_state_cache(graph).and_then(|cache| cache.is_closed(graph, edge).ok());
    cached.unwrap_or_else(|| compute_edge_properties(graph, edge).is_some_and(|(_, closed)| closed))
}

/// `SameRange(theCoEdge)`: false for an inactive coedge.
pub(crate) fn co_edge_same_range(graph: &BRepGraph, co_edge: CoEdgeId) -> bool {
    let cached = derived_state_cache(graph).and_then(|cache| cache.same_range(graph, co_edge).ok());
    cached
        .unwrap_or_else(|| is_active_co_edge(graph, co_edge) && compute_same_range(graph, co_edge))
}

/// `SameParameter(theCoEdge)`: false for an inactive coedge; the adaptor loads of the
/// sampling raise as natively.
pub(crate) fn co_edge_same_parameter(
    graph: &BRepGraph,
    co_edge: CoEdgeId,
) -> Result<bool, Failure> {
    if let Some(cache) = derived_state_cache(graph) {
        return cache.same_parameter(graph, co_edge);
    }
    if !is_active_co_edge(graph, co_edge) {
        return Ok(false);
    }
    compute_same_parameter(graph, co_edge)
}

/// `GetWireIsClosed(theWire, aClosed) && aClosed`.
pub(crate) fn wire_is_closed(graph: &BRepGraph, wire: WireId) -> bool {
    match derived_state_cache(graph).map(|cache| cache.get_wire_is_closed(graph, wire)) {
        Some(Ok(closed)) => closed.unwrap_or(false),
        _ => compute_wire_is_closed(graph, wire),
    }
}

/// `IsShellClosed(theShell)`.
pub(crate) fn shell_is_closed(graph: &BRepGraph, shell: ShellId) -> bool {
    let cached =
        derived_state_cache(graph).and_then(|cache| cache.is_shell_closed(graph, shell).ok());
    cached.unwrap_or_else(|| compute_shell_is_closed(graph, shell))
}

fn is_active_co_edge(graph: &BRepGraph, co_edge: CoEdgeId) -> bool {
    co_edge.is_valid_for(graph.topo().co_edges().nb()) && !co_edge.is_removed(graph)
}
