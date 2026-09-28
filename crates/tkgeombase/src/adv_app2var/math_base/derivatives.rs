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

//! Derivatives of canonical polynomial curves: `mmcdriv_`, `mmdrc11_`, `mmdrvck_`
//! and the file-static `mmdrvcb_`.
use super::mfac_;
use crate::adv_app2var::data::getmmcmcnp;
use crate::adv_app2var::f2c::{F2cArray, F2cArrayMut};
use crate::adv_app2var::sys_base::mvriraz_;

/// The factorials `1!..21!` of `mmdrvck_` (`mmfack`).
const MMFACK: [f64; 21] = [
    1.,
    2.,
    6.,
    24.,
    120.,
    720.,
    5040.,
    40320.,
    362880.,
    3628800.,
    39916800.,
    479001600.,
    6227020800.,
    87178291200.,
    1.307674368e12,
    2.0922789888e13,
    3.55687428096e14,
    6.402373705728e15,
    1.21645100408832e17,
    2.43290200817664e18,
    5.109094217170944e19,
];

/// `Cnp[n][p]` of common `MMCMCNP` at the native flat position `n + p*61`.
fn cnp(n: i32, p: i32) -> f64 {
    getmmcmcnp().cnp[(n + p * 61) as usize]
}

/// `k!` by the native product loop, starting from 1.
fn factorial_loop(k: i32) -> f64 {
    let mut mfactk: f64 = 1.;
    for i in 2..=k {
        mfactk *= f64::from(i);
    }
    mfactk
}

/// The coefficients of the `ideriv`-th derivative of the canonical curve `courbe`
/// (`(ndimen, ncoeff)`) into `crvdrv`; `ncofdv` receives their count (`mmcdriv_`).
/// Returns 0. `mmcdriv_(3, 4, &courbe, 1, &mut ncofdv, &mut crvdrv)`.
pub fn mmcdriv_(
    ndimen: i32,
    ncoeff: i32,
    courbe: &[f64],
    ideriv: i32,
    ncofdv: &mut i32,
    crvdrv: &mut [f64],
) -> i32 {
    let dim1: i32 = ndimen;
    let courbe = F2cArray::new(courbe, dim1 + 1);
    let mut crvdrv = F2cArrayMut::new(crvdrv, dim1 + 1);
    if ideriv >= ncoeff {
        for i in 1..=ndimen {
            crvdrv[i + dim1] = 0.;
        }
        *ncofdv = 1;
        return 0;
    }
    let k: i32 = ideriv;
    let mfactk: f64 = factorial_loop(k);
    for j in k + 1..=ncoeff {
        let bid: f64 = cnp(j - 1, k) * mfactk;
        for i in 1..=ndimen {
            crvdrv[i + (j - k) * dim1] = bid * courbe[i + j * dim1];
        }
    }
    *ncofdv = ncoeff - ideriv;
    0
}

/// The values and derivatives up to `iordre` of the canonical curve `courbe`
/// (`(0:ncoeff-1, ndimen)`) at -1 and +1 (`mmdrc11_`): `points` is
/// `(2, 0:iordre, ndimen)` and `mfactab` (at least `iordre + 1` values) receives the
/// factorials from index 1. Returns 0. `mmdrc11_(1, 1, 3, &courbe, &mut pts, &mut fact)`.
pub fn mmdrc11_(
    iordre: i32,
    ndimen: i32,
    ncoeff: i32,
    courbe: &[f64],
    points: &mut [f64],
    mfactab: &mut [f64],
) -> i32 {
    if iordre < 0 || ncoeff < 1 {
        return 0;
    }
    let dim2: i32 = iordre + 1;
    let mut pts = F2cArrayMut::new(points, (dim2 << 1) + 1);
    let cdim1: i32 = ncoeff;
    let courbe = F2cArray::new(courbe, cdim1);
    let ndgcb: i32 = ncoeff - 1;
    for nd in 1..=ndimen {
        pts[((nd * dim2) << 1) + 1] = courbe[ndgcb + nd * cdim1];
        pts[((nd * dim2) << 1) + 2] = courbe[ndgcb + nd * cdim1];
    }
    for nd in 1..=ndimen {
        for j in 1..=iordre {
            pts[((j + nd * dim2) << 1) + 1] = 0.;
            pts[((j + nd * dim2) << 1) + 2] = 0.;
        }
    }
    for nd in 1..=ndimen {
        for ndeg in 1..=ndgcb {
            for i in (1..=iordre).rev() {
                pts[((i + nd * dim2) << 1) + 1] =
                    -pts[((i + nd * dim2) << 1) + 1] + pts[((i - 1 + nd * dim2) << 1) + 1];
                pts[((i + nd * dim2) << 1) + 2] += pts[((i - 1 + nd * dim2) << 1) + 2];
            }
            let c: f64 = courbe[ndgcb - ndeg + nd * cdim1];
            pts[((nd * dim2) << 1) + 1] = -pts[((nd * dim2) << 1) + 1] + c;
            pts[((nd * dim2) << 1) + 2] += c;
        }
    }
    if iordre > 1 {
        scale_by_factorials(&mut pts, &mut mfactab[1..], iordre, ndimen);
    }
    0
}

/// The last step of `mmdrc11_`: `mfac_` fills the factorials, which scale the
/// derivatives of order 2 and more.
fn scale_by_factorials(
    pts: &mut F2cArrayMut<'_, f64>,
    mfactab: &mut [f64],
    iordre: i32,
    ndimen: i32,
) {
    mfac_(mfactab, iordre);
    let dim2: i32 = iordre + 1;
    for nd in 1..=ndimen {
        for i in 2..=iordre {
            let fact: f64 = mfactab[(i - 1) as usize];
            pts[((i + nd * dim2) << 1) + 1] *= fact;
            pts[((i + nd * dim2) << 1) + 2] *= fact;
        }
    }
}

/// `mmdrvcb_`: the value and derivatives up to `ideriv` at `tparam` of the canonical
/// curve `courbe` (`(ndim, 0:ncoeff-1)`) into `tabpnt` (`(ndim, 0:ideriv)`), by the
/// Horner scheme; `iercod` is 1 for a negative order or no coefficient.
pub(super) fn mmdrvcb_(
    ideriv: i32,
    ndim: i32,
    ncoeff: i32,
    courbe: &[f64],
    tparam: f64,
    tabpnt: &mut [f64],
    iercod: &mut i32,
) {
    if ideriv < 0 || ncoeff < 1 {
        *iercod = 1;
        return;
    }
    *iercod = 0;
    let dim1: i32 = ndim;
    let courbe = F2cArray::new(courbe, 1);
    let mut tab = F2cArrayMut::new(tabpnt, 1);
    let ndgcrb: i32 = ncoeff - 1;
    for nd in 1..=ndim {
        tab[nd] = courbe[nd + ndgcrb * dim1];
    }
    if ideriv >= 1 {
        mvriraz_(ndim * ideriv, tab.tail_mut(dim1 + 1));
    }
    for ndeg in 1..=ndgcrb {
        for nd in 1..=ndim {
            for i in (1..=ideriv).rev() {
                tab[nd + i * dim1] = tab[nd + i * dim1] * tparam + tab[nd + (i - 1) * dim1];
            }
            tab[nd] = tab[nd] * tparam + courbe[nd + (ndgcrb - ndeg) * dim1];
        }
    }
    for i in 2..=ideriv {
        for j in 2..=i {
            for nd in 1..=ndim {
                tab[nd + i * dim1] *= f64::from(j);
            }
        }
    }
}

/// The `ideriv`-th derivative at `tparam` of the canonical curve `courbe`
/// (`(ndimen, ncoeff)`) into `pntcrb` (`mmdrvck_`); returns 0.
/// `mmdrvck_(4, 3, &courbe, 1, 0.5, &mut pnt)`.
pub fn mmdrvck_(
    ncoeff: i32,
    ndimen: i32,
    courbe: &[f64],
    ideriv: i32,
    tparam: f64,
    pntcrb: &mut [f64],
) -> i32 {
    let dim1: i32 = ndimen;
    let courbe = F2cArray::new(courbe, dim1 + 1);
    let mut pnt = F2cArrayMut::new(pntcrb, 1);
    if ideriv >= ncoeff {
        for nd in 1..=ndimen {
            pnt[nd] = 0.;
        }
        return 0;
    }
    let k: i32 = ideriv;
    let mfactk: f64 = if ideriv <= 21 && ideriv > 0 {
        MMFACK[(k - 1) as usize]
    } else {
        factorial_loop(k)
    };
    for nd in 1..=ndimen {
        pnt[nd] = courbe[nd + ncoeff * dim1] * cnp(ncoeff - 1, k) * mfactk;
    }
    for j in (k + 1..=ncoeff - 1).rev() {
        let bid: f64 = cnp(j - 1, k) * mfactk;
        for nd in 1..=ndimen {
            pnt[nd] = pnt[nd] * tparam + courbe[nd + j * dim1] * bid;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derivatives_of_a_cubic() {
        // 1 + 2t + 3t^2 + 4t^3 in one dimension.
        let courbe: [f64; 4] = [1.0, 2.0, 3.0, 4.0];
        let (mut ncofdv, mut crvdrv) = (0, [0.0; 4]);
        mmcdriv_(1, 4, &courbe, 1, &mut ncofdv, &mut crvdrv);
        assert_eq!((ncofdv, &crvdrv[..3]), (3, &[2.0, 6.0, 12.0][..]));
        let mut pnt: [f64; 1] = [0.0];
        mmdrvck_(4, 1, &courbe, 1, 0.5, &mut pnt);
        assert_eq!(pnt[0], 2.0 + 6.0 * 0.5 + 12.0 * 0.25);
        let (mut tab, mut ier) = ([0.0; 3], 9);
        mmdrvcb_(2, 1, 4, &courbe, 0.5, &mut tab, &mut ier);
        assert_eq!((ier, tab), (0, [3.25, 8.0, 18.0]));
    }

    #[test]
    fn end_values_at_minus_and_plus_one() {
        // 1 + t + t^2: values 1 and 3, derivatives -1 and 3, second derivatives 2.
        let courbe: [f64; 3] = [1.0, 1.0, 1.0];
        let (mut pts, mut fact) = ([0.0; 6], [0.0; 3]);
        mmdrc11_(2, 1, 3, &courbe, &mut pts, &mut fact);
        assert_eq!(pts, [1.0, 3.0, -1.0, 3.0, 2.0, 2.0]);
        assert_eq!(fact[1..], [1.0, 2.0]);
    }
}
