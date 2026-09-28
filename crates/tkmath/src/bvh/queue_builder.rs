// Created on: 2015-05-28
// Created by: Denis BOGOLEPOV
// Copyright (c) 2015 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_QueueBuilder.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_QueueBuilder<T, N>`: builders that split nodes taken from a work queue.
use super::bounding_box::BvhBox;
use super::build_queue::BvhBuildQueue;
use super::builder::{BvhBuilderTransient, update_depth};
use super::set::BvhSet;
use super::tree::BvhTree;
use super::vector::BvhVector;
use rustycad_tkernel::standard::Failure;
mod build;
pub use build::queue_build;

/// Range of primitives belonging to a BVH node (`BVH_PrimitiveRange`).
/// Example: `BvhPrimitiveRange::new(0, 9).size() == 10`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BvhPrimitiveRange {
    /// First primitive (`Start`); -1 when uninitialized.
    pub start: i32,
    /// Last primitive, inclusive (`Final`).
    pub end: i32,
}
impl Default for BvhPrimitiveRange {
    fn default() -> Self {
        Self::new(-1, -1)
    }
}
impl BvhPrimitiveRange {
    /// Creates new primitive range; `BvhPrimitiveRange::new(0, 4)`.
    pub const fn new(start: i32, end: i32) -> Self {
        Self { start, end }
    }
    /// Returns total number of primitives; `range.size()`.
    pub const fn size(&self) -> i32 {
        self.end - self.start + 1
    }
    /// Checks if the range is initialized (`Start != -1`); `range.is_valid()`.
    pub const fn is_valid(&self) -> bool {
        self.start != -1
    }
}

/// Parameters of the two child nodes produced by a split (`BVH_ChildNodes`).
/// The default value is invalid and means "do not split".
/// Example: `BvhChildNodes::<Vec3<f64>>::default().is_valid() == false`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BvhChildNodes<V: BvhVector> {
    /// Bounding boxes of child nodes.
    pub boxes: [BvhBox<V>; 2],
    /// Primitive ranges of child nodes.
    pub ranges: [BvhPrimitiveRange; 2],
}
impl<V: BvhVector> Default for BvhChildNodes<V> {
    fn default() -> Self {
        Self {
            boxes: [BvhBox::default(); 2],
            ranges: [BvhPrimitiveRange::default(); 2],
        }
    }
}
impl<V: BvhVector> BvhChildNodes<V> {
    /// Creates new parameters of BVH child nodes; `BvhChildNodes::new(l, r, lr, rr)`.
    pub fn new(
        left_box: BvhBox<V>,
        right_box: BvhBox<V>,
        left_range: BvhPrimitiveRange,
        right_range: BvhPrimitiveRange,
    ) -> Self {
        Self {
            boxes: [left_box, right_box],
            ranges: [left_range, right_range],
        }
    }
    /// Returns number of primitives in the given child (0 or 1); `children.nb_prims(0)`.
    pub fn nb_prims(&self, child: usize) -> i32 {
        self.ranges[child].size()
    }
    /// Checks if both ranges are initialized; `children.is_valid()`.
    pub fn is_valid(&self) -> bool {
        self.ranges[0].is_valid() && self.ranges[1].is_valid()
    }
}

/// The virtual interface of `BVH_QueueBuilder`: a node split and the child
/// registration. Implementors provide `BvhBuilder::build` through [`queue_build`].
/// Example: `impl BvhQueueBuilder<V> for MySplitter { ... }`.
pub trait BvhQueueBuilder<V: BvhVector>: Send + Sync {
    /// Borrows the `BVH_BuilderTransient` parameters.
    fn queue_builder_transient(&self) -> &BvhBuilderTransient;
    /// Number of threads used to build BVH (`myNumOfThreads`).
    fn num_of_threads(&self) -> i32;
    /// Performs splitting of the given BVH node (pure virtual `buildNode`).
    fn build_node(
        &self,
        set: &mut dyn BvhSet<V>,
        bvh: &mut BvhTree<V>,
        node: i32,
    ) -> Result<BvhChildNodes<V>, Failure>;
    /// Processes child nodes of the split node (virtual `addChildren`).
    fn add_children(
        &self,
        bvh: &mut BvhTree<V>,
        queue: &BvhBuildQueue,
        node: i32,
        children: &BvhChildNodes<V>,
    ) -> Result<(), Failure> {
        add_queued_children(self.queue_builder_transient(), bvh, queue, node, children)
    }
}

/// The default `addChildren`: appends both children as leaves under the queue
/// lock, links them to `node`, and enqueues the ones that still need splitting.
/// Example: `add_queued_children(&params, &mut tree, &queue, 0, &children)?`.
pub fn add_queued_children<V: BvhVector>(
    params: &BvhBuilderTransient,
    bvh: &mut BvhTree<V>,
    queue: &BvhBuildQueue,
    node: i32,
    children: &BvhChildNodes<V>,
) -> Result<(), Failure> {
    if !children.is_valid() {
        return Ok(());
    }
    let child_nodes = {
        let _lock = queue.lock_nodes();
        let nodes = children
            .boxes
            .iter()
            .zip(children.ranges)
            .map(|(bounding, range)| bvh.add_leaf_node_box(bounding, range.start, range.end))
            .collect::<Vec<i32>>();
        update_depth(bvh, bvh.level(node)? + 1);
        nodes
    };
    for (index, child) in child_nodes.into_iter().enumerate() {
        link_child(bvh, node, index, child)?;
        let is_leaf = children.nb_prims(index) <= params.leaf_node_size()
            || bvh.level(child)? >= params.max_tree_depth();
        if !is_leaf {
            queue.enqueue(child);
        }
    }
    Ok(())
}

fn link_child<V: BvhVector>(
    bvh: &mut BvhTree<V>,
    node: i32,
    index: usize,
    child: i32,
) -> Result<(), Failure> {
    *bvh.level_mut(child)? = bvh.level(node)? + 1;
    let slot = if index == 0 {
        bvh.child_mut::<0>(node)?
    } else {
        bvh.child_mut::<1>(node)?
    };
    *slot = child;
    Ok(())
}
