// Created on: 2016-04-13
// Created by: Denis BOGOLEPOV
// Copyright (c) 2013-2016 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_QuickSorter.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_QuickSorter<T, N>`: centroid sort along one axis with `std::sort`.
use super::set::BvhSet;
use super::sorter::BvhSorter;
use super::vector::BvhVector;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::stl;

/// Centroid-based sort of an abstract set along X (0), Y (1) or Z (2).
///
/// The order of equal centroids follows libstdc++'s unstable introsort, as in
/// the GCC oracle. Example: `BvhQuickSorter::new(1).perform(&mut set)?`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BvhQuickSorter {
    axis: i32,
    is_parallel: bool,
}
impl BvhQuickSorter {
    /// Creates new BVH quick sorter for the given axis; `BvhQuickSorter::new(0)`.
    pub const fn new(axis: i32) -> Self {
        Self {
            axis,
            is_parallel: false,
        }
    }
    /// Axis used to arrange the primitives; `sorter.axis()`.
    pub const fn axis(&self) -> i32 {
        self.axis
    }
}
impl<V: BvhVector> BvhSorter<V> for BvhQuickSorter {
    /// Sorts an index permutation by centroid, then applies it with cycle swaps.
    ///
    /// Centroids are read once before sorting; `Center` is a const query, so
    /// the comparisons see the same values the native comparator recomputes.
    fn perform_range(
        &mut self,
        set: &mut dyn BvhSet<V>,
        start: i32,
        last: i32,
    ) -> Result<(), Failure> {
        let size = last - start + 1;
        if size <= 1 {
            return Ok(());
        }
        let centers = (0..size)
            .map(|offset| set.center(start + offset, self.axis))
            .collect::<Result<Vec<V::Scalar>, Failure>>()?;
        let mut indices: Vec<usize> = (0..size as usize).collect();
        stl::sort_by(&mut indices, |first, second| {
            centers[*first] < centers[*second]
        });
        apply_permutation(set, start, &indices)
    }
    fn is_parallel(&self) -> bool {
        self.is_parallel
    }
    fn set_parallel(&mut self, is_parallel: bool) {
        self.is_parallel = is_parallel;
    }
}

/// Moves element `indices[i]` to position `i` by following the cycles of the
/// inverse permutation: at most `size` swaps, in the native swap order.
fn apply_permutation<V: BvhVector>(
    set: &mut dyn BvhSet<V>,
    start: i32,
    indices: &[usize],
) -> Result<(), Failure> {
    let mut inverse = vec![0; indices.len()];
    for (position, index) in indices.iter().enumerate() {
        inverse[*index] = position;
    }
    for position in 0..inverse.len() {
        while inverse[position] != position {
            let target = inverse[position];
            set.swap(start + position as i32, start + target as i32)?;
            inverse.swap(position, target);
        }
    }
    Ok(())
}
