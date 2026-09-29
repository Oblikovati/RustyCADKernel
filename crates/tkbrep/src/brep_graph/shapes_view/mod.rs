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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_ShapesView.hxx and the
// Shape, HasOriginal, Original, Reconstruct, FindNode, HasNode, collectAddedNodes and
// CollectHistoryInputs members of BRepGraph_ShapesView.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph::ShapesView`: the shapes of the nodes and the ingestion of shapes.
//!
//! [`ShapesView`] (`graph.shapes()`) answers the `const` queries; [`ShapesViewMut`]
//! (`graph.shapes_mut()`) holds the mutable borrow for `Add` and `ClearCached`.
mod add;
mod ingest;
mod reconstruction;
mod route;

pub use add::ShapesViewMut;

use super::{BRepGraph, NodeId, NodeKind, OccurrenceId, ProductId, RefId};
use crate::brep_graph_inc::CachedShape;
use crate::brep_graph_inc::populate;
use crate::topexp::Explorer;
use crate::topods::Shape;
use crate::toptools::ShapeMapHasher;
use reconstruction::ReconstructionContext;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `NCollection_DataMap<TopoDS_Shape, BRepGraph_NodeId, TopTools_ShapeMapHasher>`: the
/// shape-to-node map of `Result::AddedNodes` and `CollectHistoryInputs`.
pub type ShapeNodeMap = DataMap<Shape, NodeId, ShapeMapHasher>;

/// `ShapesView::Options`: the population options of `Add`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    /// `Populate`.
    pub populate: populate::Options,
    /// `CreateAutoProduct`: wraps the topology root in a product (unparented `Add`).
    pub create_auto_product: bool,
    /// `Flatten`: drops the hierarchy containers, appending faces as roots.
    pub flatten: bool,
    /// `Parallel`: face-level construction in parallel.
    pub parallel: bool,
    /// `TrackAddedNodes`: fills `AddResult::added_nodes`.
    pub track_added_nodes: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            populate: populate::Options,
            create_auto_product: true,
            flatten: false,
            parallel: false,
            track_added_nodes: false,
        }
    }
}

/// `ShapesView::AddStatus`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AddStatus {
    /// `Success`.
    Success,
    /// `SuccessWithWarnings`: diagnostics such as unbounded natural faces.
    SuccessWithWarnings,
    /// `Failed`: e.g. a null shape.
    #[default]
    Failed,
}

/// `ShapesView::Result`: what `Add` created.
#[derive(Default)]
pub struct AddResult {
    /// `TopologyRoot`.
    pub topology_root: NodeId,
    /// `Product`.
    pub product: ProductId,
    /// `Occurrence`.
    pub occurrence: OccurrenceId,
    /// `InsertedRef`: the reference created under the parent.
    pub inserted_ref: RefId,
    /// `Status`.
    pub status: AddStatus,
    /// `AddedNodes`: every sub-shape of the input bound to its node (with
    /// `track_added_nodes`).
    pub added_nodes: ShapeNodeMap,
}

impl AddResult {
    /// `IsOk()`: not `Failed`; `AddResult::default().is_ok()` is false.
    pub fn is_ok(&self) -> bool {
        self.status != AddStatus::Failed
    }
}

/// `BRepGraph::ShapesView` (the `const` members): obtained by [`BRepGraph::shapes`].
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// use rustycad_tkbrep::topods::Shape;
/// let graph = BRepGraph::new();
/// assert!(!graph.shapes().has_node(&Shape::default()));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ShapesView<'g> {
    graph: &'g BRepGraph,
}

impl BRepGraph {
    /// `Shapes()` (`const`): the shape queries.
    pub fn shapes(&self) -> ShapesView<'_> {
        ShapesView { graph: self }
    }
}

impl ShapesView<'_> {
    /// `Shape(theNode)`: the original shape of a never-mutated node, else the cached
    /// reconstruction of its current SubtreeGen, else a new reconstruction (cached and,
    /// for topology nodes, bound to the node); null for an invalid or removed node.
    pub fn shape(self, node: NodeId) -> Result<Shape, Failure> {
        if !node.is_valid() || node.is_removed(self.graph) {
            return Ok(Shape::default());
        }
        let Some(subtree_gen) = self.graph.topo_entity(node).map(|def| def.subtree_gen) else {
            return Ok(Shape::default());
        };
        if subtree_gen == 0 {
            let original = self.original(node);
            if !original.is_null() {
                return Ok(original);
            }
        }
        if let Some(cached) = self.fresh_cached(node, subtree_gen) {
            return Ok(cached);
        }
        let reconstructed = ReconstructionContext::new(self.graph).shape(node)?;
        if reconstructed.is_null() {
            return Ok(reconstructed);
        }
        Ok(self.store_reconstructed(node, subtree_gen, reconstructed))
    }

    // The cached shape of the node when it was built at the SubtreeGen.
    fn fresh_cached(self, node: NodeId, subtree_gen: u32) -> Option<Shape> {
        let cache = self.graph.inc_storage().current_shapes();
        let cached = cache.get(&node)?;
        (cached.stored_subtree_gen == subtree_gen).then(|| cached.shape.clone())
    }

    // Stores under the write lock with a double check (a concurrent reconstruction of
    // the same node wins); binds topology nodes to the new shape.
    fn store_reconstructed(self, node: NodeId, subtree_gen: u32, reconstructed: Shape) -> Shape {
        let storage = self.graph.inc_storage();
        let mut cache = storage.change_current_shapes();
        if let Some(existing) = cache
            .get(&node)
            .filter(|entry| entry.stored_subtree_gen == subtree_gen)
        {
            return existing.shape.clone();
        }
        let entry = CachedShape {
            shape: reconstructed.clone(),
            stored_subtree_gen: subtree_gen,
        };
        cache.insert(node, entry);
        if node.kind != NodeKind::PRODUCT && node.kind != NodeKind::OCCURRENCE {
            storage.set_definition_shape_binding(&reconstructed, node);
        }
        reconstructed
    }

    /// `HasOriginal(theNode)`: a non-null original.
    pub fn has_original(self, node: NodeId) -> bool {
        !self.original(node).is_null()
    }

    /// `Original(theNode)`: the construction-time shape of an active node, else null.
    pub fn original(self, node: NodeId) -> Shape {
        if !node.is_valid() || node.is_removed(self.graph) {
            return Shape::default();
        }
        self.graph.inc_storage().find_original(node)
    }

    /// `Reconstruct(theRoot)`: a fresh reconstruction (no cache read or write); null for
    /// an invalid or removed root.
    pub fn reconstruct(self, root: NodeId) -> Result<Shape, Failure> {
        if !root.is_valid() || root.is_removed(self.graph) {
            return Ok(Shape::default());
        }
        ReconstructionContext::new(self.graph).shape(root)
    }

    /// `FindNode(theShape)`: the active node bound to the shape (TShape and location).
    pub fn find_node(self, shape: &Shape) -> NodeId {
        if shape.is_null() {
            return NodeId::default();
        }
        let node = self.graph.inc_storage().find_definition_by_shape(shape);
        if !node.is_valid() || node.is_removed(self.graph) || self.graph.topo_entity(node).is_none()
        {
            return NodeId::default();
        }
        node
    }

    /// `HasNode(theShape)`.
    pub fn has_node(self, shape: &Shape) -> bool {
        !shape.is_null() && self.find_node(shape).is_valid()
    }

    /// `CollectHistoryInputs(theRoots, theOutInputs)`: every sub-shape of the roots'
    /// shapes bound to its node.
    pub fn collect_history_inputs(
        self,
        roots: &[NodeId],
        inputs: &mut ShapeNodeMap,
    ) -> Result<(), Failure> {
        for &root in roots {
            let shape = self.shape(root)?;
            self.collect_added_nodes(&shape, inputs)?;
        }
        Ok(())
    }

    /// `collectAddedNodes(theGraph, theShape, theMap)`: the shape, then its sub-shapes
    /// by kind from COMPOUND to VERTEX in explorer order, each bound once to its node.
    pub(super) fn collect_added_nodes(
        self,
        shape: &Shape,
        map: &mut ShapeNodeMap,
    ) -> Result<(), Failure> {
        if shape.is_null() {
            return Ok(());
        }
        self.bind_found(shape, map);
        const KINDS: [ShapeEnum; 8] = [
            ShapeEnum::Compound,
            ShapeEnum::CompSolid,
            ShapeEnum::Solid,
            ShapeEnum::Shell,
            ShapeEnum::Face,
            ShapeEnum::Wire,
            ShapeEnum::Edge,
            ShapeEnum::Vertex,
        ];
        for kind in KINDS {
            let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape)?;
            while explorer.more() {
                self.bind_found(explorer.current(), map);
                explorer.next()?;
            }
        }
        Ok(())
    }

    fn bind_found(self, shape: &Shape, map: &mut ShapeNodeMap) {
        if shape.is_null() || map.is_bound(shape) {
            return;
        }
        let node = self.find_node(shape);
        if node.is_valid() {
            map.bind(shape.clone(), node);
        }
    }
}
