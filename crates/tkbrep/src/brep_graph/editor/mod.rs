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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.hxx,
// BRepGraph_EditorView.cxx, BRepGraph_EditorView_Mut.cxx and
// BRepGraph_EditorView_Setters.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph::EditorView`: the single mutation entry point of a graph.
//!
//! The view holds the graph's mutable borrow, and each operation group takes it over
//! (docs/topology-design.md section 11.4): `graph.editor().edges().add(...)`. A group
//! consumes the view so that the scoped guard of `mutate` (`Mut`) can outlive the
//! temporaries of the call chain; [`EditorView::reborrow`] keeps a view for several
//! groups. The members that can raise natively (lock and guard checks, `Mut` on an
//! invalid id) return `Result`; an out-of-range id that the native code indexes
//! without a check (undefined behaviour) panics with the index.
mod boundary;
mod co_edges;
mod compounds;
mod containers;
mod deferred;
mod edge_split;
mod edge_vertices;
mod edges;
mod faces;
mod gen_cleanup;
mod gen_ops;
mod gen_refs;
mod gen_remove;
mod gen_replace;
mod gen_subgraph;
mod guards;
mod helpers;
mod occurrences;
mod product_removal;
mod products;
mod ref_container;
mod removals;
mod vertices;
mod wire_checks;
mod wire_edits;
mod wires;

pub use boundary::BoundaryIssue;
pub(crate) use boundary::{BOUNDARY_KINDS, actual_active, cached_active};
pub use co_edges::CoEdgeOps;
pub use compounds::{CompSolidOps, CompoundOps};
pub use containers::{ShellOps, SolidOps};
pub use edges::EdgeOps;
pub use faces::FaceOps;
pub use gen_ops::GenOps;
pub(crate) use helpers::kind_name;
pub use occurrences::OccurrenceOps;
pub use products::ProductOps;
pub use vertices::VertexOps;
pub use wires::{CoEdgeOrderStatus, ReplaceEdgeStatus, WireOps};

use super::supplement_editor::SupplementEditor;
use crate::brep_graph::BRepGraph;

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

/// `const BRepGraph::EditorView`: the editor state queries of a shared graph
/// (`BRepGraph::Editor() const`), obtained by [`BRepGraph::editor_state`].
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// let graph = BRepGraph::new();
/// assert!(!graph.editor_state().is_deferred_mode());
/// assert!(graph.editor_state().validate_mutation_boundary(None));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct EditorState<'g> {
    graph: &'g BRepGraph,
}

impl BRepGraph {
    /// `Editor()`: the mutation view.
    pub fn editor(&mut self) -> EditorView<'_> {
        EditorView { graph: self }
    }

    /// `Editor() const`: the state queries (`IsDeferredMode`,
    /// `ValidateMutationBoundary`) that the native `const` editor exposes.
    pub fn editor_state(&self) -> EditorState<'_> {
        EditorState { graph: self }
    }
}

macro_rules! ops_accessors {
    ($($(#[$doc:meta])* $name:ident => $ops:ident;)*) => {
        impl<'g> EditorView<'g> {
            $(
                $(#[$doc])*
                pub fn $name(self) -> $ops<'g> {
                    $ops { graph: self.graph }
                }
            )*
        }
    };
}

ops_accessors! {
    /// `Vertices()`: the vertex operations.
    vertices => VertexOps;
    /// `Edges()`: the edge operations.
    edges => EdgeOps;
    /// `CoEdges()`: the coedge and p-curve operations.
    co_edges => CoEdgeOps;
    /// `Wires()`: the wire operations.
    wires => WireOps;
    /// `Faces()`: the face operations.
    faces => FaceOps;
    /// `Shells()`: the shell operations.
    shells => ShellOps;
    /// `Solids()`: the solid operations.
    solids => SolidOps;
    /// `Compounds()`: the compound operations.
    compounds => CompoundOps;
    /// `CompSolids()`: the compsolid operations.
    comp_solids => CompSolidOps;
    /// `Products()`: the product operations.
    products => ProductOps;
    /// `Occurrences()`: the occurrence operations.
    occurrences => OccurrenceOps;
    /// `Gen()`: the generic node, reference and representation operations.
    generic => GenOps;
}

impl<'g> EditorView<'g> {
    /// A shorter-lived view over the same graph, so that one binding can reach several
    /// operation groups in turn.
    pub fn reborrow(&mut self) -> EditorView<'_> {
        EditorView { graph: self.graph }
    }

    /// `Supplement()`: the runtime supplement attachment operations.
    pub fn supplement(self) -> SupplementEditor<'g> {
        SupplementEditor::new(self.graph)
    }

    /// The state queries of the edited graph.
    pub fn state(&self) -> EditorState<'_> {
        EditorState { graph: self.graph }
    }

    /// `IsDeferredMode()`.
    pub fn is_deferred_mode(&self) -> bool {
        self.state().is_deferred_mode()
    }
}

impl EditorState<'_> {
    /// `IsDeferredMode()`: deferred invalidation is active. A state flag only: it does
    /// not imply mutation ownership.
    pub fn is_deferred_mode(self) -> bool {
        self.graph.inc_storage().deferred_mode()
    }
}
