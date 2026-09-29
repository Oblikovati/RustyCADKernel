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
// Rust translation of the shape-binding and shape-cache members of
// ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Storage.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The bindings from source shapes to nodes, the construction-time originals of the
//! nodes, and the generation-validated reconstruction cache.
use super::Storage;
use super::tags::NodeTag;
use crate::brep_graph::{NodeId, NodeVisitor, TypedId};
use crate::topods::Shape;
use crate::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::DataMap;
use std::collections::{BTreeMap, HashMap};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// `BRepGraphInc_Storage::CachedShape`: a reconstructed shape and the SubtreeGen it was
/// built at.
#[derive(Clone, Debug, Default)]
pub struct CachedShape {
    /// `Shape`.
    pub shape: Shape,
    /// `StoredSubtreeGen`.
    pub stored_subtree_gen: u32,
}

/// The two binding maps behind `myShapeBindingsMutex`: shape key (TShape and location,
/// orientation ignored) to node, and node to its construction-time shape key.
#[derive(Default)]
pub struct ShapeBindings {
    pub(crate) shape_to_node: DataMap<Shape, NodeId, ShapeMapHasher>,
    pub(crate) originals: BTreeMap<NodeId, Shape>,
}

impl Storage {
    pub(crate) fn bindings(&self) -> RwLockReadGuard<'_, ShapeBindings> {
        self.shape_bindings.read().expect("shape bindings lock")
    }

    pub(crate) fn bindings_mut(&mut self) -> &mut ShapeBindings {
        self.shape_bindings.get_mut().expect("shape bindings lock")
    }

    fn bindings_write(&self) -> RwLockWriteGuard<'_, ShapeBindings> {
        self.shape_bindings.write().expect("shape bindings lock")
    }

    /// `FindDefinitionByShape(theShape)`: the active node bound to the shape key.
    pub fn find_definition_by_shape(&self, shape: &Shape) -> NodeId {
        let bound = self.bindings().shape_to_node.seek(shape).copied();
        match bound {
            Some(node) if node.is_valid() && node.visit(NotRemoved(self)) => node,
            _ => NodeId::default(),
        }
    }

    /// `HasShapeBinding(theShape)`.
    pub fn has_shape_binding(&self, shape: &Shape) -> bool {
        self.find_definition_by_shape(shape).is_valid()
    }

    /// `SetDefinitionShapeBinding(theShape, theNodeId)`: binds or rebinds.
    pub fn set_definition_shape_binding(&self, shape: &Shape, node: NodeId) {
        let mut bindings = self.bindings_write();
        if let Some(bound) = bindings.shape_to_node.change_seek(shape) {
            *bound = node;
            return;
        }
        bindings.shape_to_node.bind(shape.clone(), node);
    }

    /// `RemoveDefinitionShapeBinding(theShape, theExpectedNodeId)`: unbinds only when
    /// the key names the expected node.
    pub fn remove_definition_shape_binding(&self, shape: &Shape, expected: NodeId) -> bool {
        let mut bindings = self.bindings_write();
        if bindings.shape_to_node.seek(shape) != Some(&expected) {
            return false;
        }
        bindings.shape_to_node.unbind(shape);
        true
    }

    /// `FindOriginal(theNodeId)`: the construction-time shape, null when absent.
    pub fn find_original(&self, node: NodeId) -> Shape {
        self.bindings()
            .originals
            .get(&node)
            .cloned()
            .unwrap_or_default()
    }

    /// `HasOriginal(theNodeId)`.
    pub fn has_original(&self, node: NodeId) -> bool {
        self.bindings().originals.contains_key(&node)
    }

    /// `BindOriginal(theNodeId, theShape)`: binds or rebinds.
    pub fn bind_original(&self, node: NodeId, shape: &Shape) {
        self.bindings_write().originals.insert(node, shape.clone());
    }

    /// `UnBindOriginal(theNodeId)`.
    pub fn unbind_original(&self, node: NodeId) {
        self.bindings_write().originals.remove(&node);
    }

    /// `ForEachShapeBinding(theFunc)`: every shape key and its node, in map order.
    pub fn for_each_shape_binding(&self, mut function: impl FnMut(&Shape, NodeId)) {
        let bindings = self.bindings();
        for (shape, node) in bindings.shape_to_node.items() {
            function(shape, *node);
        }
    }

    /// `ForEachOriginalBinding(theFunc)`: every node and its original, in node order.
    pub fn for_each_original_binding(&self, mut function: impl FnMut(NodeId, &Shape)) {
        let bindings = self.bindings();
        for (node, shape) in &bindings.originals {
            function(*node, shape);
        }
    }

    /// `CopyShapeBindingsFrom(theSource)`: replaces both maps by copies of the source's.
    pub fn copy_shape_bindings_from(&mut self, source: &Storage) {
        let mut shapes = Vec::new();
        source.for_each_shape_binding(|shape, node| shapes.push((shape.clone(), node)));
        let originals = source.bindings().originals.clone();
        let bindings = self.bindings_mut();
        bindings.shape_to_node.clear(false);
        for (shape, node) in shapes {
            bindings.shape_to_node.bind(shape, node);
        }
        bindings.originals = originals;
    }

    /// `CurrentShapes()`: the reconstruction cache, read-locked.
    pub fn current_shapes(&self) -> RwLockReadGuard<'_, HashMap<NodeId, CachedShape>> {
        self.current_shapes.read().expect("shape cache lock")
    }

    /// `ChangeCurrentShapes()`: the reconstruction cache, write-locked.
    pub fn change_current_shapes(&self) -> RwLockWriteGuard<'_, HashMap<NodeId, CachedShape>> {
        self.current_shapes.write().expect("shape cache lock")
    }

    /// `CurrentShapesMutex()`: the lock of the reconstruction cache.
    pub fn current_shapes_mutex(&self) -> &RwLock<HashMap<NodeId, CachedShape>> {
        &self.current_shapes
    }

    /// `ClearCurrentShapes()`.
    pub fn clear_current_shapes(&self) {
        self.clear_shape_cache();
    }

    /// `ClearShapeCache()` (private natively).
    pub(crate) fn clear_shape_cache(&self) {
        *self.change_current_shapes() = HashMap::new();
    }

    /// `UnbindCurrentShape(theNode)`.
    pub fn unbind_current_shape(&self, node: NodeId) {
        self.change_current_shapes().remove(&node);
    }

    // The binding part of `ClearStorageForReuse()`.
    pub(crate) fn clear_shape_bindings(&mut self) {
        let bindings = self.bindings_mut();
        bindings.shape_to_node.clear(false);
        bindings.originals.clear();
    }
}

// The per-kind check of `FindDefinitionByShape`: not removed.
struct NotRemoved<'s>(&'s Storage);

impl NodeVisitor for NotRemoved<'_> {
    type Output = bool;
    fn visit<T: NodeTag>(self, id: TypedId<T>) -> bool {
        !self.0.is_removed(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph::NodeKind;
    use crate::topods::{Builder, Compound};

    fn compound() -> Shape {
        let mut shape = Compound::default();
        Builder.make_compound(&mut shape);
        shape.into()
    }

    #[test]
    fn shape_bindings_follow_is_same() {
        let mut storage = Storage::new();
        let node = NodeId::from(storage.append_compound());
        let shape = compound();
        storage.set_definition_shape_binding(&shape, node);
        assert_eq!(storage.find_definition_by_shape(&shape.reversed()), node);
        assert!(!storage.remove_definition_shape_binding(&shape, NodeId::start(NodeKind::FACE)));
        storage.bind_original(node, &shape);
        assert!(storage.has_original(node));
        let mut copy = Storage::new();
        copy.copy_shape_bindings_from(&storage);
        assert_eq!(copy.find_original(node), shape);
        assert!(storage.mark_removed(node));
        assert!(!storage.has_original(node));
        assert!(!storage.has_shape_binding(&shape));
        storage.unbind_original(node);
    }

    #[test]
    fn shape_cache_binds_and_clears() {
        let storage = Storage::new();
        let node = NodeId::start(NodeKind::VERTEX);
        storage
            .change_current_shapes()
            .insert(node, CachedShape::default());
        assert!(storage.current_shapes().contains_key(&node));
        storage.unbind_current_shape(node);
        assert!(storage.current_shapes_mutex().read().unwrap().is_empty());
        storage
            .change_current_shapes()
            .insert(node, CachedShape::default());
        storage.clear_current_shapes();
        assert!(storage.current_shapes().is_empty());
    }
}
