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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_Box.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_Box<T, N>`: axis aligned bounding box based on BVH vectors.
use super::scalar::{BvhScalar, std_max, std_min};
use super::vector::BvhVector;
mod json;
mod transform;

/// Axis aligned bounding box; invalid until a point is added.
///
/// An invalid box stores `max<T>` as minimum and `lowest<T>` as maximum, so
/// `IsValid` is decided by the first axis only, exactly as upstream.
/// Example: `BvhBox::from_corners(Vec3::new(0.0,0.0,0.0), Vec3::new(1.0,2.0,3.0))`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BvhBox<V: BvhVector> {
    min_point: V,
    max_point: V,
}
impl<V: BvhVector> Default for BvhBox<V> {
    fn default() -> Self {
        Self::new()
    }
}
impl<V: BvhVector> BvhBox<V> {
    /// Creates the invalid (uninitialized) box; `const EMPTY: BvhBox<BvhVec3d> = BvhBox::new();`.
    pub const fn new() -> Self {
        Self {
            min_point: V::FILLED_MAX,
            max_point: V::FILLED_LOWEST,
        }
    }
    /// Creates bounding box of given point; `BvhBox::from_point(Vec2::new(3.0, 7.0))`.
    pub const fn from_point(point: V) -> Self {
        Self {
            min_point: point,
            max_point: point,
        }
    }
    /// Creates bounding box from corner points, unchecked; `BvhBox::from_corners(min, max)`.
    pub const fn from_corners(min_point: V, max_point: V) -> Self {
        Self {
            min_point,
            max_point,
        }
    }
    /// Resets to the invalid sentinel state; `bounding.clear()`.
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    /// Compares only the first axis, as upstream; `BvhBox::<Vec3<f64>>::default().is_valid() == false`.
    pub fn is_valid(&self) -> bool {
        self.min_point[0] <= self.max_point[0]
    }
    /// Appends a point with `std::min`/`std::max` (`BVH::BoxMinMax`); `bounding.add(&point)`.
    pub fn add(&mut self, point: &V) {
        cwise_std_min(&mut self.min_point, point);
        cwise_std_max(&mut self.max_point, point);
    }
    /// Combines with another box, ignoring an invalid one; `bounding.combine(&other)`.
    pub fn combine(&mut self, other: &Self) {
        if other.is_valid() {
            cwise_std_min(&mut self.min_point, &other.min_point);
            cwise_std_max(&mut self.max_point, &other.max_point);
        }
    }
    /// Returns minimum point of bounding box; `bounding.corner_min().x`.
    pub const fn corner_min(&self) -> &V {
        &self.min_point
    }
    /// Returns maximum point of bounding box; `bounding.corner_max().x`.
    pub const fn corner_max(&self) -> &V {
        &self.max_point
    }
    /// Mutable minimum point, as the non-const `CornerMin()`; `bounding.corner_min_mut().x = -1.0`.
    pub fn corner_min_mut(&mut self) -> &mut V {
        &mut self.min_point
    }
    /// Mutable maximum point, as the non-const `CornerMax()`; `bounding.corner_max_mut().z = 15.0`.
    pub fn corner_max_mut(&mut self) -> &mut V {
        &mut self.max_point
    }
    /// Surface area; perimeter-like sum when degenerate, zero when invalid; `bounding.area()`.
    pub fn area(&self) -> V::Scalar {
        if !self.is_valid() {
            return V::Scalar::ZERO;
        }
        surface_area(&(self.max_point - self.min_point))
    }
    /// Returns diagonal of bounding box; `bounding.size()`.
    pub fn size(&self) -> V {
        self.max_point - self.min_point
    }
    /// Returns `(min + max) * 0.5`; `bounding.center()`.
    pub fn center(&self) -> V {
        (self.min_point + self.max_point) * V::Scalar::from_double(0.5)
    }
    /// Returns the center along one axis (`BVH::CenterAxis`); `bounding.center_axis(1)`.
    ///
    /// # Panics
    /// When `axis` is outside `0..N`; the upstream array access is undefined there.
    pub fn center_axis(&self, axis: i32) -> V::Scalar {
        let axis = axis as usize;
        (self.min_point[axis] + self.max_point[axis]) * V::Scalar::from_double(0.5)
    }
}

/// `BVH::SurfaceCalculator<T, N>::Area`: SAH surface measure of a box diagonal.
///
/// Only X, Y and Z enter for N >= 3; W is ignored by design upstream. A
/// degenerate (below `epsilon`) measure falls back to the sum of the absolute
/// extents. Example: `bvh::surface_area(&Vec3::new(1.0, 2.0, 3.0)) == 22.0`.
pub fn surface_area<V: BvhVector>(size: &V) -> V::Scalar {
    let epsilon = V::Scalar::EPSILON;
    if V::DIMENSION == 2 {
        let area = (size[0] * size[1]).absolute();
        if area < epsilon {
            return size[0].absolute() + size[1].absolute();
        }
        return area;
    }
    let (x, y, z) = (size[0], size[1], size[2]);
    let area = ((x * y).absolute() + (x * z).absolute() + (z * y).absolute())
        * V::Scalar::from_double(2.0);
    if area < epsilon {
        return x.absolute() + y.absolute() + z.absolute();
    }
    area
}

/// `BVH::BoxMinMax<T, N>::CwiseMin`: in-place `std::min` per component.
pub fn cwise_std_min<V: BvhVector>(target: &mut V, other: &V) {
    for axis in 0..V::DIMENSION {
        target[axis] = std_min(target[axis], other[axis]);
    }
}

/// `BVH::BoxMinMax<T, N>::CwiseMax`: in-place `std::max` per component.
pub fn cwise_std_max<V: BvhVector>(target: &mut V, other: &V) {
    for axis in 0..V::DIMENSION {
        target[axis] = std_max(target[axis], other[axis]);
    }
}

impl<V: BvhVector> BvhBox<V> {
    /// True when `other` is invalid or disjoint; touching boxes are not out.
    /// Example: `bounding.is_out(&other)`.
    pub fn is_out(&self, other: &Self) -> bool {
        if !other.is_valid() {
            return true;
        }
        self.is_out_corners(&other.min_point, &other.max_point)
    }
    /// Disjointness from the box given by two corners; `bounding.is_out_corners(&min, &max)`.
    pub fn is_out_corners(&self, min_point: &V, max_point: &V) -> bool {
        if !self.is_valid() {
            return true;
        }
        (0..V::DIMENSION).any(|axis| {
            self.min_point[axis] > max_point[axis] || self.max_point[axis] < min_point[axis]
        })
    }
    /// Returns (fully contains, overlap flag) as `Contains(other, hasOverlap)`.
    ///
    /// The overlap flag keeps the last axis evaluated before an early exit.
    /// Example: `let (inside, overlap) = bounding.contains(&other);`.
    pub fn contains(&self, other: &Self) -> (bool, bool) {
        if !other.is_valid() {
            return (false, false);
        }
        self.contains_corners(&other.min_point, &other.max_point)
    }
    /// Containment of the box given by two corners; `bounding.contains_corners(&min, &max)`.
    pub fn contains_corners(&self, min_point: &V, max_point: &V) -> (bool, bool) {
        if !self.is_valid() {
            return (false, false);
        }
        let mut is_inside = true;
        for axis in 0..V::DIMENSION {
            let overlaps =
                self.min_point[axis] <= max_point[axis] && self.max_point[axis] >= min_point[axis];
            if !overlaps {
                return (false, false);
            }
            is_inside = is_inside
                && self.min_point[axis] <= min_point[axis]
                && self.max_point[axis] >= max_point[axis];
        }
        (is_inside, true)
    }
    /// True when the point lies strictly outside; boundary points are inside.
    /// Example: `bounding.is_out_point(&point)`.
    pub fn is_out_point(&self, point: &V) -> bool {
        if !self.is_valid() {
            return true;
        }
        (0..V::DIMENSION)
            .any(|axis| point[axis] < self.min_point[axis] || point[axis] > self.max_point[axis])
    }
}
