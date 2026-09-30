// Copyright (c) 1992-1999 Matra Datavision
// Copyright (c) 1999-2024 OPEN CASCADE SAS
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
// Rust translation of CriticalPointsComputing, VBoundaryPrecise, DeltaU1Computing and
// StepComputing of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The critical U1 values where the walking lines of two cylinders break, and the
//! adaptive step of U1.
// The negated comparisons keep the native result for NaN operands.
#![allow(clippy::neg_cmp_op_on_partial_ord)]
use super::coeffs::{CoeffsValue, NUL_VALUE};
use super::inscribe::{exclude_near_elements, insertion_sort, min_max};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{Matrix, Vector};
use std::f64::consts::PI;

/// The number of critical points.
pub(in super::super) const NB_CRIT_POINTS_MAX: usize = 12;

/// `acos((s - C) / B) * sign + FI1` when the argument is inside ]-1, 1[, else infinite
/// (mathematically the negative ones should be -infinite).
fn crossing(s: f64, coeffs: &CoeffsValue, sign: f64) -> f64 {
    let arg: f64 = (s - coeffs.c) / coeffs.b;
    if arg.abs() < 1.0 {
        return sign * arg.acos() + coeffs.fi1;
    }
    precision::infinite()
}

/// `CriticalPointsComputing(theCoeffs, theUSurf1f, theUSurf1l, theUSurf2f,
/// theUSurf2l, thePeriod, theTol2D, theNbCritPointsMax, theU1crit)`: the U1 values
/// where U1 crosses the seam of S1 [0..1], the bounds of S1 [2..3], where U2 crosses
/// the seam of S2 [4..5], where the line crosses the U bounds of S2 [6..9], and the
/// bounds of the monotonicity of U2(U1) [10..11]; brought into `[0, T)`, sorted, the
/// near ones merged. Returns the number of significant (finite) values.
pub(in super::super) fn critical_points_computing(
    coeffs: &CoeffsValue,
    (usurf1f, usurf1l, usurf2f, usurf2l): (f64, f64, f64, f64),
    period: f64,
    tol_2d: f64,
    u1crit: &mut [f64; NB_CRIT_POINTS_MAX],
) -> usize {
    (u1crit[0], u1crit[1], u1crit[2], u1crit[3]) = (0.0, period, usurf1f, usurf1l);
    let cos_fi2: f64 = coeffs.fi2.cos();
    let bsb: f64 = coeffs.b.abs();
    if (coeffs.c - bsb <= cos_fi2) && (cos_fi2 <= coeffs.c + bsb) {
        let mut arg: f64 = (cos_fi2 - coeffs.c) / coeffs.b;
        // f64::clamp keeps NaN, as the two native comparisons do.
        arg = arg.clamp(-1.0, 1.0);
        u1crit[4] = -arg.acos() + coeffs.fi1;
        u1crit[5] = arg.acos() + coeffs.fi1;
    }
    let mut sf: f64 = (usurf2f - coeffs.fi2).cos();
    let mut sl: f64 = (usurf2l - coeffs.fi2).cos();
    min_max(&mut sf, &mut sl);
    u1crit[6] = crossing(sl, coeffs, -1.0);
    u1crit[7] = crossing(sf, coeffs, -1.0);
    u1crit[8] = crossing(sf, coeffs, 1.0);
    u1crit[9] = crossing(sl, coeffs, 1.0);
    u1crit[10] = coeffs.fi1;
    u1crit[11] = PI + coeffs.fi1;
    for u in u1crit.iter_mut() {
        if precision::is_infinite(*u) {
            continue;
        }
        *u %= period;
        if *u < 0.0 {
            *u += period;
        }
    }
    // Here all finite elements are in [0, period).
    loop {
        insertion_sort(u1crit, |a, b| a < b);
        if !exclude_near_elements(u1crit, usurf1f, usurf1l, tol_2d) {
            break;
        }
    }
    count_significant(u1crit, period, tol_2d)
}

/// The number of finite values after the sort; the last one merges with the first when
/// they differ by a period (e.g. 2e-17 and T - 1e-18).
fn count_significant(u1crit: &mut [f64; NB_CRIT_POINTS_MAX], period: f64, tol_2d: f64) -> usize {
    let mut nb: usize = NB_CRIT_POINTS_MAX;
    while nb > 0 {
        let b: f64 = u1crit[nb - 1];
        if precision::is_infinite(b) {
            nb -= 1;
            continue;
        }
        if nb == 1 {
            break;
        }
        let a: f64 = u1crit[0];
        if (b - a - period).abs() < tol_2d {
            u1crit[0] = (a + b - period) / 2.0;
            u1crit[nb - 1] = precision::infinite();
            nb -= 1;
        }
        break;
    }
    nb
}

/// A column of the 3x3 system from the column `col` of the 3x5 matrix.
fn set_col_from(syst: &mut Matrix<'_>, col: i32, matr: &Matrix<'_>, from: i32) -> Result<(), Failure> {
    syst.set_col(col, &matr.col(from)?)
}

/// `VBoundaryPrecise(theMatr, theV1AfterDecrByDelta, theV2AfterDecrByDelta, theV1Set,
/// theV2Set)`: when V1 (or V2) decreases as U1 increases (Cramer on the linearized
/// system), its target becomes the decreased value.
fn v_boundary_precise(matr: &Matrix<'_>, (v1_decr, v2_decr): (f64, f64), v1_set: &mut f64, v2_set: &mut f64) -> Result<(), Failure> {
    let mut syst = Matrix::with_bounds(1, 3, 1, 3).map_err(|e| e.into_failure("math_Matrix"))?;
    set_col_from(&mut syst, 1, matr, 1)?;
    set_col_from(&mut syst, 2, matr, 2)?;
    set_col_from(&mut syst, 3, matr, 4)?;
    let det: f64 = syst.determinant()?;
    set_col_from(&mut syst, 1, matr, 3)?;
    let det1: f64 = syst.determinant()?;
    set_col_from(&mut syst, 1, matr, 1)?;
    set_col_from(&mut syst, 2, matr, 3)?;
    let det2: f64 = syst.determinant()?;
    // dV1 = -dU1*aDet1/aDet, dV2 = -dU1*aDet2/aDet.
    if det * det1 > 0.0 {
        *v1_set = v1_decr;
    }
    if det * det2 > 0.0 {
        *v2_set = v2_decr;
    }
    Ok(())
}

/// `DeltaU1Computing(theSyst, theFree, theDeltaU1Found)`: the U1 step of the system
/// (Cramer on its second unknown) when its determinant is not null.
fn delta_u1_computing(syst: &Matrix<'_>, free: &Vector<'_>) -> Result<Option<f64>, Failure> {
    let det: f64 = syst.determinant()?;
    if !(det.abs() > NUL_VALUE) {
        return Ok(None);
    }
    let mut syst1: Matrix<'_> = syst.clone();
    syst1.set_col(2, free)?;
    Ok(Some((syst1.determinant()? / det).abs()))
}

/// `theMatr.Col(5) - aV * theMatr.Col(k)`.
fn free_column(matr: &Matrix<'_>, v: f64, k: i32) -> Result<Vector<'static>, Failure> {
    let mut out = matr.col(5)?;
    let col = matr.col(k)?;
    for i in 1..=3 {
        out[i] -= v * col[i];
    }
    Ok(out)
}

/// `StepComputing(theMatr, theV1Cur, theV2Cur, theDeltaV1, theDeltaV2,
/// theDeltaU1Found)`: the U1 step that moves V1 or V2 by its delta on the linearized
/// system `(a1 a2 a3 a4 | b)` of `V1, V2, dU1, dU2`; the smallest of the two, or the
/// least-squares-like choice of the overdetermined system when neither is solvable.
pub(in super::super) fn step_computing(
    matr: &Matrix<'_>,
    (v1_cur, v2_cur): (f64, f64),
    (delta_v1, delta_v2): (f64, f64),
) -> Result<Option<f64>, Failure> {
    let mut is_success: bool = false;
    let mut delta_u1: f64 = f64::MAX;
    // By default, V1(U1) and V2(U1) are considered increasing; what is indeed?
    let (mut v1_set, mut v2_set) = (v1_cur + delta_v1, v2_cur + delta_v2);
    v_boundary_precise(matr, (v1_cur - delta_v1, v2_cur - delta_v2), &mut v1_set, &mut v2_set)?;
    let mut syst = Matrix::with_bounds(1, 3, 1, 3).map_err(|e| e.into_failure("math_Matrix"))?;
    set_col_from(&mut syst, 2, matr, 3)?;
    set_col_from(&mut syst, 3, matr, 4)?;
    for i in 0..2 {
        let free = if i == 0 {
            // V1 is known.
            set_col_from(&mut syst, 1, matr, 2)?;
            free_column(matr, v1_set, 1)?
        } else {
            // V2 is known.
            set_col_from(&mut syst, 1, matr, 1)?;
            free_column(matr, v2_set, 2)?
        };
        if let Some(new_du) = delta_u1_computing(&syst, &free)? {
            is_success = true;
            if new_du < delta_u1 {
                delta_u1 = new_du;
            }
        }
    }
    if is_success {
        return Ok(Some(delta_u1));
    }
    overdetermined_step(matr, (v1_set, v2_set))
}

/// The overdetermined system of `dU1, dU2` with both V known: the couple of equations
/// of the greatest determinant.
fn overdetermined_step(matr: &Matrix<'_>, (v1_set, v2_set): (f64, f64)) -> Result<Option<f64>, Failure> {
    let m = |r: i32, c: i32| -> f64 { matr[(r, c)] };
    let (col5, col1, col2) = (matr.col(5)?, matr.col(1)?, matr.col(2)?);
    let free = |i: i32| -> f64 { col5[i] - v1_set * col1[i] - v2_set * col2[i] };
    let det1: f64 = m(1, 3) * m(2, 4) - m(2, 3) * m(1, 4);
    let det2: f64 = m(1, 3) * m(3, 4) - m(3, 3) * m(1, 4);
    let det3: f64 = m(2, 3) * m(3, 4) - m(3, 3) * m(2, 4);
    let (abs1, abs2, abs3) = (det1.abs(), det2.abs(), det3.abs());
    let by_det3 = || -> Option<f64> {
        (!(abs3 <= NUL_VALUE)).then(|| (free(2) * m(3, 4) - free(3) * m(2, 4)).abs() / abs3)
    };
    if abs1 >= abs2 {
        if abs1 >= abs3 {
            return Ok((!(abs1 <= NUL_VALUE)).then(|| (free(1) * m(2, 4) - free(2) * m(1, 4)).abs() / abs1));
        }
        return Ok(by_det3());
    }
    if abs2 >= abs3 {
        return Ok((!(abs2 <= NUL_VALUE)).then(|| (free(1) * m(3, 4) - free(3) * m(1, 4)).abs() / abs2));
    }
    Ok(by_det3())
}
