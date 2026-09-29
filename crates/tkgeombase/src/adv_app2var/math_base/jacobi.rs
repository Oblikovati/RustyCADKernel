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

//! Jacobi to canonical conversion: `mmjacan_` and `mmjaccv_`.
use crate::adv_app2var::data::getmmjcobi;
use crate::adv_app2var::f2c::{F2cArray, F2cArrayMut};

/// Converts a Jacobi polynomial of degree `ndeg` and order `ideriv` (-1..=2), given
/// as its even then odd coefficients `poljac` (`(0:ndeg/2, 2)`), to the canonical
/// base `polcan[0..=ndeg]` with table `PLGCAN` (`mmjacan_`); returns 0.
/// `mmjacan_(0, 3, &poljac, &mut polcan)`.
pub fn mmjacan_(ideriv: i32, ndeg: i32, poljac: &[f64], polcan: &mut [f64]) -> i32 {
    let plgcan: &[f64] = getmmjcobi().plgcan;
    let poljac_dim1: i32 = ndeg / 2 + 1;
    for i in 0..=ndeg / 2 {
        let mut bid: f64 = 0.;
        let iptt: i32 = i * 31 - (i + 1) * i / 2 + 1;
        for j in i..=ndeg / 2 {
            bid += plgcan[(iptt + j + ideriv * 992 + 991) as usize] * poljac[j as usize];
        }
        polcan[(i * 2) as usize] = bid;
    }
    if ndeg == 0 {
        return 0;
    }
    for i in 0..=(ndeg - 1) / 2 {
        let mut bid: f64 = 0.;
        let iptt: i32 = i * 31 - (i + 1) * i / 2 + 1;
        for j in i..=(ndeg - 1) / 2 {
            let table: f64 = plgcan[(iptt + j + ((ideriv << 1) + 1) * 496 + 991) as usize];
            bid += table * poljac[(j + poljac_dim1) as usize];
        }
        polcan[((i << 1) + 1) as usize] = bid;
    }
    0
}

/// Converts `ndim` Jacobi curves of `ncoef` coefficients and order `ider`
/// (`crvlgd`, `(0:ncoef-1, ndim)`) to the canonical base (`crvcan`, same layout),
/// through the even/odd scratch `polaux` (`mmjaccv_`); returns 0.
/// `mmjaccv_(4, 1, 0, &crvlgd, &mut polaux, &mut crvcan)`.
pub fn mmjaccv_(
    ncoef: i32,
    ndim: i32,
    ider: i32,
    crvlgd: &[f64],
    polaux: &mut [f64],
    crvcan: &mut [f64],
) -> i32 {
    let polaux_dim1: i32 = (ncoef - 1) / 2 + 1;
    let dim1: i32 = ncoef;
    let crvlgd = F2cArray::new(crvlgd, dim1);
    let mut crvcan = F2cArrayMut::new(crvcan, dim1);
    let ndeg: i32 = ncoef - 1;
    for nd in 1..=ndim {
        let mut ii: i32 = 0;
        for i in 0..=ndeg / 2 {
            polaux[i as usize] = crvlgd[ii + nd * dim1];
            ii += 2;
        }
        ii = 1;
        if ndeg >= 1 {
            for i in 0..=(ndeg - 1) / 2 {
                polaux[(i + polaux_dim1) as usize] = crvlgd[ii + nd * dim1];
                ii += 2;
            }
        }
        mmjacan_(ider, ndeg, polaux, crvcan.tail_mut(nd * dim1));
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_constant_jacobi_polynomial_stays_constant() {
        let mut polcan: [f64; 1] = [0.0];
        mmjacan_(-1, 0, &[2.0], &mut polcan);
        assert_eq!(polcan[0], 2.0 * getmmjcobi().plgcan[0]);
    }

    #[test]
    fn curves_convert_column_by_column() {
        let crvlgd: [f64; 4] = [1.0, 0.0, 0.5, 0.25];
        let (mut polaux, mut crvcan) = ([0.0; 4], [0.0; 4]);
        mmjaccv_(2, 2, 0, &crvlgd, &mut polaux, &mut crvcan);
        let (mut first, mut second) = ([0.0; 2], [0.0; 2]);
        mmjacan_(0, 1, &[1.0, 0.0], &mut first);
        mmjacan_(0, 1, &[0.5, 0.25], &mut second);
        assert_eq!(crvcan, [first[0], first[1], second[0], second[1]]);
    }
}
