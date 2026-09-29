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
// Rust translation of the BRepGraph_CacheDerivedState cache service (IsDegenerated,
// SameParameter, SameRange, IsClosed, GetWireIsClosed, SetWireIsClosed,
// IsShellClosed, Clear and the ensure* members) of
// ModelingData/TKBRep/BRepGraph/BRepGraph_CacheDerivedState.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_CacheDerivedState`: the memoised derived edge, coedge, wire and shell
//! state of a graph.
use super::entries::{
    COMPUTED_SAME_PARAM, COMPUTED_SAME_RANGE, CoEdgeSameRangeEntry, EdgeEntry, ShellEntry,
    WireEntry,
};
use super::{
    EdgeGeomStatus, ShellClosureStatus, compute_same_range, compute_shell_closure,
    compute_wire_is_closed, edge_status, same_parameter_with_range,
};
use crate::brep_graph::cache::{BRepGraphCache, CacheBase, CacheType};
use crate::brep_graph::copy_remap::CopyRemap;
use crate::brep_graph::{BRepGraph, CoEdgeId, EdgeId, NodeId, ShellId, WireId};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Guid, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::OnceLock;

/// `myEdgeEntries`, `myCoEdgeSameRangeEntries`, `myWireEntries`, `myShellEntries`,
/// indexed by the entity index.
#[derive(Clone, Default)]
pub(super) struct DerivedEntries {
    pub(super) edges: Vec<EdgeEntry>,
    pub(super) co_edges: Vec<CoEdgeSameRangeEntry>,
    pub(super) wires: Vec<WireEntry>,
    pub(super) shells: Vec<ShellEntry>,
}

/// `ensureSize(theArray, theIndex)`: grows the entries to hold `index`.
pub(super) fn ensure_size<T: Default + Clone>(entries: &mut Vec<T>, index: u32) {
    if index as usize >= entries.len() {
        entries.resize(index as usize + 1, T::default());
    }
}

/// `BRepGraph_CacheDerivedState`: the derived properties of edges (degenerated,
/// closed), coedges (SameRange, SameParameter), wires and shells (closed), each
/// computed on the first query and kept while the entity's OwnGen is unchanged. Every
/// query takes the graph the cache is attached to: a detached cache raises
/// Standard_ProgramError (`Graph()`) when it has to compute.
///
/// ```
/// use rustycad_tkbrep::brep_graph::cache_derived_state::CacheDerivedState;
/// use rustycad_tkbrep::brep_graph::{BRepGraph, EdgeId};
/// let graph = BRepGraph::new();
/// let cache = graph.cache_registry().ensure::<CacheDerivedState>().unwrap();
/// assert!(!cache.is_degenerated(&graph, EdgeId::new(0)).unwrap());
/// assert!(CacheDerivedState::new().is_closed(&graph, EdgeId::new(0)).is_err());
/// ```
#[derive(Default)]
pub struct CacheDerivedState {
    transient: StandardTransient,
    pub(super) base: CacheBase,
    pub(super) entries: SharedState<DerivedEntries>,
}
impl_transient!(CacheDerivedState, "BRepGraph_CacheDerivedState", dyn BRepGraphCache, transient,
    interfaces: [dyn BRepGraphCache]);

/// `GetID()`: a1c2e3f4-5678-4abc-9def-0123456789ab.
const CACHE_DERIVED_STATE_ID: Guid = Guid::from_components(
    0xa1c2e3f4_u32 as i32,
    0x5678,
    0x4abc,
    0x9def,
    0x01,
    0x23,
    0x45,
    0x67,
    0x89,
    0xab,
);

impl CacheType for CacheDerivedState {
    fn get_id() -> Guid {
        CACHE_DERIVED_STATE_ID
    }
}

impl CacheDerivedState {
    /// A detached, empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// `IsDegenerated(theEdge)`: no 3D curve and collapsed vertices; false for an
    /// inactive edge.
    pub fn is_degenerated(&self, graph: &BRepGraph, edge: EdgeId) -> Result<bool, Failure> {
        let entry = self.ensure_edge_entry(graph, edge)?;
        Ok(entry.is_some_and(|entry| entry.status() == EdgeGeomStatus::DegenerateOnSurface))
    }

    /// `IsClosed(theEdge)`: the same start and end vertex; false for an inactive edge.
    pub fn is_closed(&self, graph: &BRepGraph, edge: EdgeId) -> Result<bool, Failure> {
        let entry = self.ensure_edge_entry(graph, edge)?;
        Ok(entry.is_some_and(|entry| entry.is_closed()))
    }

    /// `SameRange(theCoEdge)`: false for an inactive coedge.
    pub fn same_range(&self, graph: &BRepGraph, co_edge: CoEdgeId) -> Result<bool, Failure> {
        let entry = self.ensure_co_edge_entry(graph, co_edge, COMPUTED_SAME_RANGE)?;
        Ok(entry.is_some_and(|entry| entry.same_range()))
    }

    /// `SameParameter(theCoEdge)`: false for an inactive coedge; the adaptor loads of
    /// the sampling raise as natively.
    pub fn same_parameter(&self, graph: &BRepGraph, co_edge: CoEdgeId) -> Result<bool, Failure> {
        let entry = self.ensure_co_edge_entry(graph, co_edge, COMPUTED_SAME_PARAM)?;
        Ok(entry.is_some_and(|entry| entry.same_parameter()))
    }

    /// `ensureEdgeEntry(theEdge, theEntry)` (private): the fresh computed entry, or a
    /// new one computed and bound; `None` for an inactive edge.
    fn ensure_edge_entry(
        &self,
        graph: &BRepGraph,
        edge: EdgeId,
    ) -> Result<Option<EdgeEntry>, Failure> {
        let node = NodeId::from(edge);
        if let Some(cached) = self.fresh_edge(graph, edge) {
            return Ok(Some(cached));
        }
        let graph = self.base.graph(graph)?;
        if !edge.is_valid_for(graph.topo().edges().nb()) || edge.is_removed(graph) {
            return Ok(None);
        }
        let mut computed = EdgeEntry::default();
        let (status, closed) = edge_status(graph, edge);
        computed.set(status, closed);
        Ok(self.store_edge_entry(graph, node, edge.index, computed))
    }

    // The locked tail of `ensureEdgeEntry`: a fresh computed stored entry wins, else the
    // computed one is stored and bound (reset and `None` when the bind fails).
    fn store_edge_entry(
        &self,
        graph: &BRepGraph,
        node: NodeId,
        index: u32,
        computed: EdgeEntry,
    ) -> Option<EdgeEntry> {
        let mut entries = self.entries.write();
        ensure_size(&mut entries.edges, index);
        let stored = &mut entries.edges[index as usize];
        if stored.node.is_fresh_own(&self.base, graph, node) && stored.is_computed() {
            return Some(*stored);
        }
        *stored = computed;
        if !stored.node.bind_own_gen(&self.base, graph, node) {
            stored.node.reset();
            return None;
        }
        Some(*stored)
    }

    // The lock-free read path of `ensureEdgeEntry`.
    fn fresh_edge(&self, graph: &BRepGraph, edge: EdgeId) -> Option<EdgeEntry> {
        let entries = self.entries.read();
        let cached = entries.edges.get(edge.index as usize)?;
        let fresh = cached.node.is_fresh_own(&self.base, graph, edge.into());
        (fresh && cached.is_computed()).then_some(*cached)
    }

    /// `ensureCoEdgeSameRangeEntry(theCoEdge, theRequiredFlags, theEntry)` (private): a
    /// fresh entry with the required flags computed, or the missing flags computed on
    /// a copy of the fresh entry and merged into the stored one.
    fn ensure_co_edge_entry(
        &self,
        graph: &BRepGraph,
        co_edge: CoEdgeId,
        required: u8,
    ) -> Result<Option<CoEdgeSameRangeEntry>, Failure> {
        let fresh = self.fresh_co_edge(graph, co_edge);
        if let Some(cached) = fresh.filter(|cached| cached.computed() & required == required) {
            return Ok(Some(cached));
        }
        let graph = self.base.graph(graph)?;
        if !co_edge.is_valid_for(graph.topo().co_edges().nb()) || co_edge.is_removed(graph) {
            return Ok(None);
        }
        let mut computed = self.fresh_co_edge(graph, co_edge).unwrap_or_default();
        compute_missing_flags(graph, co_edge, required, &mut computed)?;
        Ok(self.store_co_edge_entry(graph, co_edge, &computed))
    }

    // The lock-free read path of `ensureCoEdgeSameRangeEntry`: the fresh entry.
    fn fresh_co_edge(&self, graph: &BRepGraph, co_edge: CoEdgeId) -> Option<CoEdgeSameRangeEntry> {
        let entries = self.entries.read();
        let cached = entries.co_edges.get(co_edge.index as usize)?;
        cached
            .node
            .is_fresh_own(&self.base, graph, co_edge.into())
            .then_some(*cached)
    }

    // The locked tail of `ensureCoEdgeSameRangeEntry`: merge into a fresh stored entry
    // or replace it, then bind.
    fn store_co_edge_entry(
        &self,
        graph: &BRepGraph,
        co_edge: CoEdgeId,
        computed: &CoEdgeSameRangeEntry,
    ) -> Option<CoEdgeSameRangeEntry> {
        let node = NodeId::from(co_edge);
        let mut entries = self.entries.write();
        ensure_size(&mut entries.co_edges, co_edge.index);
        let stored = &mut entries.co_edges[co_edge.index as usize];
        if stored.node.is_fresh_own(&self.base, graph, node) {
            computed.merge_into(stored);
        } else {
            *stored = *computed;
        }
        if !stored.node.bind_own_gen(&self.base, graph, node) {
            stored.node.reset();
            return None;
        }
        Some(*stored)
    }

    /// `GetWireIsClosed(theWire, theClosed)`: the fresh stored closure, or the computed
    /// one stored and bound; `None` when the wire cannot be bound (inactive).
    pub fn get_wire_is_closed(
        &self,
        graph: &BRepGraph,
        wire: WireId,
    ) -> Result<Option<bool>, Failure> {
        if let Some(closed) = self.fresh_wire(graph, wire) {
            return Ok(Some(closed));
        }
        let graph = self.base.graph(graph)?;
        let closed = compute_wire_is_closed(graph, wire);
        Ok(self.store_wire(graph, wire, closed).then_some(closed))
    }

    /// `SetWireIsClosed(theWire, theClosed)`: stores a closure computed elsewhere (not
    /// stored for an inactive wire).
    pub fn set_wire_is_closed(&self, graph: &BRepGraph, wire: WireId, closed: bool) {
        self.store_wire(graph, wire, closed);
    }

    // The lock-free read path of `GetWireIsClosed`.
    fn fresh_wire(&self, graph: &BRepGraph, wire: WireId) -> Option<bool> {
        let entries = self.entries.read();
        let cached = entries.wires.get(wire.index as usize)?;
        let fresh =
            cached.is_computed() && cached.node.is_fresh_own(&self.base, graph, wire.into());
        fresh.then(|| cached.is_closed())
    }

    // The locked store of `GetWireIsClosed` and `SetWireIsClosed`: false (the entry
    // reset) when the wire cannot be bound. An index the graph cannot bind is refused
    // before the entries grow (natively they grow to it, which for the invalid index is
    // a 2^32-entry allocation); the answer is the same.
    fn store_wire(&self, graph: &BRepGraph, wire: WireId, closed: bool) -> bool {
        let node = NodeId::from(wire);
        let mut entries = self.entries.write();
        if self.base.node_own_gen(graph, node).is_none() {
            let stored = entries.wires.get_mut(wire.index as usize);
            reset_node(stored.map(|stored| &mut stored.node));
            return false;
        }
        ensure_size(&mut entries.wires, wire.index);
        let stored = &mut entries.wires[wire.index as usize];
        if !stored.node.bind_own_gen(&self.base, graph, node) {
            stored.node.reset();
            return false;
        }
        stored.set_closed(closed);
        true
    }

    /// `IsShellClosed(theShell)`: the fresh stored status, or the computed one stored
    /// and bound; false for an inactive shell.
    pub fn is_shell_closed(&self, graph: &BRepGraph, shell: ShellId) -> Result<bool, Failure> {
        if let Some(status) = self.fresh_shell(graph, shell) {
            return Ok(status == ShellClosureStatus::Closed);
        }
        let graph = self.base.graph(graph)?;
        let status = compute_shell_closure(graph, shell);
        Ok(self.store_shell(graph, shell, status) == Some(ShellClosureStatus::Closed))
    }

    // The lock-free read path of `IsShellClosed`.
    fn fresh_shell(&self, graph: &BRepGraph, shell: ShellId) -> Option<ShellClosureStatus> {
        let entries = self.entries.read();
        let cached = entries.shells.get(shell.index as usize)?;
        let computed = cached.status != ShellClosureStatus::Invalid;
        let fresh = computed && cached.node.is_fresh_own(&self.base, graph, shell.into());
        fresh.then_some(cached.status)
    }

    // The locked tail of `IsShellClosed`: a fresh stored status wins; `None` when the
    // shell cannot be bound (refused before growing, as for wires).
    fn store_shell(
        &self,
        graph: &BRepGraph,
        shell: ShellId,
        status: ShellClosureStatus,
    ) -> Option<ShellClosureStatus> {
        let node = NodeId::from(shell);
        let mut entries = self.entries.write();
        if self.base.node_own_gen(graph, node).is_none() {
            reset_node(
                entries
                    .shells
                    .get_mut(shell.index as usize)
                    .map(|stored| &mut stored.node),
            );
            return None;
        }
        ensure_size(&mut entries.shells, shell.index);
        let stored = &mut entries.shells[shell.index as usize];
        if stored.status != ShellClosureStatus::Invalid
            && stored.node.is_fresh_own(&self.base, graph, node)
        {
            return Some(stored.status);
        }
        if !stored.node.bind_own_gen(&self.base, graph, node) {
            stored.node.reset();
            return None;
        }
        stored.status = status;
        Some(status)
    }
}

// The failed bind of an entity the graph cannot bind: an existing entry is unbound.
fn reset_node(entry: Option<&mut crate::brep_graph::cache::NodeEntry>) {
    if let Some(entry) = entry {
        entry.reset();
    }
}

// The flag computations of `ensureCoEdgeSameRangeEntry` on a copy of the fresh entry:
// `computeSameRange` when required and missing, then `computeSameParameter` (which
// computes SameRange first when it is missing).
fn compute_missing_flags(
    graph: &BRepGraph,
    co_edge: CoEdgeId,
    required: u8,
    entry: &mut CoEdgeSameRangeEntry,
) -> Result<(), Failure> {
    if entry.computed() & COMPUTED_SAME_RANGE == 0 && required & COMPUTED_SAME_RANGE != 0 {
        entry.set_same_range(compute_same_range(graph, co_edge));
    }
    if entry.computed() & COMPUTED_SAME_PARAM == 0 && required & COMPUTED_SAME_PARAM != 0 {
        if entry.computed() & COMPUTED_SAME_RANGE == 0 {
            entry.set_same_range(compute_same_range(graph, co_edge));
        }
        let same_range = entry.same_range();
        entry.set_same_parameter(same_parameter_with_range(graph, co_edge, || same_range)?);
    }
    Ok(())
}

impl BRepGraphCache for CacheDerivedState {
    fn cache_base(&self) -> &CacheBase {
        &self.base
    }

    fn id(&self) -> Guid {
        CACHE_DERIVED_STATE_ID
    }

    fn name(&self) -> &AsciiString {
        static NAME: OnceLock<AsciiString> = OnceLock::new();
        NAME.get_or_init(|| AsciiString::new("BRepGraph_CacheDerivedState"))
    }

    /// `Clear()`: every entry's node binding is reset (their flags stay, unread).
    fn clear(&self) {
        let mut entries = self.entries.write();
        entries
            .edges
            .iter_mut()
            .for_each(|entry| entry.node.reset());
        entries
            .co_edges
            .iter_mut()
            .for_each(|entry| entry.node.reset());
        entries
            .wires
            .iter_mut()
            .for_each(|entry| entry.node.reset());
        entries
            .shells
            .iter_mut()
            .for_each(|entry| entry.node.reset());
    }

    fn copy_fresh_to(&self, copy: &CopyRemap<'_>) -> Result<(), Failure> {
        self.copy_fresh_entries_to(copy)
    }
}
