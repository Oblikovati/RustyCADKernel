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

use super::{BoundingNode, BoundingVolume, UBTree};
impl<T: Clone + Default, B: BoundingVolume> UBTree<T, B> {
    pub(in crate::ncollection) fn remove_leaf(&mut self, index: usize) -> Option<usize> {
        let Some(parent) = self.node(index).parent else {
            self.clear();
            return None;
        };
        let [first, second] = self.node(parent).children.unwrap();
        let sibling: usize = if first == index { second } else { first };
        let mut survivor: BoundingNode<T, B> = self.node(sibling).clone();
        survivor.parent = self.node(parent).parent;
        *self.node_mut(parent) = survivor;
        self.nodes[index] = None;
        self.nodes[sibling] = None;
        self.reparent_children(parent);
        self.update_ancestors(parent);
        self.node(parent).is_leaf().then_some(parent)
    }
    fn update_ancestors(&mut self, mut index: usize) {
        while let Some(parent) = self.node(index).parent {
            let [first, second] = self.node(parent).children.unwrap();
            let mut bounds: B = self.node(first).bounds.clone();
            bounds.add(&self.node(second).bounds);
            self.node_mut(parent).bounds = bounds;
            index = parent;
        }
    }
}
