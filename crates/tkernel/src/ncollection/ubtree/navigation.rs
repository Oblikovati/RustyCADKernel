// Created on: 2002-07-30
// Created by: Michael SAZONOV
// Copyright (c) 2002-2014 OPEN CASCADE SAS
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

//! Native `TreeNode` link accessors (`Child`, `Parent`, their `Change*` twins) over arena ids.
use super::{BoundingNode, BoundingVolume, UBTree};
impl<T: Clone + Default, B: BoundingVolume> UBTree<T, B> {
    /// The live node with arena id `id`; `None` for a released or unknown id.
    /// Example: `tree.node_by_id(tree.root_id().unwrap())`.
    pub fn node_by_id(&self, id: usize) -> Option<&BoundingNode<T, B>> {
        self.nodes.get(id).and_then(|slot| slot.as_deref())
    }
    /// Mutable live node with arena id `id`; `tree.change_node_by_id(id)`.
    pub fn change_node_by_id(&mut self, id: usize) -> Option<&mut BoundingNode<T, B>> {
        self.nodes.get_mut(id).and_then(|slot| slot.as_deref_mut())
    }
    /// Child `index` of `node`, as native `Child(i)`; `None` for a leaf, where the native
    /// call dereferences a null pointer. Example: `tree.child(tree.root().unwrap(), 0)`.
    pub fn child(&self, node: &BoundingNode<T, B>, index: i32) -> Option<&BoundingNode<T, B>> {
        self.node_by_id(node.child_id(index)?)
    }
    /// Parent of `node`, as native `Parent()`; `None` for the root. Example: `tree.parent(node)`.
    pub fn parent(&self, node: &BoundingNode<T, B>) -> Option<&BoundingNode<T, B>> {
        self.node_by_id(node.parent_id()?)
    }
    /// Mutable child `index` of node `id`, as native `ChangeChild(i)`; `tree.change_child(id, 1)`.
    pub fn change_child(&mut self, id: usize, index: i32) -> Option<&mut BoundingNode<T, B>> {
        let child: usize = self.node_by_id(id)?.child_id(index)?;
        self.change_node_by_id(child)
    }
    /// Mutable parent of node `id`, as native `ChangeParent()`; `tree.change_parent(id)`.
    pub fn change_parent(&mut self, id: usize) -> Option<&mut BoundingNode<T, B>> {
        let parent: usize = self.node_by_id(id)?.parent_id()?;
        self.change_node_by_id(parent)
    }
    /// Split node `id` into a branch, as native `TreeNode::Gemmate`: child 0 takes the node's
    /// previous content (and adopts its children), child 1 the new leaf; the node gets
    /// `new_bounds` and a default object. Returns the new leaf's id, `None` for a dead id.
    /// Example: `tree.gemmate(root, union, 7, bounds)`.
    pub fn gemmate(&mut self, id: usize, new_bounds: B, object: T, bounds: B) -> Option<usize> {
        let previous: BoundingNode<T, B> = self.node_by_id(id)?.clone();
        let first: usize = self.allocate_node(previous);
        self.node_mut(first).parent = Some(id);
        let second: usize = self.allocate_node(BoundingNode::new(object, bounds));
        self.node_mut(second).parent = Some(id);
        self.reparent_children(first);
        let node: &mut BoundingNode<T, B> = self.node_mut(id);
        node.children = Some([first, second]);
        node.bounds = new_bounds;
        // Native `myObject = TheObjType()` nullifies the branch object.
        node.object = T::default();
        Some(second)
    }
}
