// Copyright (c) 2026 OPEN CASCADE SAS
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

//! Static median KD tree with OCCT leaf buckets and one-based original point indices.
use super::Array1;
use crate::standard::Failure;
mod array_build;
mod build;
mod heap;
mod nearest;
mod radii;
mod search;
mod selection;
/// Coordinates supplied by an application point type; `point.coord(1)` selects X.
pub trait KdPoint: Clone {
    /// Returns the one-based coordinate; `point.coord(1)` selects X.
    fn coord(&self, index: usize) -> f64;
}
impl<const D: usize> KdPoint for [f64; D] {
    fn coord(&self, index: usize) -> f64 {
        self[index - 1]
    }
}
/// Owned point cloud, optional radii, and a median permutation; `KDTree::<[f64;2],2>::default()`.
#[derive(Clone)]
pub struct KDTree<P, const D: usize, const R: bool = false> {
    points: Vec<P>,
    indices: Vec<usize>,
    radii: Vec<f64>,
    max_radius: Vec<f64>,
}
impl<P, const D: usize, const R: bool> Default for KDTree<P, D, R> {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            indices: Vec::new(),
            radii: Vec::new(),
            max_radius: Vec::new(),
        }
    }
}
impl<P: KdPoint, const D: usize, const R: bool> KDTree<P, D, R> {
    /// Counts original points, including duplicates; `tree.len()`.
    pub fn len(&self) -> usize {
        self.points.len()
    }
    /// Reports an empty cloud; `tree.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
    /// Releases the entire cloud and permutation; `tree.clear()`.
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    /// Borrows by one-based original index; `tree.point(1)?`.
    pub fn point(&self, index: usize) -> Result<&P, Failure> {
        self.points.get(index.wrapping_sub(1)).ok_or_else(|| {
            Failure::out_of_range(&format!(
                "KDTree index {index}; expected 1..={}",
                self.len()
            ))
        })
    }
    fn distance(&self, query: &P, index: usize) -> f64 {
        square_distance::<P, D>(query, &self.points[index - 1])
    }
}
fn square_distance<P: KdPoint, const D: usize>(left: &P, right: &P) -> f64 {
    let mut distance = 0.0;
    for axis in 1..=D {
        let diff = left.coord(axis) - right.coord(axis);
        distance += diff * diff;
    }
    distance
}
#[derive(Clone, Copy)]
struct Region<const D: usize> {
    lo: usize,
    hi: usize,
    depth: usize,
    min: [f64; D],
    max: [f64; D],
}
impl<const D: usize> Region<D> {
    fn root(size: usize) -> Self {
        Self {
            lo: 0,
            hi: size,
            depth: 0,
            min: [-f64::MAX; D],
            max: [f64::MAX; D],
        }
    }
    fn middle(&self) -> usize {
        self.lo + (self.hi - self.lo - 1) / 2
    }
    fn children(&self, split: f64) -> (Self, Self) {
        let mid = self.middle();
        let axis = self.depth % D;
        let mut left = *self;
        left.hi = mid;
        left.depth += 1;
        left.max[axis] = split;
        let mut right = *self;
        right.lo = mid + 1;
        right.depth += 1;
        right.min[axis] = split;
        (left, right)
    }
    fn distance<P: KdPoint>(&self, query: &P) -> f64 {
        let mut distance = 0.0;
        for axis in 0..D {
            let coordinate = query.coord(axis + 1);
            let diff = if coordinate < self.min[axis] {
                self.min[axis] - coordinate
            } else if coordinate > self.max[axis] {
                coordinate - self.max[axis]
            } else {
                0.0
            };
            distance += diff * diff;
        }
        distance
    }
}
