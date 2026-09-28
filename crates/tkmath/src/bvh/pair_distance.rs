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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_PairDistance.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_PairDistance<T, N, BVHSetType>`: minimum distance between the elements
//! of two BVH sets.
use super::scalar::BvhScalar;
use super::tools::box_box_square_distance_by_corners;
use super::traverse::BvhPairSetTraverse;
use super::vector::BvhVector;
use rustycad_tkernel::standard::Failure;
use std::marker::PhantomData;

/// Data and rules of `BVH_PairDistance`: the running square distance
/// (initially `numeric_limits<T>::max()`) and the done flag.
///
/// A selector embeds it, implements `accept` (lowering the distance through
/// [`BvhPairDistance::set_distance`]) and forwards `reject_node` and the base
/// rules to this type. Example: `let state = BvhPairDistance::<BvhVec3d>::default();`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BvhPairDistance<V: BvhVector> {
    distance: V::Scalar,
    is_done: bool,
    vector: PhantomData<V>,
}
impl<V: BvhVector> Default for BvhPairDistance<V> {
    fn default() -> Self {
        Self {
            distance: V::Scalar::MAX,
            is_done: false,
            vector: PhantomData,
        }
    }
}
impl<V: BvhVector> BvhPairDistance<V> {
    /// Returns IsDone flag; `state.is_done()`.
    pub fn is_done(&self) -> bool {
        self.is_done
    }
    /// Returns the computed square distance; `state.distance()`.
    pub fn distance(&self) -> V::Scalar {
        self.distance
    }
    /// Writes the protected `myDistance`, as `Accept` does in derived selectors.
    /// Example: `state.set_distance(candidate)`.
    pub fn set_distance(&mut self, distance: V::Scalar) {
        self.distance = distance;
    }
    /// The smaller metric is better; `state.is_metric_better(&1.0, &2.0)`.
    pub fn is_metric_better(&self, left: &V::Scalar, right: &V::Scalar) -> bool {
        *left < *right
    }
    /// Writes the square distance between the node boxes and rejects the pair
    /// when it exceeds the current distance.
    /// Example: `state.reject_node(&min1, &max1, &min2, &max2, &mut metric)`.
    pub fn reject_node(
        &self,
        corner_min1: &V,
        corner_max1: &V,
        corner_min2: &V,
        corner_max2: &V,
        metric: &mut V::Scalar,
    ) -> bool {
        *metric =
            box_box_square_distance_by_corners(corner_min1, corner_max1, corner_min2, corner_max2);
        *metric > self.distance
    }
    /// Rejects the branch whose metric exceeds the current distance.
    /// Example: `state.reject_metric(&metric)`.
    pub fn reject_metric(&self, metric: &V::Scalar) -> bool {
        *metric > self.distance
    }
    /// Stops the descent once the distance is exactly zero; `state.stop()`.
    pub fn stop(&self) -> bool {
        self.distance == V::Scalar::ZERO
    }
}

/// `BVH_PairDistance::ComputeDistance` for a selector that embeds
/// [`BvhPairDistance`]. Example: `let squared = tool.compute_distance()?;`.
pub trait BvhPairDistanceTraverse<V: BvhVector>: BvhPairSetTraverse<V> {
    /// Borrows the embedded pair distance data.
    fn pair_distance_state(&self) -> &BvhPairDistance<V>;
    /// Mutably borrows the embedded pair distance data.
    fn pair_distance_state_mut(&mut self) -> &mut BvhPairDistance<V>;
    /// Selects through both sets, marks the result done when any pair was
    /// accepted, and returns the square distance.
    fn compute_distance(&mut self) -> Result<V::Scalar, Failure>
    where
        Self: Sized,
    {
        let selected = self.select()?;
        self.pair_distance_state_mut().is_done = selected > 0;
        Ok(self.pair_distance_state().distance)
    }
}
