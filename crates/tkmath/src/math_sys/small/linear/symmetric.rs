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

//! Symmetric 2x2 steps of MathSys_Newton2D.hxx (`SolveSymmetric2x2SVD` and the
//! `Solve2DSymmetric` fallback chain) at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::solve_2x2;
use crate::math_sys::small::checks::{CRITICAL_GRAD, list_max, std_max};

/// Solves the symmetric system `[[j11, j12], [j12, j22]] delta = -F` through its eigen
/// decomposition, dropping eigenvalues below `tol * max |lambda|` (`SolveSymmetric2x2SVD`,
/// native default `tol = 1e-15`). Outputs are written before the finiteness test;
/// `solve_symmetric_2x2_svd(2.0, 0.0, 4.0, 2.0, 4.0, &mut du, &mut dv, 1e-15)` gives `(-1, -1)`.
#[allow(clippy::too_many_arguments)] // Keeps the upstream eight-argument contract.
pub fn solve_symmetric_2x2_svd(
    j11: f64,
    j12: f64,
    j22: f64,
    f1: f64,
    f2: f64,
    du: &mut f64,
    dv: &mut f64,
    tol: f64,
) -> bool {
    let scale: f64 = list_max(&[j11.abs(), j12.abs(), j22.abs()]);
    if scale.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
        return false;
    }
    let a: [f64; 3] = [j11 / scale, j12 / scale, j22 / scale];
    let b: [f64; 2] = [f1 / scale, f2 / scale];
    if !b[0].is_finite() || !b[1].is_finite() {
        return false;
    }
    let Some(d) = eigen_step(&a, &b, tol) else {
        return false;
    };
    (*du, *dv) = (d[0], d[1]);
    du.is_finite() && dv.is_finite()
}
fn eigen_step(a: &[f64; 3], b: &[f64; 2], tol: f64) -> Option<[f64; 2]> {
    let trace: f64 = a[0] + a[2];
    let diff: f64 = (a[0] - a[2]) * 0.5;
    let discrim: f64 = diff.hypot(a[1]);
    let lambdas: [f64; 2] = [trace * 0.5 + discrim, trace * 0.5 - discrim];
    let cutoff: f64 = std_max(lambdas[0].abs(), lambdas[1].abs()) * tol;
    if lambdas.iter().all(|v| v.abs() < cutoff) {
        return None;
    }
    let v1: [f64; 2] = eigenvector(a, diff, lambdas[0], tol)?;
    let v2: [f64; 2] = [-v1[1], v1[0]];
    let projected: [f64; 2] = [
        v1[0] * (-b[0]) + v1[1] * (-b[1]),
        v2[0] * (-b[0]) + v2[1] * (-b[1]),
    ];
    let x: [f64; 2] = std::array::from_fn(|i| {
        if lambdas[i].abs() > cutoff {
            projected[i] / lambdas[i]
        } else {
            0.0
        }
    });
    Some([v1[0] * x[0] + v2[0] * x[1], v1[1] * x[0] + v2[1] * x[1]])
}
fn eigenvector(a: &[f64; 3], diff: f64, lambda: f64, tol: f64) -> Option<[f64; 2]> {
    let v: [f64; 2] = if a[1].abs() > diff.abs() {
        [a[1], lambda - a[0]]
    } else {
        [lambda - a[2], a[1]]
    };
    let length: f64 = v[0].hypot(v[1]);
    if length < tol {
        return None;
    }
    Some(v.map(|x| x / length))
}

/// `Solve2DSymmetric` direction: general 2x2, then the eigen solve, then a scaled
/// steepest-descent step.
pub(crate) fn symmetric_direction(
    j: &[[f64; 2]; 2],
    f: &[f64; 2],
    residual: f64,
) -> Option<[f64; 2]> {
    let (mut du, mut dv): (f64, f64) = (0.0, 0.0);
    if solve_2x2(j, f, &mut du, &mut dv)
        || solve_symmetric_2x2_svd(
            j[0][0], j[0][1], j[1][1], f[0], f[1], &mut du, &mut dv, 1.0e-15,
        )
    {
        return Some([du, dv]);
    }
    fallback(j, f, residual)
}
fn fallback(j: &[[f64; 2]; 2], f: &[f64; 2], residual: f64) -> Option<[f64; 2]> {
    let scale: f64 = list_max(&[j[0][0].abs(), j[0][1].abs(), j[1][1].abs()]);
    let a: [[f64; 2]; 2] = j.map(|row| row.map(|v| if scale > 0.0 { v / scale } else { 0.0 }));
    let fs: f64 = std_max(f[0].abs(), f[1].abs());
    let g: [f64; 2] = [
        a[0][0] * (f[0] / fs) + a[0][1] * (f[1] / fs),
        a[0][1] * (f[0] / fs) + a[1][1] * (f[1] / fs),
    ];
    let length: f64 = g[0].hypot(g[1]);
    // `!(|g| >= CRITICAL_GRAD)` also reports a NaN gradient as singular.
    if length.partial_cmp(&CRITICAL_GRAD).is_none_or(|v| v.is_lt()) {
        return None;
    }
    let alpha: f64 = if residual >= 10.0 * length {
        1.0
    } else {
        0.1 * residual / length
    };
    Some(g.map(|v| -alpha * v / length))
}
