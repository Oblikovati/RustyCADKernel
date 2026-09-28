// Created on: 2014-07-18
// Created by: Alexander Malyshev
// Copyright (c) 2014-2014 OPEN CASCADE SAS
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

//! Native wrapping-u32 Bullard generator, including its fixed default seed.
/// Reproducible OCCT generator; `BullardGenerator::default().next_real()`.
pub struct BullardGenerator {
    high: u32,
    low: u32,
}
impl Default for BullardGenerator {
    fn default() -> Self {
        Self::new(1)
    }
}
impl BullardGenerator {
    /// Starts from an explicit seed; `BullardGenerator::new(1)`.
    pub fn new(seed: u32) -> Self {
        Self {
            high: seed,
            low: seed ^ 0x49616e42,
        }
    }
    /// Restores both native words; `generator.set_seed(1)`.
    pub fn set_seed(&mut self, seed: u32) {
        self.high = seed;
        self.low = seed ^ 0x49616e42;
    }
    /// Produces the native 32-bit unsigned output; `generator.next_int()`.
    pub fn next_int(&mut self) -> u32 {
        self.high = (self.high >> 2).wrapping_add(self.high << 2);
        self.high = self.high.wrapping_add(self.low);
        self.low = self.low.wrapping_add(self.high);
        self.high
    }
    /// Divides by u32::MAX, retaining both endpoints; `generator.next_real()`.
    pub fn next_real(&mut self) -> f64 {
        f64::from(self.next_int()) / f64::from(u32::MAX)
    }
}
