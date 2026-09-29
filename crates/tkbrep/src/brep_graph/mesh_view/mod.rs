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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_MeshView.hxx,
// BRepGraph_MeshView.cxx and BRepGraph::Mesh() of BRepGraph.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph::MeshView`: the mesh reads and cache writes of a graph, split by source.
//!
//! [`MeshView::cache`] reads the graph's [`CacheMesh`] only (fresh entries),
//! [`MeshView::persistent`] the mesh representations of the definitions,
//! [`MeshView::effective`] the cache first and the definitions second, and
//! [`MeshView::editor`] writes the cache. There is no hidden fallback between cache and
//! persistent data. The views borrow the graph; since the cache locks itself, the
//! `const` and non-`const` `Mesh()` accessors are one (docs/topology-design.md 11.3).
use crate::brep_graph::BRepGraph;
use crate::brep_graph::cache_mesh::CacheMesh;
use rustycad_tkernel::standard::Handle;

pub mod cache_view;
pub mod editor_view;
pub mod effective_view;
pub mod persistent_view;
pub mod poly;

use cache_view::CacheView;
use editor_view::EditorView;
use effective_view::EffectiveView;
use persistent_view::PersistentView;
use poly::PolyOps;

/// `BRepGraph::MeshView`.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, FaceId};
/// let graph = BRepGraph::new();
/// let mesh = graph.mesh();
/// assert!(!mesh.cache().faces().has(FaceId::new(0)));
/// assert!(!mesh.persistent().faces().has(FaceId::new(0)));
/// assert_eq!(mesh.poly().nb_face_triangulations(), 0);
/// ```
#[derive(Clone, Copy)]
pub struct MeshView<'g> {
    graph: &'g BRepGraph,
}

impl BRepGraph {
    /// `Mesh()` (both overloads): the mesh view of this graph.
    pub fn mesh(&self) -> MeshView<'_> {
        MeshView { graph: self }
    }
}

impl<'g> MeshView<'g> {
    /// `Cache()`: cache-only reads.
    pub fn cache(self) -> CacheView<'g> {
        CacheView::new(self.graph)
    }

    /// `Persistent()`: definition-resident reads.
    pub fn persistent(self) -> PersistentView<'g> {
        PersistentView::new(self.graph)
    }

    /// `Effective()`: cache first, persistent fallback.
    pub fn effective(self) -> EffectiveView<'g> {
        EffectiveView::new(self.graph)
    }

    /// `Editor()`: cache mutations.
    pub fn editor(self) -> EditorView<'g> {
        EditorView::new(self.graph)
    }

    /// `Poly()`: the representation counts.
    pub fn poly(self) -> PolyOps<'g> {
        PolyOps::new(self.graph)
    }
}

/// `getCacheMesh(theGraph)`: the graph's mesh cache, registered on first use; `None`
/// when another class holds its GUID (natively a null dereference).
pub(crate) fn cache_mesh_of(graph: &BRepGraph) -> Option<Handle<CacheMesh>> {
    let cache = graph.cache_registry().ensure::<CacheMesh>().ok()?;
    cache.is_some().then_some(cache)
}
