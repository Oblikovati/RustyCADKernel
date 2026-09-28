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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_Builder.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_BuilderTransient` parameters and the abstract `BVH_Builder<T, N>`.
use super::bounding_box::BvhBox;
use super::set::BvhSet;
use super::tree::BvhTree;
use super::vector::BvhVector;
use rustycad_tkernel::standard::Failure;
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, Ordering};

/// Data of `BVH_BuilderTransient`: depth and leaf limits plus the parallel flag.
///
/// Builders are shared through handles upstream, so the flag is atomic and can
/// be changed through a shared reference. Example: `BvhBuilderTransient::new(5, 32)`.
#[derive(Debug)]
pub struct BvhBuilderTransient {
    max_tree_depth: i32,
    leaf_node_size: i32,
    is_parallel: AtomicBool,
}
impl BvhBuilderTransient {
    /// Stores the limits; parallel execution starts disabled; `BvhBuilderTransient::new(5, 32)`.
    pub fn new(leaf_node_size: i32, max_tree_depth: i32) -> Self {
        Self {
            max_tree_depth,
            leaf_node_size,
            is_parallel: AtomicBool::new(false),
        }
    }
    /// Returns the maximum depth of constructed BVH; `params.max_tree_depth()`.
    pub fn max_tree_depth(&self) -> i32 {
        self.max_tree_depth
    }
    /// Returns the maximum number of sub-elements in the leaf; `params.leaf_node_size()`.
    pub fn leaf_node_size(&self) -> i32 {
        self.leaf_node_size
    }
    /// Returns parallel flag; `params.is_parallel()`.
    pub fn is_parallel(&self) -> bool {
        self.is_parallel.load(Ordering::Relaxed)
    }
    /// Sets the flag controlling possibility of parallel execution; `params.set_parallel(true)`.
    pub fn set_parallel(&self, is_parallel: bool) {
        self.is_parallel.store(is_parallel, Ordering::Relaxed);
    }
}

/// Constructs a BVH tree from the AABBs of a set of objects.
/// Example: `builder.build(&mut set, &mut tree, &set.bounding_box()?)?`.
pub trait BvhBuilder<V: BvhVector>: Debug + Send + Sync {
    /// Borrows the `BVH_BuilderTransient` parameters.
    fn builder_transient(&self) -> &BvhBuilderTransient;
    /// Builds BVH using specific algorithm (the pure virtual `Build`).
    fn build(
        &self,
        set: &mut dyn BvhSet<V>,
        bvh: &mut BvhTree<V>,
        bounding: &BvhBox<V>,
    ) -> Result<(), Failure>;
}

/// `BVH_Builder3d`: the builder interface of double-precision 3D BVHs.
pub type BvhBuilder3d = dyn BvhBuilder<crate::bvh::BvhVec3d>;

/// `BVH_Builder::updateDepth`: raises the tree depth to `level` when deeper.
/// Example: `update_depth(&mut tree, 3)`.
pub fn update_depth<V: BvhVector>(bvh: &mut BvhTree<V>, level: i32) {
    if level > bvh.depth() {
        *bvh.depth_mut() = level;
    }
}
