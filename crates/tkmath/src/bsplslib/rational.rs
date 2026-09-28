// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/BSplSLib/BSplSLib.cxx (RationalDerivative) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Derivatives of a rational surface f = Numerator / Denominator:
//! f(N,M) = (Numerator(N,M) - sum_{(p,q)<(N,M)} C(N,p) C(M,q) f(p,q) D(N-p,M-q)) / D(0,0).
use crate::plib::access::{non_negative, require_length};
use crate::plib::bin;
use rustycad_tkernel::standard::Failure;

const RATIONAL: &str = "BSplSLib::RationalDerivative";

/// Rational derivatives up to (N, M) from homogeneous derivatives `[x, y, z, w]`
/// stored `(VDeg+1)*4` values per U order (`BSplSLib::RationalDerivative`).
///
/// With `all` the (N+1)*(M+1) derivatives are written, `M+1` per U order, three
/// values each; otherwise only derivative (N, M). Orders above the degrees read as
/// zero; `bsplslib::rational_derivative(1, 1, 1, 1, &homogeneous, &mut rational, true)?`.
pub fn rational_derivative(
    u_deg: i32,
    v_deg: i32,
    n: i32,
    m: i32,
    h_derivatives: &[f64],
    r_derivatives: &mut [f64],
    all: bool,
) -> Result<(), Failure> {
    let (n1, m1): (usize, usize) = (
        non_negative(RATIONAL, "N", n)? + 1,
        non_negative(RATIONAL, "M", m)? + 1,
    );
    let min_n1: usize = (u_deg.min(n).max(-1) + 1) as usize;
    let min_m1: usize = (v_deg.min(m).max(-1) + 1) as usize;
    let m4: usize = (v_deg.max(-1) as usize + 1) * 4;
    let needed: usize = if min_n1 == 0 {
        4
    } else {
        (min_n1 - 1) * m4 + min_m1 * 4
    };
    require_length(RATIONAL, "HDerivatives", h_derivatives.len(), needed.max(4))?;
    let mut store: Vec<f64> = vec![0.0; n1 * m1 * 3];
    let mut weights: Vec<f64> = vec![0.0; n1 * m1];
    let orders = Orders {
        n1,
        m1,
        min_n1,
        min_m1,
        m4,
    };
    orders.split(h_derivatives, &mut store, &mut weights);
    orders.solve(&mut store, &weights, 1.0e0 / h_derivatives[3])?;
    if all {
        require_length(RATIONAL, "RDerivatives", r_derivatives.len(), store.len())?;
        r_derivatives[..store.len()].copy_from_slice(&store);
        return Ok(());
    }
    require_length(RATIONAL, "RDerivatives", r_derivatives.len(), 3)?;
    let index: usize = ((n1 - 1) * m1 + (m1 - 1)) * 3;
    r_derivatives[..3].copy_from_slice(&store[index..index + 3]);
    Ok(())
}

struct Orders {
    n1: usize,
    m1: usize,
    min_n1: usize,
    min_m1: usize,
    m4: usize,
}

impl Orders {
    // Numerators to the result rows and denominators to the weight rows; zeros beyond
    // the degrees.
    fn split(&self, homogeneous: &[f64], rational: &mut [f64], weights: &mut [f64]) {
        for ii in 0..self.min_n1 {
            for jj in 0..self.min_m1 {
                let source: usize = ii * self.m4 + jj * 4;
                let target: usize = (ii * self.m1 + jj) * 3;
                rational[target..target + 3].copy_from_slice(&homogeneous[source..source + 3]);
                weights[ii * self.m1 + jj] = homogeneous[source + 3];
            }
        }
    }

    fn solve(
        &self,
        rational: &mut [f64],
        weights: &[f64],
        denominator: f64,
    ) -> Result<(), Failure> {
        let (m1, m3): (usize, usize) = (self.m1, self.m1 * 3);
        for ii in 0..self.n1 {
            for jj in 0..m1 {
                let target: usize = ii * m3 + jj * 3;
                for pp in 0..ii {
                    let pip: f64 = bin(ii as i32, pp as i32)?;
                    for qq in 0..=jj {
                        // W(ii-pp, jj-qq) times f(pp, qq).
                        let w: f64 = weights[(ii - pp) * m1 + (jj - qq)];
                        let pjq: f64 = pip * bin(jj as i32, qq as i32)? * w;
                        subtract(rational, target, pp * m3 + qq * 3, pjq);
                    }
                }
                let pii: f64 = bin(ii as i32, ii as i32)?;
                for qq in 0..jj {
                    let pjq: f64 = pii * bin(jj as i32, qq as i32)? * weights[jj - qq];
                    subtract(rational, target, ii * m3 + qq * 3, pjq);
                }
                for value in &mut rational[target..target + 3] {
                    *value *= denominator;
                }
            }
        }
        Ok(())
    }
}

fn subtract(rational: &mut [f64], target: usize, source: usize, factor: f64) {
    for k in 0..3 {
        rational[target + k] -= factor * rational[source + k];
    }
}
