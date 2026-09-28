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
use super::PIVOT_TOL;
pub(super) fn solve<const N: usize>(j: &[[f64; N]; N], f: &[f64; N]) -> Option<[f64; N]> {
    let (mut a, mut b): ([[f64; N]; N], [f64; N]) = scaled(j, f)?;
    for pivot in 0..N {
        let row: usize = (pivot + 1..N).fold(pivot, |best, r| {
            if a[r][pivot].abs() > a[best][pivot].abs() {
                r
            } else {
                best
            }
        });
        let magnitude: f64 = a[row][pivot].abs();
        if (N == 4 && magnitude <= PIVOT_TOL)
            || (N != 4 && magnitude.partial_cmp(&PIVOT_TOL) != Some(std::cmp::Ordering::Greater))
        {
            return None;
        }
        swap(&mut a, &mut b, row, pivot);
        eliminate(&mut a, &mut b, pivot);
    }
    let mut delta: [f64; N] = [0.0; N];
    for row in (0..N).rev() {
        let initial: f64 = if N == 4 { b[row] } else { -b[row] };
        let sum: f64 = (row + 1..N).fold(initial, |v, col| v - a[row][col] * delta[col]);
        delta[row] = sum / a[row][row];
    }
    delta.iter().all(|v| v.is_finite()).then_some(delta)
}
fn scaled<const N: usize>(j: &[[f64; N]; N], f: &[f64; N]) -> Option<([[f64; N]; N], [f64; N])> {
    let mut a: [[f64; N]; N] = [[0.0; N]; N];
    let mut b: [f64; N] = [0.0; N];
    for row in 0..N {
        let scale: f64 = j[row].iter().fold(0.0_f64, |v, x| v.max(x.abs()));
        if scale <= 0.0 {
            return None;
        }
        a[row] = j[row].map(|v| v / scale);
        b[row] = if N == 4 {
            -f[row] / scale
        } else {
            f[row] / scale
        };
        if !b[row].is_finite() {
            return None;
        }
    }
    Some((a, b))
}
// Native row/column indices retain the elimination update order.
#[allow(clippy::needless_range_loop)]
fn swap<const N: usize>(a: &mut [[f64; N]; N], b: &mut [f64; N], row: usize, pivot: usize) {
    for col in pivot..N {
        let saved: f64 = a[pivot][col];
        a[pivot][col] = a[row][col];
        a[row][col] = saved;
    }
    b.swap(row, pivot);
}
#[allow(clippy::needless_range_loop)]
fn eliminate<const N: usize>(a: &mut [[f64; N]; N], b: &mut [f64; N], pivot: usize) {
    let inverse: f64 = 1.0 / a[pivot][pivot];
    for row in pivot + 1..N {
        let factor: f64 = if N == 4 {
            a[row][pivot] * inverse
        } else {
            a[row][pivot] / a[pivot][pivot]
        };
        for col in pivot + 1..N {
            a[row][col] -= factor * a[pivot][col];
        }
        b[row] -= factor * b[pivot];
        if N == 4 {
            a[row][pivot] = 0.0;
        }
    }
}
