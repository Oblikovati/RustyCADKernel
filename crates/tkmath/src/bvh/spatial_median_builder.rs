// Created on: 2013-12-20
// Created by: Denis BOGOLEPOV
// Copyright (c) 2013-2014 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_SpatialMedianBuilder.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_SpatialMedianBuilder<T, N>`: the binned builder with two bins.
use super::binned_builder::BvhBinnedBuilder;
use super::bounding_box::BvhBox;
use super::build_queue::BvhBuildQueue;
use super::builder::{BvhBuilder, BvhBuilderTransient};
use super::constants::{LEAF_NODE_SIZE_DEFAULT, MAX_TREE_DEPTH};
use super::queue_builder::{BvhChildNodes, BvhQueueBuilder};
use super::set::BvhSet;
use super::tree::BvhTree;
use super::vector::BvhVector;
use rustycad_tkernel::standard::Failure;

/// Spatial median split: `BVH_BinnedBuilder<T, N, 2>` on one thread, so every
/// split plane is the middle of the node box (or the index middle when all
/// centroids fall into one half).
/// Example: `BvhSpatialMedianBuilder::<BvhVec3d>::new(5, 32, false)`.
#[derive(Debug)]
pub struct BvhSpatialMedianBuilder<V: BvhVector> {
    binned: BvhBinnedBuilder<V, 2>,
}
impl<V: BvhVector> Default for BvhSpatialMedianBuilder<V> {
    fn default() -> Self {
        Self::new(LEAF_NODE_SIZE_DEFAULT, MAX_TREE_DEPTH, false)
    }
}
impl<V: BvhVector> BvhSpatialMedianBuilder<V> {
    /// Creates spatial median split builder; `use_main_axis` restricts splits to
    /// the widest axis. Example: `BvhSpatialMedianBuilder::<BvhVec3d>::new(5, 32, true)`.
    pub fn new(leaf_node_size: i32, max_tree_depth: i32, use_main_axis: bool) -> Self {
        Self {
            binned: BvhBinnedBuilder::new(leaf_node_size, max_tree_depth, use_main_axis, 1),
        }
    }
    /// The underlying two-bin builder (the C++ base class); `builder.binned()`.
    pub fn binned(&self) -> &BvhBinnedBuilder<V, 2> {
        &self.binned
    }
}
impl<V: BvhVector> BvhQueueBuilder<V> for BvhSpatialMedianBuilder<V> {
    fn queue_builder_transient(&self) -> &BvhBuilderTransient {
        self.binned.queue_builder_transient()
    }
    fn num_of_threads(&self) -> i32 {
        self.binned.num_of_threads()
    }
    fn build_node(
        &self,
        set: &mut dyn BvhSet<V>,
        bvh: &mut BvhTree<V>,
        node: i32,
    ) -> Result<BvhChildNodes<V>, Failure> {
        self.binned.build_node(set, bvh, node)
    }
    fn add_children(
        &self,
        bvh: &mut BvhTree<V>,
        queue: &BvhBuildQueue,
        node: i32,
        children: &BvhChildNodes<V>,
    ) -> Result<(), Failure> {
        self.binned.add_children(bvh, queue, node, children)
    }
}
impl<V: BvhVector> BvhBuilder<V> for BvhSpatialMedianBuilder<V> {
    fn builder_transient(&self) -> &BvhBuilderTransient {
        self.binned.builder_transient()
    }
    fn build(
        &self,
        set: &mut dyn BvhSet<V>,
        bvh: &mut BvhTree<V>,
        bounding: &BvhBox<V>,
    ) -> Result<(), Failure> {
        self.binned.build(set, bvh, bounding)
    }
}
