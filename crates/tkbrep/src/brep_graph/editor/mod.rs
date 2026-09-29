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
// Rust translation of the EditorView members that BRepGraph::ShapesView::Add calls,
// from ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.hxx, BRepGraph_EditorView.cxx
// and BRepGraph_EditorView_Setters.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph::EditorView`: the single mutation entry point of a graph.
//!
//! The view holds the graph's mutable borrow, and its operation groups reborrow it
//! (docs/topology-design.md section 11.4). Phase 1 ports the product, container-append
//! and child-location members that shape ingestion needs, with the lock and guard
//! checks; the editor area of phase 2 adds the rest of `EditorView` and `MutGuard`
//! here.
mod appends;
mod products;

pub use appends::{CompSolidOps, CompoundOps, GenOps, ShellOps, SolidOps};
pub use products::ProductOps;

use crate::brep_graph::{BRepGraph, ItemDomain, ItemId, NodeId, NodeKind};
use rustycad_tkernel::standard::Failure;

/// `BRepGraph::EditorView`: obtained by [`BRepGraph::editor`].
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// let mut graph = BRepGraph::new();
/// let product = graph.editor().products().add_empty();
/// assert!(product.is_valid());
/// ```
#[derive(Debug)]
pub struct EditorView<'g> {
    graph: &'g mut BRepGraph,
}

impl BRepGraph {
    /// `Editor()`: the mutation view.
    pub fn editor(&mut self) -> EditorView<'_> {
        EditorView { graph: self }
    }
}

impl EditorView<'_> {
    /// `Products()`: the product operations.
    pub fn products(&mut self) -> ProductOps<'_> {
        ProductOps { graph: self.graph }
    }

    /// `Compounds()`: the compound operations.
    pub fn compounds(&mut self) -> CompoundOps<'_> {
        CompoundOps { graph: self.graph }
    }

    /// `Shells()`: the shell operations.
    pub fn shells(&mut self) -> ShellOps<'_> {
        ShellOps { graph: self.graph }
    }

    /// `Solids()`: the solid operations.
    pub fn solids(&mut self) -> SolidOps<'_> {
        SolidOps { graph: self.graph }
    }

    /// `CompSolids()`: the compsolid operations.
    pub fn comp_solids(&mut self) -> CompSolidOps<'_> {
        CompSolidOps { graph: self.graph }
    }

    /// `Gen()`: the generic operations.
    pub fn generic(&mut self) -> GenOps<'_> {
        GenOps { graph: self.graph }
    }
}

/// `isOwned(theItem)` (private natively): the item is in range and owned.
pub(crate) fn is_owned(graph: &BRepGraph, item: ItemId) -> bool {
    match item.item_domain() {
        ItemDomain::Node => item.node_id().is_owned(graph),
        ItemDomain::Reference => item.ref_id().is_owned(graph),
        ItemDomain::None => false,
    }
}

/// `requireUnlocked(theItem, theOperation)` (private natively): an owned item raises
/// Standard_ProgramError with the operation text.
pub(crate) fn require_unlocked(
    graph: &BRepGraph,
    item: ItemId,
    operation: &str,
) -> Result<(), Failure> {
    if is_owned(graph, item) {
        return Err(program_error(operation, item));
    }
    Ok(())
}

/// `requireNoActiveGuard(theItem, theOperation)` (private natively): a guarded item
/// raises Standard_ProgramError with the operation text.
pub(crate) fn require_no_active_guard(
    graph: &BRepGraph,
    item: ItemId,
    operation: &str,
) -> Result<(), Failure> {
    if graph.inc_storage().is_item_guarded(item) {
        return Err(program_error(operation, item));
    }
    Ok(())
}

/// Both checks of an editor member on one node, in native order, with the native
/// `locked` and `guard active` texts.
pub(crate) fn require_mutable(
    graph: &BRepGraph,
    node: NodeId,
    locked: &str,
    guarded: &str,
) -> Result<(), Failure> {
    require_unlocked(graph, node.into(), locked)?;
    require_no_active_guard(graph, node.into(), guarded)
}

fn program_error(operation: &str, item: ItemId) -> Failure {
    Failure::with_type(
        "Standard_ProgramError",
        &format!(
            "{operation} (item domain {:?}, kind {}, index {})",
            item.item_domain(),
            item.raw_kind(),
            item.index()
        ),
        "",
    )
}

/// `isActiveNode` (private natively): in range and not removed.
pub(crate) fn is_active_node(graph: &BRepGraph, node: NodeId) -> bool {
    graph.topo().generic().is_active(node)
}

/// `isActiveTopologyNode` (private natively): an active node of a topology kind.
pub(crate) fn is_active_topology_node(graph: &BRepGraph, node: NodeId) -> bool {
    node.is_valid() && NodeKind::is_topology_kind(node.kind) && is_active_node(graph, node)
}
