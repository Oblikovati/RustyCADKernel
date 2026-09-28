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
mod box_set;
mod build_queue;
mod build_thread;
mod builder;
pub mod constants;
mod distance;
mod distance_field;
mod geometry;
mod linear_builder;
mod object;
mod object_set;
mod pair_distance;
mod primitive_set;
mod properties;
mod queue_builder;
mod quick_sorter;
mod radix_sorter;
mod ray;
mod scalar;
mod set;
mod sorter;
mod spatial_median_builder;
mod sweep_plane_builder;
pub mod tools;
mod traverse;
mod tree;
mod tree_owner;
mod triangulation;
mod vector;

pub use binned_builder::{
    BvhBin, BvhBinnedBuilder, BvhSplitPlane, get_sub_volumes, main_axis, split_primitives,
};
pub use bounding_box::{BvhBox, cwise_std_max, cwise_std_min};
pub use box_set::{BvhBoxSet, BvhIndexedBoxSet};
pub use build_queue::BvhBuildQueue;
pub use build_thread::{BvhBuildThread, BvhBuildTool};
pub use builder::{BvhBuilder, BvhBuilder3d, BvhBuilderTransient, update_depth};
pub use distance::{BvhDistance, BvhDistanceTraverse};
pub use distance_field::{
    BvhDistanceField, BvhFieldVector, BvhPointGeometrySquareDistance,
    BvhPointTriangulationSquareDistance, BvhSquareDistanceToPoint, direction_to_nearest_point,
    distance_to_box, square_distance_to_geometry, square_distance_to_object,
};
pub use geometry::BvhGeometry;
pub use linear_builder::{BvhLinearBuilder, BvhUpdateBoundTask, update_bounds};
pub use object::{BvhObject, BvhObjectTransient};
pub use object_set::{BvhObjectHandle, BvhObjectList, BvhObjectSet};
pub use pair_distance::{BvhPairDistance, BvhPairDistanceTraverse};
pub use primitive_set::{
    BvhBuilderHandle, BvhPrimitiveSet, BvhPrimitiveSet3d, BvhPrimitiveSetAccess,
};
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
pub use spatial_median_builder::BvhSpatialMedianBuilder;
pub use sweep_plane_builder::BvhSweepPlaneBuilder;
pub use traverse::{
    BvhBaseTraverse, BvhNodeInStack, BvhPairNodesInStack, BvhPairSetTraverse, BvhPairTraverse,
    BvhSetTraverse, BvhTraverse, select_elements, select_pairs,
};
pub use tree::{BvhBinaryTree, BvhQuadTree, BvhTree};
pub use tree_owner::BvhTreeOwner;
pub use triangulation::BvhTriangulation;
pub use vector::{
    BvhArray2d, BvhArray2f, BvhArray2i, BvhArray3d, BvhArray3f, BvhArray3i, BvhArray4d, BvhArray4f,
    BvhArray4i, BvhArrayNt, BvhMat4d, BvhMat4f, BvhVec2d, BvhVec2f, BvhVec2i, BvhVec3d, BvhVec3f,
    BvhVec3i, BvhVec4d, BvhVec4f, BvhVec4i, BvhVecNt, BvhVector, Dimension, VectorType,
};
