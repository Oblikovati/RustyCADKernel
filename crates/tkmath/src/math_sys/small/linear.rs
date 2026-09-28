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

//! Port of MathSys_NewtonTypes/2D/3D/4D at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::merit::norm;
mod elimination;
mod symmetric;
pub(super) use symmetric::symmetric_direction;
const PIVOT_TOL: f64 = 64.0 * f64::EPSILON;
pub(super) fn solve2(j: &[[f64; 2]; 2], f: &[f64; 2]) -> Option<[f64; 2]> {
    let scales: [f64; 2] = [
        j[0][0].abs().max(j[0][1].abs()),
        j[1][0].abs().max(j[1][1].abs()),
    ];
    if scales.iter().any(|s| *s <= 0.0) {
        return None;
    }
    let a: [[f64; 2]; 2] = std::array::from_fn(|r| j[r].map(|v| v / scales[r]));
    let b: [f64; 2] = std::array::from_fn(|r| f[r] / scales[r]);
    let det: f64 = a[0][0] * a[1][1] - a[0][1] * a[1][0];
    if b.iter().any(|v| !v.is_finite())
        || det.abs().partial_cmp(&PIVOT_TOL) != Some(std::cmp::Ordering::Greater)
    {
        return None;
    }
    let d: [f64; 2] = [
        (-b[0] * a[1][1] + b[1] * a[0][1]) / det,
        (-b[1] * a[0][0] + b[0] * a[1][0]) / det,
    ];
    d.iter().all(|v| v.is_finite()).then_some(d)
}
pub(super) fn direction<const N: usize>(
    j: &[[f64; N]; N],
    f: &[f64; N],
    residual: f64,
) -> Option<[f64; N]> {
    let solved: Option<[f64; N]> = if N == 2 {
        solve2(&[[j[0][0], j[0][1]], [j[1][0], j[1][1]]], &[f[0], f[1]])
            .map(|d| std::array::from_fn(|i| d[i]))
    } else {
        elimination::solve(j, f)
    };
    solved.or_else(|| gradient_step(j, f, residual))
}
fn gradient_step<const N: usize>(
    j: &[[f64; N]; N],
    f: &[f64; N],
    residual: f64,
) -> Option<[f64; N]> {
    let gradient: [f64; N] = std::array::from_fn(|col| {
        if N < 4 {
            (1..N).fold(j[0][col] * f[0], |a, row| a + j[row][col] * f[row])
        } else {
            (0..N).fold(0.0, |a, row| a + j[row][col] * f[row])
        }
    });
    let length: f64 = if N == 2 {
        gradient[0].hypot(gradient[1])
    } else {
        norm(&gradient)
    };
    if length < 1e-30 {
        return None;
    }
    let alpha: f64 = 1.0_f64.min(residual / length * 0.1);
    Some(gradient.map(|g| -alpha * g))
}
