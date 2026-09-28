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
// Rust translation of FoundationClasses/TKMath/PLib/PLib.cxx (Bin) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Binomial coefficients from a compile-time Pascal triangle.
use rustycad_tkernel::standard::Failure;

/// Largest supported N; mirrors `BSplCLib::MaxDegree()`, which sizes the native table.
pub const MAX_BINOMIAL_DEGREE: i32 = 25;
const SIZE: usize = MAX_BINOMIAL_DEGREE as usize + 1;
// Native BinomAllocator builds the same integer triangle with C(n,k)=C(n-1,k-1)+C(n-1,k).
const PASCAL: [[i32; SIZE]; SIZE] = pascal_triangle();

const fn pascal_triangle() -> [[i32; SIZE]; SIZE] {
    let mut table = [[0; SIZE]; SIZE];
    table[0][0] = 1;
    let mut i = 1;
    while i < SIZE {
        table[i][0] = 1;
        table[i][i] = 1;
        let mut j = 1;
        while j < i {
            table[i][j] = table[i - 1][j - 1] + table[i - 1][j];
            j += 1;
        }
        i += 1;
    }
    table
}

/// Returns C(N,P) for 0 <= P <= N <= 25 (`PLib::Bin`); `plib::bin(5, 2)? == 10.0`.
pub fn bin(n: i32, p: i32) -> Result<f64, Failure> {
    if !(0..=MAX_BINOMIAL_DEGREE).contains(&n) {
        return Err(Failure::out_of_range(&format!(
            "PLib::Bin: degree N is out of supported range [0, 25] (N = {n})"
        )));
    }
    if p < 0 || p > n {
        return Err(Failure::out_of_range(&format!(
            "PLib::Bin: parameter P is out of valid range [0, N] (N = {n}, P = {p})"
        )));
    }
    Ok(f64::from(PASCAL[n as usize][p as usize]))
}
