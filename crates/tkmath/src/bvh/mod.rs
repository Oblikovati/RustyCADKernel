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

//! OCCT `BVH`: bounding volume hierarchies over boxes of NCollection vectors.
//!
//! The C++ templates `BVH_X<T, N>` become generics over one vector type
//! `V: BvhVector` (`Vec2/3/4<f32|f64>`); see `docs/bvh-design.md`.
//! Example: `let mut tree = BvhTree::<BvhVec3d>::default();`.
pub mod array;
mod binned_builder;
mod bounding_box;
mod build_queue;
mod build_thread;
mod builder;
pub mod constants;
mod object;
mod primitive_set;
mod properties;
mod queue_builder;
mod quick_sorter;
mod radix_sorter;
mod ray;
mod scalar;
mod set;
mod sorter;
pub mod tools;
mod tree;
mod triangulation;
mod vector;

pub use binned_builder::{BvhBin, BvhBinnedBuilder, BvhSplitPlane, main_axis, split_primitives};
pub use bounding_box::{BvhBox, cwise_std_max, cwise_std_min};
pub use build_queue::BvhBuildQueue;
pub use build_thread::{BvhBuildThread, BvhBuildTool};
pub use builder::{BvhBuilder, BvhBuilderTransient, update_depth};
pub use object::{BvhObject, BvhObjectTransient};
pub use primitive_set::{BvhBuilderHandle, BvhPrimitiveSet, BvhPrimitiveSetAccess};
pub use properties::{BvhProperties, BvhTransform};
pub use queue_builder::{
    BvhChildNodes, BvhPrimitiveRange, BvhQueueBuilder, add_queued_children, queue_build,
};
pub use quick_sorter::BvhQuickSorter;
pub use radix_sorter::{
    BitComparator, BitPredicate, BvhEncodedLink, BvhRadixSorter, THE_MORTON_LUT,
    encode_morton_code, sort_links,
};
pub use ray::BvhRay;
pub use scalar::{BvhScalar, int_floor, std_max, std_min};
pub use set::{BvhSet, combined_element_boxes};
pub use sorter::BvhSorter;
pub use tree::{BvhBinaryTree, BvhQuadTree, BvhTree};
pub use triangulation::BvhTriangulation;
pub use vector::{
    BvhArray2d, BvhArray2f, BvhArray2i, BvhArray3d, BvhArray3f, BvhArray3i, BvhArray4d, BvhArray4f,
    BvhArray4i, BvhArrayNt, BvhMat4d, BvhMat4f, BvhVec2d, BvhVec2f, BvhVec2i, BvhVec3d, BvhVec3f,
    BvhVec3i, BvhVec4d, BvhVec4f, BvhVec4i, BvhVecNt, BvhVector, Dimension, VectorType,
};
