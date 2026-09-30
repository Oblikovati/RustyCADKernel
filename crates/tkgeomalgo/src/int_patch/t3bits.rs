// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/
// IntPatch_PrmPrmIntersection_T3Bits.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_PrmPrmIntersection_T3Bits`: a set of `size^3` bits in 32-bit words, the
//! grid cells of `IntPatch_PrmPrmIntersection`.

/// `IntPatch_PrmPrmIntersection_T3Bits`: `(size^3) >> 5` words of 32 bits (the bits of
/// a last incomplete word are not allocated). Setting or reading a bit outside the words
/// panics (undefined natively).
///
/// ```
/// use rustycad_tkgeomalgo::int_patch::IntPatchPrmPrmIntersectionT3Bits;
/// let (mut a, mut b) = (IntPatchPrmPrmIntersectionT3Bits::new(4), IntPatchPrmPrmIntersectionT3Bits::new(4));
/// a.add(37);
/// b.add(37);
/// let mut index = 0;
/// assert_eq!(a.and(&mut b, &mut index), 1);
/// assert_eq!((index, a.val(37)), (37, 0));
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IntPatchPrmPrmIntersectionT3Bits {
    p: Vec<i32>,
}

/// The word and the bit of `t` (`t >> 5`, `1 << (t & 31)`).
fn word_bit(t: i32) -> (usize, i32) {
    let word: usize = usize::try_from(t >> 5).unwrap_or(usize::MAX);
    (word, 1_i32.wrapping_shl((t as u32) & 31))
}

impl IntPatchPrmPrmIntersectionT3Bits {
    /// `IntPatch_PrmPrmIntersection_T3Bits(size)`: `(size^3) >> 5` null words (for
    /// `size^3 < 32` the native constructor writes before its empty array; here the set
    /// is empty). `IntPatchPrmPrmIntersectionT3Bits::new(4)`.
    pub fn new(size: i32) -> Self {
        let nb: i32 = size.wrapping_mul(size).wrapping_mul(size) >> 5;
        Self {
            p: vec![0; nb.max(0) as usize],
        }
    }

    /// `Add(t)`: sets the bit `t`. `bits.add(3)`.
    pub fn add(&mut self, t: i32) {
        let (word, bit) = word_bit(t);
        self.p[word] |= bit;
    }

    /// `Val(t)`: the bit `t` in its place in the word (0 when unset; negative for the
    /// bit 31). `bits.val(3)`.
    pub fn val(&self, t: i32) -> i32 {
        let (word, bit) = word_bit(t);
        self.p[word] & bit
    }

    /// `Raz(t)`: clears the bit `t`. `bits.raz(3)`.
    pub fn raz(&mut self, t: i32) {
        let (word, bit) = word_bit(t);
        self.p[word] &= !bit;
    }

    /// `ResetAnd()`: does nothing (its body is commented out). `bits.reset_and()`.
    pub fn reset_and(&mut self) {}

    /// `And(Oth, indiceprecedent)`: the first bit set in both sets from the word of
    /// `indice`; clears it in both, stores it in `indice` and returns 1; 0 when none.
    /// `bits.and(&mut other, &mut index)`.
    pub fn and(&mut self, other: &mut Self, indice: &mut i32) -> i32 {
        let mut k: i32 = *indice >> 5;
        while k < self.p.len() as i32 {
            let mut r: i32 = self.p[k as usize] & other.p[k as usize];
            if r != 0 {
                for c in 0..32 {
                    if r & 1 != 0 {
                        let op: i32 = (k << 5) | c;
                        self.raz(op);
                        other.raz(op);
                        *indice = op;
                        return 1;
                    }
                    r >>= 1;
                }
            }
            k += 1;
        }
        0
    }
}
