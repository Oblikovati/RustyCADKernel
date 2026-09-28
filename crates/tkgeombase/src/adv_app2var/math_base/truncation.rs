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

//! Truncation of Legendre and Jacobi curves to a tolerance: `mmtrpjj_` and the
//! file-static `mmtrpj0_`, `mmtrpj2_`, `mmtrpj4_`, `mmtrpj6_`.
use super::approx_errors::{XMAXJ_ORDER2, XMAXJ_ORDER4, XMAXJ_ORDER6};
use super::{mmeps1_, mzsnorm_};
use crate::adv_app2var::f2c::{F2cArray, F2cArrayMut};

/// Finds how many coefficients of a Jacobi curve of continuity order `iordre` keep
/// the truncation error within `epsi3d` (`mmtrpjj_`): `ncfnew` receives the count,
/// `errmax` the error of the dropped tail and `ycvmax` its bound per dimension.
/// Orders beyond 1 all use the order-2 (`mmtrpj6_`) tables; returns 0.
/// `mmtrpjj_(6, 3, 6, 1e-6, 0, &crvlgd, &mut ycvmax, &mut errmax, &mut ncfnew)`.
pub fn mmtrpjj_(
    ncofmx: i32,
    ndimen: i32,
    ncoeff: i32,
    epsi3d: f64,
    iordre: i32,
    crvlgd: &[f64],
    ycvmax: &mut [f64],
    errmax: &mut f64,
    ncfnew: &mut i32,
) -> i32 {
    let curve = JacobiCurve {
        ncofmx,
        ndimen,
        ncoeff,
        coefficients: crvlgd,
    };
    match (iordre + 1) << 1 {
        0 => mmtrpj0_(&curve, epsi3d, ycvmax, errmax, ncfnew),
        2 => mmtrpj_(&curve, 2, &XMAXJ_ORDER2, epsi3d, ycvmax, errmax, ncfnew),
        4 => mmtrpj_(&curve, 4, &XMAXJ_ORDER4, epsi3d, ycvmax, errmax, ncfnew),
        _ => mmtrpj_(&curve, 6, &XMAXJ_ORDER6, epsi3d, ycvmax, errmax, ncfnew),
    }
    0
}

/// A `(ncofmx, ndimen)` coefficient array of which the first `ncoeff` are used.
struct JacobiCurve<'a> {
    ncofmx: i32,
    ndimen: i32,
    ncoeff: i32,
    coefficients: &'a [f64],
}

/// `mmtrpj0_`: truncation of a Legendre curve, dropping coefficients from the
/// highest down while the accumulated bound stays within `epsi3d`.
fn mmtrpj0_(
    curve: &JacobiCurve<'_>,
    epsi3d: f64,
    ycvmax: &mut [f64],
    epstrc: &mut f64,
    ncfnew: &mut i32,
) {
    let dim1: i32 = curve.ncofmx;
    let crvlgd = F2cArray::new(curve.coefficients, dim1 + 1);
    let mut ycv = F2cArrayMut::new(ycvmax, 1);
    *ncfnew = 1;
    for i in 1..=curve.ndimen {
        ycv[i] = 0.;
    }
    *epstrc = 0.;
    for i in (2..=curve.ncoeff).rev() {
        let bidon: f64 = ((f64::from(i - 1) * 2. + 1.) / 2.).sqrt();
        for nd in 1..=curve.ndimen {
            ycv[nd] += crvlgd[i + nd * dim1].abs() * bidon;
        }
        let error: f64 = mzsnorm_(curve.ndimen, ycv.tail(1));
        if error > epsi3d {
            *ncfnew = i;
            return;
        }
        *epstrc = error;
    }
}

/// `mmtrpj2_`, `mmtrpj4_` and `mmtrpj6_`: truncation of a Jacobi curve that keeps at
/// least the `ia` constraint coefficients; when all the others go, the constraint
/// coefficients below `eps1` are dropped too (label L400).
fn mmtrpj_(
    curve: &JacobiCurve<'_>,
    ia: i32,
    xmaxj: &[f64],
    epsi3d: f64,
    ycvmax: &mut [f64],
    epstrc: &mut f64,
    ncfnew: &mut i32,
) {
    let dim1: i32 = curve.ncofmx;
    let crvlgd = F2cArray::new(curve.coefficients, dim1 + 1);
    let mut ycv = F2cArrayMut::new(ycvmax, 1);
    *ncfnew = ia;
    for i in 1..=curve.ndimen {
        ycv[i] = 0.;
    }
    *epstrc = 0.;
    let ncut: i32 = ia + 1;
    for i in (ncut..=curve.ncoeff).rev() {
        let bidon: f64 = xmaxj[(i - ncut) as usize];
        for nd in 1..=curve.ndimen {
            ycv[nd] += crvlgd[i + nd * dim1].abs() * bidon;
        }
        let error: f64 = mzsnorm_(curve.ndimen, ycv.tail(1));
        if error > epsi3d {
            *ncfnew = i;
            break;
        }
        *epstrc = error;
    }
    if *ncfnew == ia {
        *ncfnew = significant_constraints(&crvlgd, dim1, curve.ndimen, ia);
    }
}

/// The L400 search of `mmtrpj2_`..`mmtrpj6_`: the highest degree `2..=ia` whose
/// coefficients exceed `eps1` in sum, or 1.
fn significant_constraints(crvlgd: &F2cArray<'_, f64>, dim1: i32, ndimen: i32, ia: i32) -> i32 {
    let mut eps1: f64 = 0.;
    mmeps1_(&mut eps1);
    for i in (2..=ia).rev() {
        let mut bid: f64 = 0.;
        for nd in 1..=ndimen {
            bid += crvlgd[i + nd * dim1].abs();
        }
        if bid > eps1 {
            return i;
        }
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legendre_truncation_stops_at_the_first_large_tail() {
        let crvlgd: [f64; 4] = [1.0, 0.5, 1e-9, 1e-10];
        let (mut ycvmax, mut errmax, mut ncfnew) = ([0.0; 1], 0.0, 0);
        mmtrpjj_(
            4,
            1,
            4,
            1e-6,
            -1,
            &crvlgd,
            &mut ycvmax,
            &mut errmax,
            &mut ncfnew,
        );
        assert_eq!(ncfnew, 2);
        assert!(errmax > 0.0 && errmax < 1e-6);
    }

    #[test]
    fn jacobi_truncation_drops_negligible_constraint_coefficients() {
        let crvlgd: [f64; 5] = [1.0, 0.0, 0.0, 0.0, 0.0];
        let (mut ycvmax, mut errmax, mut ncfnew) = ([0.0; 1], 0.0, 0);
        mmtrpjj_(
            5,
            1,
            5,
            1e-6,
            0,
            &crvlgd,
            &mut ycvmax,
            &mut errmax,
            &mut ncfnew,
        );
        assert_eq!((ncfnew, errmax), (1, 0.0));
        let crvlgd: [f64; 5] = [1.0, 1.0, 1.0, 0.0, 1.0];
        mmtrpjj_(
            5,
            1,
            5,
            1e-6,
            0,
            &crvlgd,
            &mut ycvmax,
            &mut errmax,
            &mut ncfnew,
        );
        assert_eq!(ncfnew, 5);
    }
}
