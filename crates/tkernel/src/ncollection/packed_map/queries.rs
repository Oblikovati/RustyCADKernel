// Copyright (c) 2005-2026 OPEN CASCADE SAS
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

use super::{PackedInteger, PackedMap, PackedWord};
impl<T: PackedInteger> PackedMap<T> {
    /// Return the native minimum or type maximum when empty; `map.minimal()`.
    pub fn minimal(&self) -> T {
        let mut result: T = T::MAX;
        let mut selected = None;
        for block in self.buckets.iter().flatten() {
            let base: T = T::from_bits(block.index.bits() << Self::SHIFT);
            if result > base {
                result = base;
                selected = Some(block);
            }
        }
        selected.map_or(result, |block| {
            T::from_bits(
                (block.index.bits() << Self::SHIFT) | u64::from(block.bits.bits().trailing_zeros()),
            )
        })
    }
    /// Preserve native block-base selection, including the lowest-block edge case; `map.maximal()`.
    pub fn maximal(&self) -> T {
        let mut result: T = T::MIN;
        let mut selected = None;
        for block in self.buckets.iter().flatten() {
            let base: T = T::from_bits(block.index.bits() << Self::SHIFT);
            if result < base {
                result = base;
                selected = Some(block);
            }
        }
        selected.map_or(result, |block| {
            T::from_bits(
                (block.index.bits() << Self::SHIFT)
                    | u64::from(63 - block.bits.bits().leading_zeros()),
            )
        })
    }
}
