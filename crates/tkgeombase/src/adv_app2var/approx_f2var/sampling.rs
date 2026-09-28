// Copyright (c) 1996-1999 Matra Datavision
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

//! Sampling of the function on the square: `mma2ds1_` and the file-static
//! `mma2ds2_`.
use super::discretise::{SquareTables, SquareViews};
use crate::adv_app2var::evaluator::{AdvApp2VarEvaluation, AdvApp2VarEvaluatorFunc2Var};
use crate::adv_app2var::f2c::{F2cArray, F2cArrayMut};
use crate::adv_app2var::math_base::mmfmtb1_;

/// The square of `mma2ds1_`: the parameter intervals, the Legendre roots on
/// [-1,1] and the favourite iso direction.
pub struct SampledSquare<'a> {
    /// `ndimen`: dimension of the function.
    pub ndimen: i32,
    /// `uintfn`: the U interval.
    pub uintfn: [f64; 2],
    /// `vintfn`: the V interval.
    pub vintfn: [f64; 2],
    /// `nbpntu`: number of U roots.
    pub nbpntu: i32,
    /// `nbpntv`: number of V roots.
    pub nbpntv: i32,
    /// `urootb`: the U roots.
    pub urootb: &'a [f64],
    /// `vrootb`: the V roots.
    pub vrootb: &'a [f64],
    /// `isofav`: 1 to evaluate U-isos, 2 (or anything else) for V-isos.
    pub isofav: i32,
}

/// Adds the samples of `foncnp` at the roots of the square to the four symmetric
/// tables (`mma2ds1_`), evaluating along the favourite iso direction; `fpntab`
/// (`ndimen * max(nbpntu, nbpntv)`) and `ttable` are scratch. `iercod` is the
/// evaluator code plus 100 when it stops the sampling; returns 0.
///
/// Native quirk: for `isofav == 1` the tables are transposed back with
/// `mmfmtb1_` after the sampling whatever its outcome, and that call clears
/// `iercod`, so an evaluator failure is lost; the port keeps this.
/// `mma2ds1_(&square, &f, &mut tables, &mut fpntab, &mut ttable, &mut ier)`.
pub fn mma2ds1_(
    square: &SampledSquare<'_>,
    foncnp: &dyn AdvApp2VarEvaluatorFunc2Var,
    tables: &mut SquareTables<'_>,
    fpntab: &mut [f64],
    ttable: &mut [f64],
    iercod: &mut i32,
) -> i32 {
    *iercod = 0;
    let iuouv: i32 = if square.isofav < 1 || square.isofav > 2 {
        2
    } else {
        square.isofav
    };
    if iuouv == 2 {
        mma2ds2_(square, iuouv, foncnp, tables, fpntab, ttable, iercod);
    } else if transpose_tables(
        square.ndimen,
        [square.nbpntu, square.nbpntv],
        tables,
        iercod,
    ) {
        let swapped = SampledSquare {
            uintfn: square.vintfn,
            vintfn: square.uintfn,
            nbpntu: square.nbpntv,
            nbpntv: square.nbpntu,
            urootb: square.vrootb,
            vrootb: square.urootb,
            ..*square
        };
        // The transposed soditb plays disotb's role and conversely.
        let mut transposed = SquareTables {
            sosotb: &mut *tables.sosotb,
            disotb: &mut *tables.soditb,
            soditb: &mut *tables.disotb,
            diditb: &mut *tables.diditb,
        };
        mma2ds2_(
            &swapped,
            iuouv,
            foncnp,
            &mut transposed,
            fpntab,
            ttable,
            iercod,
        );
        transpose_tables(
            square.ndimen,
            [square.nbpntv, square.nbpntu],
            tables,
            iercod,
        );
    }
    if *iercod > 0 {
        *iercod += 100;
    }
    0
}

/// The `mmfmtb1_` transpositions of `mma2ds1_`, per dimension, of the four tables
/// sampled on `nbpnt = [first, second]` roots; false when one fails.
fn transpose_tables(
    ndimen: i32,
    nbpnt: [i32; 2],
    tables: &mut SquareTables<'_>,
    iercod: &mut i32,
) -> bool {
    let full: [i32; 2] = [nbpnt[0] / 2 + 1, nbpnt[1] / 2 + 1];
    let half: [i32; 2] = [nbpnt[0] / 2, nbpnt[1] / 2];
    for nd in 1..=ndimen {
        let blocks: [(&mut [f64], [i32; 2]); 4] = [
            (&mut *tables.sosotb, full),
            (&mut *tables.diditb, full),
            (&mut *tables.soditb, half),
            (&mut *tables.disotb, half),
        ];
        for (table, [isz1, isz2]) in blocks {
            let start: usize = ((nd - 1) * isz1 * isz2) as usize;
            // In place natively: mmfmtb1_ transposes through its own scratch table.
            let block: Vec<f64> = table[start..start + (isz1 * isz2) as usize].to_vec();
            let (mut ibid1, mut ibid2) = (0, 0);
            mmfmtb1_(
                isz1,
                &block,
                isz1,
                isz2,
                isz2,
                &mut table[start..],
                &mut ibid1,
                &mut ibid2,
                iercod,
            );
            if *iercod > 0 {
                return false;
            }
        }
    }
    true
}

/// `mma2ds2_`: the sampling itself; like `mma2ds1_` it adds 100 to a positive
/// evaluator code, so a failure seen through `mma2ds1_` carries 200.
fn mma2ds2_(
    square: &SampledSquare<'_>,
    iiuouv: i32,
    foncnp: &dyn AdvApp2VarEvaluatorFunc2Var,
    tables: &mut SquareTables<'_>,
    fpntab: &mut [f64],
    ttable: &mut [f64],
    iercod: &mut i32,
) {
    *iercod = 0;
    sample_rows(square, iiuouv, foncnp, tables, fpntab, ttable, iercod);
    if *iercod > 0 {
        *iercod += 100;
    }
}

/// The rows of `mma2ds2_`, with the constant parameter along V: the V roots at
/// `-t`, then the middle, then `+t`, each folded into the tables; stops at the
/// first positive evaluator code.
fn sample_rows(
    square: &SampledSquare<'_>,
    iiuouv: i32,
    foncnp: &dyn AdvApp2VarEvaluatorFunc2Var,
    tables: &mut SquareTables<'_>,
    fpntab: &mut [f64],
    ttable: &mut [f64],
    iercod: &mut i32,
) {
    let (u, v) = (square.uintfn, square.vintfn);
    let (alinu, blinu) = ((u[1] - u[0]) / 2., (u[1] + u[0]) / 2.);
    let (alinv, blinv) = ((v[1] - v[0]) / 2., (v[1] + v[0]) / 2.);
    let (dbfn1, dbfn2) = if iiuouv == 1 { (v, u) } else { (u, v) };
    let (nbpntu, nbpntv) = (square.nbpntu, square.nbpntv);
    let urootb = F2cArray::new(square.urootb, 1);
    let vrootb = F2cArray::new(square.vrootb, 1);
    {
        let mut tt = F2cArrayMut::new(ttable, 1);
        for iu in 1..=nbpntu {
            tt[iu] = blinu + alinu * urootb[iu];
        }
    }
    let sampler = RowSampler {
        square,
        iiuouv,
        dbfn: [dbfn1, dbfn2],
        ttable,
        foncnp,
    };
    let mut t = SquareViews::new(tables, nbpntu, nbpntv);
    let nvroo: i32 = nbpntv / 2;
    for iv in 1..=nvroo {
        let tcons: f64 = blinv + alinv * vrootb[iv];
        if !sampler.row(tcons, fpntab, iercod) {
            return;
        }
        fold_row(&mut t, fpntab, square, nvroo - iv + 1, -1.);
    }
    if nbpntv % 2 != 0 {
        if !sampler.row(blinv, fpntab, iercod) {
            return;
        }
        fold_middle_row(&mut t, fpntab, square);
    }
    for iv in 1..=nvroo {
        let tcons: f64 = alinv * vrootb[(nbpntv + 1) / 2 + iv] + blinv;
        if !sampler.row(tcons, fpntab, iercod) {
            return;
        }
        fold_row(&mut t, fpntab, square, iv, 1.);
    }
}

/// One `Evaluate` call of `mma2ds2_` along the U roots at the constant `tcons`.
struct RowSampler<'a> {
    square: &'a SampledSquare<'a>,
    iiuouv: i32,
    dbfn: [[f64; 2]; 2],
    ttable: &'a [f64],
    foncnp: &'a dyn AdvApp2VarEvaluatorFunc2Var,
}

impl RowSampler<'_> {
    /// Evaluates the row; false when the evaluator returns a positive code.
    fn row(&self, tcons: f64, fpntab: &mut [f64], iercod: &mut i32) -> bool {
        let request = AdvApp2VarEvaluation {
            dimension: self.square.ndimen,
            u_start_end: self.dbfn[0],
            v_start_end: self.dbfn[1],
            favor_iso: self.iiuouv,
            const_param: tcons,
            nb_params: self.square.nbpntu,
            parameters: self.ttable,
            u_order: 0,
            v_order: 0,
        };
        self.foncnp.evaluate(&request, fpntab, iercod);
        *iercod <= 0
    }
}

/// Folds one row of samples at V root row `jv` into the tables: `sign` is -1 for
/// the rows at `-t` (subtracted in the V differences) and +1 for those at `+t`.
fn fold_row(
    t: &mut SquareViews<'_>,
    fpntab: &[f64],
    square: &SampledSquare<'_>,
    jv: i32,
    sign: f64,
) {
    let ndimen: i32 = square.ndimen;
    let fpnt = F2cArray::new(fpntab, ndimen + 1);
    let nuroo: i32 = square.nbpntu / 2;
    let jdec: i32 = (square.nbpntu + 1) / 2;
    for id in 1..=ndimen {
        for iu in 1..=nuroo {
            let up: f64 = fpnt[id + (iu + jdec) * ndimen];
            let um: f64 = fpnt[id + (nuroo - iu + 1) * ndimen];
            let (f, h) = (t.f(iu, jv, id), t.h(iu, jv, id));
            t.soso[f] = t.soso[f] + up + um;
            t.diso[h] = t.diso[h] + up - um;
            if sign < 0. {
                t.sodi[h] = t.sodi[h] - up - um;
                t.didi[f] = t.didi[f] - up + um;
            } else {
                t.sodi[h] = t.sodi[h] + up + um;
                t.didi[f] = t.didi[f] + up - um;
            }
        }
        if square.nbpntu % 2 != 0 {
            let up: f64 = fpnt[id + jdec * ndimen];
            let f: i32 = t.f(0, jv, id);
            t.soso[f] += up;
            if sign < 0. {
                t.didi[f] -= up;
            } else {
                t.didi[f] += up;
            }
        }
    }
}

/// Folds the row at the middle V root of an odd count into the `jv = 0` entries.
fn fold_middle_row(t: &mut SquareViews<'_>, fpntab: &[f64], square: &SampledSquare<'_>) {
    let ndimen: i32 = square.ndimen;
    let fpnt = F2cArray::new(fpntab, ndimen + 1);
    let nuroo: i32 = square.nbpntu / 2;
    let jdec: i32 = (square.nbpntu + 1) / 2;
    for id in 1..=ndimen {
        for iu in 1..=nuroo {
            let up: f64 = fpnt[id + (jdec + iu) * ndimen];
            let um: f64 = fpnt[id + (nuroo - iu + 1) * ndimen];
            let f: i32 = t.f(iu, 0, id);
            t.soso[f] = t.soso[f] + up + um;
            t.didi[f] = t.didi[f] + up - um;
        }
        if square.nbpntu % 2 != 0 {
            let up: f64 = fpnt[id + jdec * ndimen];
            let f: i32 = t.f(0, 0, id);
            t.soso[f] += up;
        }
    }
}
