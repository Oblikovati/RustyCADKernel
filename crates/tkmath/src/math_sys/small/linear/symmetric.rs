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
use super::solve2;
pub(crate) fn symmetric_direction(
    j: &[[f64; 2]; 2],
    f: &[f64; 2],
    residual: f64,
) -> Option<[f64; 2]> {
    solve2(j, f)
        .or_else(|| svd(j, f, 1e-15))
        .or_else(|| fallback(j, f, residual))
}
fn fallback(j: &[[f64; 2]; 2], f: &[f64; 2], residual: f64) -> Option<[f64; 2]> {
    let scale: f64 = j[0][0].abs().max(j[0][1].abs()).max(j[1][1].abs());
    let a: [[f64; 2]; 2] = j.map(|row| row.map(|v| if scale > 0.0 { v / scale } else { 0.0 }));
    let fs: f64 = f[0].abs().max(f[1].abs());
    let g: [f64; 2] = [
        a[0][0] * (f[0] / fs) + a[0][1] * (f[1] / fs),
        a[0][1] * (f[0] / fs) + a[1][1] * (f[1] / fs),
    ];
    let length: f64 = g[0].hypot(g[1]);
    if length.partial_cmp(&1e-30).is_none_or(|v| v.is_lt()) {
        return None;
    }
    let alpha: f64 = if residual >= 10.0 * length {
        1.0
    } else {
        0.1 * residual / length
    };
    Some(g.map(|v| -alpha * v / length))
}
fn svd(j: &[[f64; 2]; 2], f: &[f64; 2], tol: f64) -> Option<[f64; 2]> {
    let scale: f64 = j[0][0].abs().max(j[0][1].abs()).max(j[1][1].abs());
    if scale <= 0.0 {
        return None;
    }
    let a: [[f64; 2]; 2] = j.map(|row| row.map(|v| v / scale));
    let b: [f64; 2] = f.map(|v| v / scale);
    if b.iter().any(|v| !v.is_finite()) {
        return None;
    }
    eigen_step(&a, &b, tol)
}
fn eigen_step(a: &[[f64; 2]; 2], b: &[f64; 2], tol: f64) -> Option<[f64; 2]> {
    let trace: f64 = a[0][0] + a[1][1];
    let diff: f64 = (a[0][0] - a[1][1]) * 0.5;
    let discrim: f64 = diff.hypot(a[0][1]);
    let lambdas: [f64; 2] = [trace * 0.5 + discrim, trace * 0.5 - discrim];
    let cutoff: f64 = lambdas[0].abs().max(lambdas[1].abs()) * tol;
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
    let d: [f64; 2] = [v1[0] * x[0] + v2[0] * x[1], v1[1] * x[0] + v2[1] * x[1]];
    d.iter().all(|v| v.is_finite()).then_some(d)
}
fn eigenvector(a: &[[f64; 2]; 2], diff: f64, lambda: f64, tol: f64) -> Option<[f64; 2]> {
    let v: [f64; 2] = if a[0][1].abs() > diff.abs() {
        [a[0][1], lambda - a[0][0]]
    } else {
        [lambda - a[1][1], a[0][1]]
    };
    let length: f64 = v[0].hypot(v[1]);
    if length < tol {
        return None;
    }
    Some(v.map(|x| x / length))
}
