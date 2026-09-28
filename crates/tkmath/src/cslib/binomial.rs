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

//! Temporary copy of PLib::Bin for CSLib.
//!
//! PLib is being ported in parallel. This file must be deleted and its callers
//! switched to the PLib port's `Bin` when both are merged; it reproduces only
//! the native algorithm CSLib depends on.
use rustycad_tkernel::standard::Failure;

/// BSplCLib::MaxDegree(), the largest degree PLib's Pascal triangle stores.
const MAX_DEGREE: usize = 25;

/// Pascal's triangle built with PLib's `int` recurrence, so every entry is exact.
const PASCAL: [[i32; MAX_DEGREE + 1]; MAX_DEGREE + 1] = pascal_triangle();

const fn pascal_triangle() -> [[i32; MAX_DEGREE + 1]; MAX_DEGREE + 1] {
    let mut table: [[i32; MAX_DEGREE + 1]; MAX_DEGREE + 1] = [[0; MAX_DEGREE + 1]; MAX_DEGREE + 1];
    table[0][0] = 1;
    let mut row: usize = 1;
    while row <= MAX_DEGREE {
        table[row][0] = 1;
        table[row][row] = 1;
        let mut column: usize = 1;
        while column < row {
            table[row][column] = table[row - 1][column - 1] + table[row - 1][column];
            column += 1;
        }
        row += 1;
    }
    table
}

/// PLib::Bin(N,P) with its native range checks; `plib_bin(4, 2)? == 6.0`.
///
/// Must be replaced by the PLib port's binomial when merged.
pub(crate) fn plib_bin(n: i32, p: i32) -> Result<f64, Failure> {
    if n < 0 || n > MAX_DEGREE as i32 {
        return Err(bin_range(
            n,
            p,
            "degree N is out of supported range [0, 25]",
        ));
    }
    if p < 0 || p > n {
        return Err(bin_range(n, p, "parameter P is out of valid range [0, N]"));
    }
    Ok(f64::from(PASCAL[n as usize][p as usize]))
}

fn bin_range(n: i32, p: i32, reason: &str) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("PLib::Bin({n}, {p}): {reason}"),
        "",
    )
}

#[cfg(test)]
mod tests {
    use super::plib_bin;

    #[test]
    fn plib_bin_matches_pascal_and_rejects_native_ranges() {
        assert_eq!(plib_bin(0, 0).unwrap(), 1.0);
        assert_eq!(plib_bin(4, 2).unwrap(), 6.0);
        assert_eq!(plib_bin(25, 12).unwrap(), 5_200_300.0);
        for (n, p) in [(-1, 0), (26, 0), (3, -1), (3, 4)] {
            let failure = plib_bin(n, p).unwrap_err();
            assert_eq!(failure.exception_type(), "Standard_OutOfRange");
        }
    }
}
