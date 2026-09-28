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

//! MathUtils_Random.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
/// Native xoshiro256** generator; `RandomGenerator::new(6)`.
#[derive(Clone, Debug)]
pub struct RandomGenerator {
    state: [u64; 4],
}
impl Default for RandomGenerator {
    fn default() -> Self {
        Self::new(1)
    }
}
impl RandomGenerator {
    /// Expands the seed with native SplitMix64; `RandomGenerator::new(6)`.
    pub fn new(seed: u64) -> Self {
        let mut result: Self = Self { state: [0; 4] };
        result.set_seed(seed);
        result
    }
    /// Resets all four words; `generator.set_seed(42)`.
    pub fn set_seed(&mut self, mut seed: u64) {
        self.state = std::array::from_fn(|_| splitmix(&mut seed));
    }
    /// Returns the next native 64-bit output; `generator.next_int()`.
    pub fn next_int(&mut self) -> u64 {
        let result: u64 = self.state[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t: u64 = self.state[1] << 17;
        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];
        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(45);
        result
    }
    /// Uses the leading 53 random bits; `generator.next_real()`.
    pub fn next_real(&mut self) -> f64 {
        (self.next_int() >> 11) as f64 * (1.0 / 9007199254740992.0)
    }
}
fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e3779b97f4a7c15);
    let mut z: u64 = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}
