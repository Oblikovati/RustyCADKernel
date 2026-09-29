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
// Rust translation of the Add and ClearCached members of
// ModelingData/TKBRep/BRepGraph/BRepGraph_ShapesView.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Shapes().Add` and `Shapes().ClearCached`: ingestion of a shape as a new root
//! (optionally wrapped in a product) or under a parent node.
use super::ingest::{
    append_impl, bind_source_shape_aliases, detect_topology_root, snapshot_count_for_kind,
};
use super::route::{
    ShapeParentRoute, attach_supplement_to_parent, classify_shape_to_parent,
    shape_without_own_location, should_store_root_location_in_ref,
};
use super::{AddResult, AddStatus, Options};
use crate::brep_graph::{
    BRepGraph, CompSolidId, CompoundId, FaceId, NodeId, NodeKind, OccurrenceId, ProductId, RefId,
    ShellId, SolidId,
};
use crate::brep_graph_inc::ParityOrientation;
use crate::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::toploc::Location;

/// `BRepGraph::ShapesView` (the mutating members): obtained by
/// [`BRepGraph::shapes_mut`].
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// use rustycad_tkbrep::topods::Shape;
/// let mut graph = BRepGraph::new();
/// assert!(!graph.shapes_mut().add(&Shape::default()).unwrap().is_ok());
/// ```
pub struct ShapesViewMut<'g> {
    graph: &'g mut BRepGraph,
}

impl BRepGraph {
    /// `Shapes()` (non-`const`): `Add` and `ClearCached`.
    pub fn shapes_mut(&mut self) -> ShapesViewMut<'_> {
        ShapesViewMut { graph: self }
    }
}

// The topology root of an append: the first flattened root, else the first new node of
// the shape's kind, else the node already bound to the populated shape.
fn resolve_topology_root(
    graph: &BRepGraph,
    shape: &Shape,
    populate_shape: &Shape,
    flat_roots: &[NodeId],
    old_count: u32,
) -> Result<NodeId, Failure> {
    let mut root = match flat_roots.first() {
        Some(first) => *first,
        None => detect_topology_root(graph, shape.shape_type()?, old_count),
    };
    if !root.is_valid() {
        root = graph.shapes().find_node(populate_shape);
    }
    Ok(root)
}

// The shape to populate (without its own location when the reference keeps it) and
// whether the reference keeps it.
fn populate_shape_for(
    shape: &Shape,
    options: &Options,
    parent: NodeId,
) -> Result<(Shape, bool), Failure> {
    let stored_in_ref = should_store_root_location_in_ref(options, parent);
    let populate_shape = if stored_in_ref {
        shape_without_own_location(shape)?
    } else {
        shape.clone()
    };
    Ok((populate_shape, stored_in_ref))
}

impl ShapesViewMut<'_> {
    /// `Add(theShape)`: `Add` with the default options (an auto product).
    pub fn add(&mut self, shape: &Shape) -> Result<AddResult, Failure> {
        self.add_with_options(shape, &Options::default())
    }

    /// `Add(theShape, theOptions)`: appends the shape (without its own location when the
    /// auto product keeps it) and wraps the topology root in a document-root product
    /// placed at the shape location. Failed for a null shape or without root.
    pub fn add_with_options(
        &mut self,
        shape: &Shape,
        options: &Options,
    ) -> Result<AddResult, Failure> {
        let mut result = AddResult::default();
        if shape.is_null() {
            return Ok(result);
        }
        let old_count = snapshot_count_for_kind(self.graph, shape.shape_type()?);
        let (populate_shape, stored_in_ref) =
            populate_shape_for(shape, options, NodeId::default())?;
        let mut flat_roots = Vec::new();
        result.status = append_impl(self.graph, &populate_shape, options, &mut flat_roots)?;
        result.topology_root =
            resolve_topology_root(self.graph, shape, &populate_shape, &flat_roots, old_count)?;
        if stored_in_ref && result.topology_root.is_valid() {
            bind_source_shape_aliases(self.graph, shape, &populate_shape)?;
        }
        if options.create_auto_product && result.topology_root.is_valid() {
            self.wrap_in_auto_product(&mut result, shape.location())?;
        }
        let has_product = options.create_auto_product && result.product.is_valid();
        if !result.topology_root.is_valid() && !has_product {
            result.status = AddStatus::Failed;
        }
        if options.track_added_nodes && result.topology_root.is_valid() {
            self.graph
                .shapes()
                .collect_added_nodes(shape, &mut result.added_nodes)?;
        }
        Ok(result)
    }

    // The auto product of the root, made a document root, and its first occurrence.
    fn wrap_in_auto_product(
        &mut self,
        result: &mut AddResult,
        placement: &Location,
    ) -> Result<(), Failure> {
        let mut editor = self.graph.editor();
        result.product = editor.products().add(result.topology_root, placement);
        editor.products().append_document_root(result.product)?;
        if !result.product.is_valid() {
            return Ok(());
        }
        let storage = self.graph.inc_storage();
        if let Some(first) = storage
            .product_relations(result.product)
            .occurrence_ref_ids
            .first()
        {
            result.occurrence = storage.occurrence_ref(*first).child_occurrence_id;
        }
        Ok(())
    }

    /// `Add(theShape, theParent)`: `Add` under a parent with the default options.
    pub fn add_under(&mut self, shape: &Shape, parent: NodeId) -> Result<AddResult, Failure> {
        self.add_under_with_options(shape, parent, &Options::default())
    }

    /// `Add(theShape, theParent, theOptions)`: a core child is appended (never with an
    /// auto product) and referenced from the parent; a supplement child is attached to
    /// the parent's supplement layer; anything else (or a null shape or parent) fails.
    pub fn add_under_with_options(
        &mut self,
        shape: &Shape,
        parent: NodeId,
        options: &Options,
    ) -> Result<AddResult, Failure> {
        let mut result = AddResult::default();
        if shape.is_null() || !parent.is_valid() {
            return Ok(result);
        }
        match classify_shape_to_parent(shape.shape_type()?, shape.orientation(), parent) {
            ShapeParentRoute::Reject => Ok(result),
            ShapeParentRoute::Supplement => {
                if attach_supplement_to_parent(self.graph, shape, parent) != 0 {
                    result.status = AddStatus::Success;
                }
                Ok(result)
            }
            ShapeParentRoute::Core => self.add_core_child(shape, parent, options, result),
        }
    }

    // The core route: append, find the root, bind the aliases, then reference it.
    fn add_core_child(
        &mut self,
        shape: &Shape,
        parent: NodeId,
        options: &Options,
        mut result: AddResult,
    ) -> Result<AddResult, Failure> {
        let old_count = snapshot_count_for_kind(self.graph, shape.shape_type()?);
        let (populate_shape, stored_in_ref) = populate_shape_for(shape, options, parent)?;
        let inner = Options {
            create_auto_product: false,
            ..*options
        };
        let mut flat_roots = Vec::new();
        if append_impl(self.graph, &populate_shape, &inner, &mut flat_roots)? == AddStatus::Failed {
            return Ok(result);
        }
        result.topology_root =
            resolve_topology_root(self.graph, shape, &populate_shape, &flat_roots, old_count)?;
        if !result.topology_root.is_valid() {
            return Ok(result);
        }
        if stored_in_ref {
            bind_source_shape_aliases(self.graph, shape, &populate_shape)?;
        }
        if inner.track_added_nodes {
            self.graph
                .shapes()
                .collect_added_nodes(shape, &mut result.added_nodes)?;
        }
        self.reference_from_parent(shape, parent, &mut result)?;
        Ok(result)
    }

    // The parent's reference to the new root (a child product and its occurrence under a
    // product parent); Success when it was created.
    fn reference_from_parent(
        &mut self,
        shape: &Shape,
        parent: NodeId,
        result: &mut AddResult,
    ) -> Result<(), Failure> {
        if parent.kind == NodeKind::PRODUCT {
            return self.reference_from_product(shape, ProductId::from_node_id(parent), result);
        }
        let root = result.topology_root;
        let orientation = || ParityOrientation::new(shape.orientation());
        let mut editor = self.graph.editor();
        let inserted: RefId = match parent.kind {
            NodeKind::COMPOUND => {
                let reference = editor.compounds().append(
                    CompoundId::from_node_id(parent),
                    root,
                    orientation()?,
                )?;
                if reference.is_valid() {
                    editor
                        .generic()
                        .set_child_ref_local_location(reference, shape.location())?;
                }
                reference.into()
            }
            NodeKind::SHELL => {
                let (shell, face) = (ShellId::from_node_id(parent), FaceId::from_node_id(root));
                editor.shells().append(shell, face, orientation()?)?.into()
            }
            NodeKind::SOLID if root.kind == NodeKind::SHELL => {
                let (solid, shell) = (SolidId::from_node_id(parent), ShellId::from_node_id(root));
                editor.solids().append(solid, shell, orientation()?)?.into()
            }
            NodeKind::COMP_SOLID if root.kind == NodeKind::SOLID => {
                let (comp_solid, solid) = (
                    CompSolidId::from_node_id(parent),
                    SolidId::from_node_id(root),
                );
                editor
                    .comp_solids()
                    .append(comp_solid, solid, orientation()?)?
                    .into()
            }
            _ => return Ok(()),
        };
        if inserted.is_valid() {
            result.inserted_ref = inserted;
            result.status = AddStatus::Success;
        }
        Ok(())
    }

    // The Product case: a child product of the root, placed by an occurrence under the
    // parent product at the shape location.
    fn reference_from_product(
        &mut self,
        shape: &Shape,
        parent: ProductId,
        result: &mut AddResult,
    ) -> Result<(), Failure> {
        let mut editor = self.graph.editor();
        let child = editor
            .products()
            .add(result.topology_root, &Location::default());
        if !child.is_valid() {
            return Ok(());
        }
        let (occurrence, reference) =
            editor
                .products()
                .append(parent, child, shape.location(), OccurrenceId::default())?;
        if !occurrence.is_valid() {
            return Ok(());
        }
        result.product = child;
        result.occurrence = occurrence;
        result.inserted_ref = reference.into();
        result.status = AddStatus::Success;
        Ok(())
    }

    /// `ClearCached(theNode)`: drops the cached reconstruction of an active node.
    pub fn clear_cached(&mut self, node: NodeId) {
        if !node.is_valid() || self.graph.topo().generic().is_removed(node) {
            return;
        }
        self.graph.inc_storage().unbind_current_shape(node);
    }

    /// `ClearCached(theRef)`: `ClearCached` of an active reference's child node.
    pub fn clear_cached_ref(&mut self, reference: RefId) {
        if !reference.is_valid() || self.graph.refs().generic().is_removed(reference) {
            return;
        }
        let child = self.graph.refs().generic().child_node(reference);
        self.clear_cached(child);
    }
}
