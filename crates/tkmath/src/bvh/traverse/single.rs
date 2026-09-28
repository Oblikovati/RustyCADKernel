// Created by: Eugeny MALTCHIKOV
// Created on: 2019-04-17
// Copyright (c) 2019 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_Traverse.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_Traverse::Select`: stack-based descent of one binary tree.
use super::{BvhBaseTraverse, TraverseStack, pop_surviving};
use crate::bvh::array;
use crate::bvh::constants::MAX_TREE_DEPTH;
use crate::bvh::tree::BvhTree;
use crate::bvh::tree_owner::BvhTreeOwner;
use crate::bvh::vector::{BvhVec4i, BvhVector};
use rustycad_tkernel::standard::Failure;

/// `BVH_NodeInStack`: a node to process with the metric computed for it.
/// Example: `BvhNodeInStack::new(0, 0.0_f64)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BvhNodeInStack<M> {
    /// Id of the node in the BVH tree.
    pub node_id: i32,
    /// Metric computed for the node.
    pub metric: M,
}
impl<M> BvhNodeInStack<M> {
    /// Creates a stack entry; `BvhNodeInStack::new(3, metric)`.
    pub const fn new(node_id: i32, metric: M) -> Self {
        Self { node_id, metric }
    }
}

/// Selection of elements from one binary tree (`BVH_Traverse`).
///
/// `M` is the metric type (`MetricType`, by default the scalar `T`). Metrics
/// the selector does not write start as `M::default()`; upstream leaves them
/// uninitialized. `Accept` may fail, as its upstream body may throw.
/// Example: `let count = selector.select_tree(Some(&tree))?;`.
pub trait BvhTraverse<V: BvhVector, M: Copy + Default = <V as BvhVector>::Scalar>:
    BvhBaseTraverse<M>
{
    /// True when the whole branch may be accepted by its metric (default: false).
    fn accept_metric(&self, _metric: &M) -> bool {
        false
    }
    /// Rejection of a node by its box; writes the node metric.
    fn reject_node(&self, corner_min: &V, corner_max: &V, metric: &mut M) -> bool;
    /// Leaf element acceptance with the metric of its leaf node.
    fn accept(&mut self, index: i32, metric: &M) -> Result<bool, Failure>;
    /// `Select(theBVH)`: returns the number of accepted elements; `None` stands
    /// for a null handle. Example: `selector.select_tree(Some(&tree))?`.
    fn select_tree(&mut self, bvh: Option<&BvhTree<V>>) -> Result<i32, Failure>
    where
        Self: Sized,
    {
        select_elements(self, bvh)
    }
}

/// `BVH_Traverse::Select()`: selection through the set that owns the tree
/// (`myBVHSet`); a selector without a set selects nothing.
/// Example: `distance_tool.select()?`.
pub trait BvhSetTraverse<V: BvhVector, M: Copy + Default = <V as BvhVector>::Scalar>:
    BvhTraverse<V, M>
{
    /// The set containing the BVH tree (`BVHSetType`).
    type Set: BvhTreeOwner<V> + ?Sized;
    /// The set given to `SetBVHSet`, or `None` for a null pointer.
    fn bvh_set_mut(&mut self) -> Option<&mut Self::Set>;
    /// Builds the set tree when dirty (`BVH()`), then selects from it.
    fn select(&mut self) -> Result<i32, Failure>
    where
        Self: Sized,
    {
        let Some(set) = self.bvh_set_mut() else {
            return Ok(0);
        };
        let bvh = set.take_built_bvh()?;
        let selected = select_elements(self, Some(&bvh));
        if let Some(set) = self.bvh_set_mut() {
            set.restore_bvh(bvh);
        }
        selected
    }
}

/// Whether the descent goes on after a node was processed.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Flow {
    Continue,
    Stop,
}

/// Traversal state: the current node, the pending nodes and the counter.
struct SingleWalk<M> {
    node: BvhNodeInStack<M>,
    stack: TraverseStack<BvhNodeInStack<M>>,
    accepted: i32,
}

/// `BVH_Traverse::Select(theBVH)`: descends from the root, testing both children
/// of an inner node and applying `Accept` to every element of a reached leaf.
///
/// The root is never tested by `RejectNode`. When both children pass, the one
/// with the better metric is processed first and the other is stacked.
/// Example: `select_elements(&mut selector, Some(&tree))?`.
pub fn select_elements<V, M, T>(traverse: &mut T, bvh: Option<&BvhTree<V>>) -> Result<i32, Failure>
where
    V: BvhVector,
    M: Copy + Default,
    T: BvhTraverse<V, M> + ?Sized,
{
    let Some(bvh) = bvh else { return Ok(0) };
    if bvh.length() == 0 {
        return Ok(0);
    }
    let mut walk = SingleWalk {
        node: BvhNodeInStack::new(0, M::default()),
        stack: TraverseStack::new(MAX_TREE_DEPTH as usize, "Error! BVH stack overflow"),
        accepted: 0,
    };
    let mut previous = walk.node.node_id;
    loop {
        if walk.visit(traverse, bvh)? == Flow::Stop || !walk.advance(traverse, previous) {
            return Ok(walk.accepted);
        }
        previous = walk.node.node_id;
    }
}

impl<M: Copy + Default> SingleWalk<M> {
    fn visit<V: BvhVector, T: BvhTraverse<V, M> + ?Sized>(
        &mut self,
        traverse: &mut T,
        bvh: &BvhTree<V>,
    ) -> Result<Flow, Failure> {
        let data = *array::value(bvh.node_info_buffer(), self.node.node_id)?;
        if data.x != 0 {
            return self.visit_leaf(traverse, &data);
        }
        if traverse.accept_metric(&self.node.metric) {
            // Both children will be accepted: take the left one, stack the right one.
            self.stack.ensure_room()?;
            self.node = BvhNodeInStack::new(data.y, self.node.metric);
            self.stack
                .push(BvhNodeInStack::new(data.z, self.node.metric));
            return Ok(Flow::Continue);
        }
        self.visit_children(traverse, bvh, &data)
    }

    /// Tests both children, stopping as soon as `Stop` asks for it.
    fn visit_children<V: BvhVector, T: BvhTraverse<V, M> + ?Sized>(
        &mut self,
        traverse: &T,
        bvh: &BvhTree<V>,
        data: &BvhVec4i,
    ) -> Result<Flow, Failure> {
        let mut left_metric = M::default();
        let (left_min, left_max) = (bvh.min_point(data.y)?, bvh.max_point(data.y)?);
        let is_good_left = !traverse.reject_node(left_min, left_max, &mut left_metric);
        if traverse.stop() {
            return Ok(Flow::Stop);
        }
        let mut right_metric = M::default();
        let (right_min, right_max) = (bvh.min_point(data.z)?, bvh.max_point(data.z)?);
        let is_good_right = !traverse.reject_node(right_min, right_max, &mut right_metric);
        if traverse.stop() {
            return Ok(Flow::Stop);
        }
        let left = BvhNodeInStack::new(data.y, left_metric);
        let right = BvhNodeInStack::new(data.z, right_metric);
        self.choose(traverse, (left, is_good_left), (right, is_good_right))?;
        Ok(Flow::Continue)
    }

    /// Processes the better good child next and stacks the other one.
    fn choose<T: BvhBaseTraverse<M> + ?Sized>(
        &mut self,
        traverse: &T,
        (left, is_good_left): (BvhNodeInStack<M>, bool),
        (right, is_good_right): (BvhNodeInStack<M>, bool),
    ) -> Result<(), Failure> {
        if is_good_left && is_good_right {
            self.stack.ensure_room()?;
            let (next, later) = if traverse.is_metric_better(&left.metric, &right.metric) {
                (left, right)
            } else {
                (right, left)
            };
            self.node = next;
            self.stack.push(later);
        } else if is_good_left || is_good_right {
            self.node = if is_good_left { left } else { right };
        }
        Ok(())
    }

    /// Applies `Accept` to every element of the leaf, checking `Stop` after each.
    fn visit_leaf<V: BvhVector, T: BvhTraverse<V, M> + ?Sized>(
        &mut self,
        traverse: &mut T,
        data: &BvhVec4i,
    ) -> Result<Flow, Failure> {
        for index in data.y..=data.z {
            if traverse.accept(index, &self.node.metric)? {
                self.accepted += 1;
            }
            if traverse.stop() {
                return Ok(Flow::Stop);
            }
        }
        Ok(Flow::Continue)
    }

    /// When the node did not change, resumes from the stack, skipping entries
    /// that `RejectMetric` prunes; false when nothing is left.
    fn advance<T: BvhBaseTraverse<M> + ?Sized>(&mut self, traverse: &T, previous: i32) -> bool {
        if self.node.node_id != previous {
            return true;
        }
        match pop_surviving(&mut self.stack, traverse, |entry| &entry.metric) {
            Some(entry) => {
                self.node = entry;
                true
            }
            None => false,
        }
    }
}
