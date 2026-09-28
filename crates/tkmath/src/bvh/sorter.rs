// Created on: 2014-01-10
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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_Sorter.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_Sorter<T, N>`: tool objects that reorder a primitive set in place.
use super::set::BvhSet;
use super::vector::BvhVector;
use rustycad_tkernel::standard::Failure;

/// Sorts an abstract primitive set, optionally in parallel.
/// Example: `BvhQuickSorter::new(0).perform(&mut set)?`.
pub trait BvhSorter<V: BvhVector> {
    /// Sorts the given inclusive range `[start, last]` of the set.
    fn perform_range(
        &mut self,
        set: &mut dyn BvhSet<V>,
        start: i32,
        last: i32,
    ) -> Result<(), Failure>;
    /// Sorts the whole set: `[0, Size() - 1]`; `sorter.perform(&mut set)?`.
    fn perform(&mut self, set: &mut dyn BvhSet<V>) -> Result<(), Failure> {
        let last = set.size() - 1;
        self.perform_range(set, 0, last)
    }
    /// Returns parallel flag (`myIsParallel`, initially false); `sorter.is_parallel()`.
    fn is_parallel(&self) -> bool;
    /// Sets the flag controlling possibility of parallel execution; `sorter.set_parallel(true)`.
    fn set_parallel(&mut self, is_parallel: bool);
}
