// Created on: 2014-01-09
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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_SweepPlaneBuilder.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_SweepPlaneBuilder<T, N>`: full sweep SAH over sorted centroids.
use super::bounding_box::BvhBox;
use super::builder::{BvhBuilder, BvhBuilderTransient};
use super::constants::{LEAF_NODE_SIZE_DEFAULT, MAX_TREE_DEPTH, THE_NODE_MIN_SIZE};
use super::queue_builder::{BvhChildNodes, BvhPrimitiveRange, BvhQueueBuilder, queue_build};
use super::quick_sorter::BvhQuickSorter;
use super::scalar::BvhScalar;
use super::set::BvhSet;
use super::sorter::BvhSorter;
use super::tree::BvhTree;
use super::vector::BvhVector;
use rustycad_tkernel::standard::Failure;
use std::marker::PhantomData;

/// Sweep plane SAH builder: for each axis the node primitives are sorted by
/// centroid and every split position between them is costed.
/// Example: `BvhSweepPlaneBuilder::<BvhVec3d>::new(5, 32, 1)`.
#[derive(Debug)]
pub struct BvhSweepPlaneBuilder<V: BvhVector> {
    transient: BvhBuilderTransient,
    num_of_threads: i32,
    vector: PhantomData<V>,
}
impl<V: BvhVector> Default for BvhSweepPlaneBuilder<V> {
    fn default() -> Self {
        Self::new(LEAF_NODE_SIZE_DEFAULT, MAX_TREE_DEPTH, 1)
    }
}

/// The best split so far: `aMinSplitAxis`, `aMinSplitIndex`, `aMinSplitCost`.
struct SweepSplit {
    axis: i32,
    index: i32,
    cost: f64,
}

/// The primitive range `[begin, end]` of the node being split.
#[derive(Clone, Copy)]
struct NodeRange {
    begin: i32,
    end: i32,
    count: i32,
}

impl<V: BvhVector> BvhSweepPlaneBuilder<V> {
    /// Creates sweep plane SAH BVH builder; `BvhSweepPlaneBuilder::<BvhVec3d>::new(5, 32, 1)`.
    pub fn new(leaf_node_size: i32, max_tree_depth: i32, num_of_threads: i32) -> Self {
        Self {
            transient: BvhBuilderTransient::new(leaf_node_size, max_tree_depth),
            num_of_threads,
            vector: PhantomData,
        }
    }

    /// Tries X, Y (and Z) in order; an axis whose node extent is at most
    /// `THE_NODE_MIN_SIZE` is skipped. The set stays sorted along the last
    /// axis tried.
    fn find_best_split(
        &self,
        set: &mut dyn BvhSet<V>,
        bvh: &BvhTree<V>,
        node: i32,
        range: NodeRange,
    ) -> Result<SweepSplit, Failure> {
        let mut best = SweepSplit {
            axis: -1,
            index: 0,
            cost: f64::MAX,
        };
        let mut left_areas = vec![0.0_f64; range.count as usize];
        let mut right_areas = vec![0.0_f64; range.count as usize];
        for axis in 0..V::DIMENSION.min(3) as i32 {
            let extent =
                bvh.max_point(node)?.component(axis) - bvh.min_point(node)?.component(axis);
            if extent.to_double() <= THE_NODE_MIN_SIZE {
                continue;
            }
            BvhQuickSorter::new(axis).perform_range(set, range.begin, range.end)?;
            sweep_areas(&*set, range, &mut left_areas, &mut right_areas)?;
            choose_split(&left_areas, &right_areas, range.count, axis, &mut best);
        }
        Ok(best)
    }
}

/// Areas of the boxes of the first (left) and last (right) `k` primitives,
/// for k in `1..count`, stored at index k as `double`.
fn sweep_areas<V: BvhVector>(
    set: &dyn BvhSet<V>,
    range: NodeRange,
    left_areas: &mut [f64],
    right_areas: &mut [f64],
) -> Result<(), Failure> {
    let mut left_box = BvhBox::<V>::default();
    for index in 1..range.count {
        left_box.combine(&set.element_box(index + range.begin - 1)?);
        left_areas[index as usize] = left_box.area().to_double();
    }
    let mut right_box = BvhBox::<V>::default();
    for index in 1..range.count {
        right_box.combine(&set.element_box(range.end - index + 1)?);
        right_areas[index as usize] = right_box.area().to_double();
    }
    Ok(())
}

/// Simplified SAH (areas are not divided by the node area); only a strictly
/// smaller cost replaces the current best.
fn choose_split(
    left_areas: &[f64],
    right_areas: &[f64],
    count: i32,
    axis: i32,
    best: &mut SweepSplit,
) {
    for left in 1..count {
        let right = count - left;
        let cost = left_areas[left as usize] * f64::from(left)
            + right_areas[right as usize] * f64::from(right);
        if cost < best.cost {
            *best = SweepSplit {
                axis,
                index: left,
                cost,
            };
        }
    }
}

/// Boxes of the primitives before and after the split, the right one grown
/// from the last primitive down.
fn split_boxes<V: BvhVector>(
    set: &dyn BvhSet<V>,
    range: NodeRange,
    middle: i32,
) -> Result<(BvhBox<V>, BvhBox<V>), Failure> {
    let mut left = BvhBox::default();
    for index in range.begin..middle {
        left.combine(&set.element_box(index)?);
    }
    let mut right = BvhBox::default();
    for index in (middle..=range.end).rev() {
        right.combine(&set.element_box(index)?);
    }
    Ok((left, right))
}

impl<V: BvhVector> BvhQueueBuilder<V> for BvhSweepPlaneBuilder<V> {
    fn queue_builder_transient(&self) -> &BvhBuilderTransient {
        &self.transient
    }
    fn num_of_threads(&self) -> i32 {
        self.num_of_threads
    }
    /// Splits a node with more than `LeafNodeSize` primitives at the cheapest
    /// sweep position; no split when every axis is too thin. The set is sorted
    /// again unless the chosen axis is the last one (`N < 4 ? N - 1 : 2`).
    fn build_node(
        &self,
        set: &mut dyn BvhSet<V>,
        bvh: &mut BvhTree<V>,
        node: i32,
    ) -> Result<BvhChildNodes<V>, Failure> {
        let (begin, end) = (bvh.beg_primitive(node)?, bvh.end_primitive(node)?);
        let range = NodeRange {
            begin,
            end,
            count: bvh.nb_primitives(node)?,
        };
        if range.count <= self.transient.leaf_node_size() {
            return Ok(BvhChildNodes::default());
        }
        let best = self.find_best_split(set, bvh, node, range)?;
        if best.axis == -1 {
            return Ok(BvhChildNodes::default());
        }
        bvh.set_inner(node)?;
        if best.axis != V::DIMENSION.min(3) as i32 - 1 {
            BvhQuickSorter::new(best.axis).perform_range(set, begin, end)?;
        }
        let middle = begin + best.index;
        let (left, right) = split_boxes(&*set, range, middle)?;
        Ok(BvhChildNodes::new(
            left,
            right,
            BvhPrimitiveRange::new(begin, middle - 1),
            BvhPrimitiveRange::new(middle, end),
        ))
    }
}
impl<V: BvhVector> BvhBuilder<V> for BvhSweepPlaneBuilder<V> {
    fn builder_transient(&self) -> &BvhBuilderTransient {
        &self.transient
    }
    fn build(
        &self,
        set: &mut dyn BvhSet<V>,
        bvh: &mut BvhTree<V>,
        bounding: &BvhBox<V>,
    ) -> Result<(), Failure> {
        queue_build(self, set, bvh, bounding)
    }
}
