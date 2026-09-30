// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT2d/MAT2d_BiInt.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MAT2d_BiInt`: a pair of integers, the key of the maps of MAT2d (a line and an
//! item of the line).
use rustycad_tkernel::standard::hashing;

/// `MAT2d_BiInt`: two integers; equal when both are.
/// ```
/// use rustycad_tktopalgo::mat2d::Mat2dBiInt;
/// let mut key = Mat2dBiInt::new(1, 2);
/// key.set_second_index(3);
/// assert!(key.is_equal(&Mat2dBiInt::new(1, 3)));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mat2dBiInt {
    i1: i32,
    i2: i32,
}

impl Mat2dBiInt {
    /// `MAT2d_BiInt(I1, I2)`; `Mat2dBiInt::new(1, 2)`.
    pub fn new(i1: i32, i2: i32) -> Self {
        Self { i1, i2 }
    }
    /// `FirstIndex()`; `key.first_index()`.
    pub fn first_index(&self) -> i32 {
        self.i1
    }
    /// `SecondIndex()`; `key.second_index()`.
    pub fn second_index(&self) -> i32 {
        self.i2
    }
    /// `FirstIndex(I1)`; `key.set_first_index(1)`.
    pub fn set_first_index(&mut self, i1: i32) {
        self.i1 = i1;
    }
    /// `SecondIndex(I2)`; `key.set_second_index(2)`.
    pub fn set_second_index(&mut self, i2: i32) {
        self.i2 = i2;
    }
    /// `IsEqual(B)` (and `operator==`); `key.is_equal(&other)`.
    pub fn is_equal(&self, b: &Mat2dBiInt) -> bool {
        self.i1 == b.first_index() && self.i2 == b.second_index()
    }
    /// `std::hash<MAT2d_BiInt>`: `opencascade::hashBytes` of the two integers;
    /// `key.hash_code()`.
    pub fn hash_code(&self) -> usize {
        let mut bytes = [0u8; 8];
        bytes[..4].copy_from_slice(&self.i1.to_ne_bytes());
        bytes[4..].copy_from_slice(&self.i2.to_ne_bytes());
        hashing::hash_bytes(&bytes)
    }
}

impl std::hash::Hash for Mat2dBiInt {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.hash_code());
    }
}
