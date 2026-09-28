// Created on: 2016-06-20
// Created by: Denis BOGOLEPOV
// Copyright (c) 2016 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_BinaryTree.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_Tree<T, N, BVH_BinaryTree>`: node editing, children and SAH estimation.
use super::{BvhBinaryTree, BvhTree};
use crate::bvh::array;
use crate::bvh::bounding_box::BvhBox;
use crate::bvh::scalar::BvhScalar;
use crate::bvh::vector::{BvhVec4i, BvhVector};
use rustycad_tkernel::standard::Failure;

impl<V: BvhVector> BvhTree<V, BvhBinaryTree> {
    /// Sets node type to 'outer' (leaf); `tree.set_outer(0)?`.
    pub fn set_outer(&mut self, node: i32) -> Result<(), Failure> {
        self.node_info_mut(node)?.x = 1;
        Ok(())
    }
    /// Sets node type to 'inner'; `tree.set_inner(0)?`.
    pub fn set_inner(&mut self, node: i32) -> Result<(), Failure> {
        self.node_info_mut(node)?.x = 0;
        Ok(())
    }
    /// Returns index of the K-th child (record component K + 1); `tree.child::<0>(root)?`.
    ///
    /// # Panics
    /// When `K > 2`: the upstream component access has no fourth-plus-one slot.
    pub fn child<const K: usize>(&self, node: i32) -> Result<i32, Failure> {
        Ok(self.node_info(node)?[K + 1])
    }
    /// Mutable K-th child, as `ChangeChild<K>`; `*tree.child_mut::<0>(root)? = leaf`.
    pub fn child_mut<const K: usize>(&mut self, node: i32) -> Result<&mut i32, Failure> {
        Ok(&mut self.node_info_mut(node)?[K + 1])
    }
    /// Removes all nodes and resets depth; `tree.clear()`.
    pub fn clear(&mut self) {
        self.depth = 0;
        array::clear(&mut self.min_point_buffer);
        array::clear(&mut self.max_point_buffer);
        array::clear(&mut self.node_info_buffer);
    }
    /// Reserves storage for the given number of nodes; `tree.reserve(100)?`.
    pub fn reserve(&mut self, node_count: i32) -> Result<(), Failure> {
        array::reserve(&mut self.min_point_buffer, node_count)?;
        array::reserve(&mut self.max_point_buffer, node_count)?;
        array::reserve(&mut self.node_info_buffer, node_count)
    }
    /// Adds new leaf node and returns its index; `tree.add_leaf_node(min, max, 0, 10)`.
    pub fn add_leaf_node(&mut self, min: V, max: V, begin: i32, end: i32) -> i32 {
        array::append(&mut self.min_point_buffer, min);
        array::append(&mut self.max_point_buffer, max);
        self.add_leaf_node_uninit(begin, end)
    }
    /// Adds new inner node and returns its index; `tree.add_inner_node(min, max, left, right)`.
    pub fn add_inner_node(&mut self, min: V, max: V, left: i32, right: i32) -> i32 {
        array::append(&mut self.min_point_buffer, min);
        array::append(&mut self.max_point_buffer, max);
        self.add_inner_node_uninit(left, right)
    }
    /// Adds a leaf with the box corners; `tree.add_leaf_node_box(&bounding, 0, 5)`.
    pub fn add_leaf_node_box(&mut self, bounding: &BvhBox<V>, begin: i32, end: i32) -> i32 {
        self.add_leaf_node(*bounding.corner_min(), *bounding.corner_max(), begin, end)
    }
    /// Adds an inner node with the box corners; `tree.add_inner_node_box(&bounding, l, r)`.
    pub fn add_inner_node_box(&mut self, bounding: &BvhBox<V>, left: i32, right: i32) -> i32 {
        self.add_inner_node(*bounding.corner_min(), *bounding.corner_max(), left, right)
    }
    /// Adds only the leaf record; the point buffers are NOT extended (UNINITIALIZED
    /// bounds upstream). Example: `tree.add_leaf_node_uninit(0, 5)`.
    pub fn add_leaf_node_uninit(&mut self, begin: i32, end: i32) -> i32 {
        array::append(&mut self.node_info_buffer, BvhVec4i::new(1, begin, end, 0));
        array::size(&self.node_info_buffer) - 1
    }
    /// Adds only the inner record; the point buffers are NOT extended.
    /// Example: `tree.add_inner_node_uninit(1, 2)`.
    pub fn add_inner_node_uninit(&mut self, left: i32, right: i32) -> i32 {
        array::append(&mut self.node_info_buffer, BvhVec4i::new(0, left, right, 0));
        array::size(&self.node_info_buffer) - 1
    }
    /// Returns the surface area heuristic (SAH) cost of the tree rooted at node 0.
    /// Example: `tree.estimate_sah()? > 0.0`.
    pub fn estimate_sah(&self) -> Result<V::Scalar, Failure> {
        let mut sah = V::Scalar::ZERO;
        self.estimate_node_sah(0, V::Scalar::ONE, &mut sah)?;
        Ok(sah)
    }
    /// `BVH::EstimateSAH`: accumulates `prob * cost`, splitting the probability
    /// by child/parent area ratios while it stays positive.
    fn estimate_node_sah(
        &self,
        node: i32,
        probability: V::Scalar,
        sah: &mut V::Scalar,
    ) -> Result<(), Failure> {
        let bounding = self.node_box(node)?;
        if self.is_outer(node)? {
            let count = self.end_primitive(node)? - self.beg_primitive(node)? + 1;
            *sah += probability * V::Scalar::from_int(count);
            return Ok(());
        }
        *sah += probability * V::Scalar::from_double(2.0);
        let left = self.child::<0>(node)?;
        let left_box = self.node_box(left)?;
        if probability > V::Scalar::ZERO {
            let ratio = probability * left_box.area() / bounding.area();
            self.estimate_node_sah(left, ratio, sah)?;
        }
        let right = self.child::<1>(node)?;
        let right_box = self.node_box(right)?;
        if probability > V::Scalar::ZERO {
            let ratio = probability * right_box.area() / bounding.area();
            self.estimate_node_sah(right, ratio, sah)?;
        }
        Ok(())
    }
    fn node_box(&self, node: i32) -> Result<BvhBox<V>, Failure> {
        Ok(BvhBox::from_corners(
            *self.min_point(node)?,
            *self.max_point(node)?,
        ))
    }
}
