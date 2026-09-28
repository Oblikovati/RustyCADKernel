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
// Rust translation of ModelingData/TKGeomBase/AdvApp2Var/AdvApp2Var_MathBase.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AdvApp2Var_MathBase`: the f2c polynomial, Jacobi and linear-algebra routines.
//!
//! Every public static member is a free function with the native name; the
//! file-static helpers of AdvApp2Var_MathBase.cxx sit in the module of their
//! caller. The modules group the routines by kernel family so that each file stays
//! small: curve reparametrisation, coefficient packing, derivatives, evaluation,
//! Gauss integration, Hermite and Jacobi bases, dense and sparse linear systems,
//! sorting, and vectors.
//!
//! The precision common `MMPRCSN` is the one piece of process-global state that
//! the port keeps global: see [`mmwprcs_`].
#![allow(clippy::too_many_arguments)] // The f2c routines keep the Fortran argument lists.

mod approx_errors;
mod bases;
mod compress;
mod derivatives;
mod evaluation;
mod gauss;
mod hermite;
mod jacobi;
mod linear;
mod reparam;
mod sorting;
mod sparse;
mod truncation;
mod vectors;

pub use approx_errors::mmaperx_;
pub use bases::{mmhjcan_, mmpobas_};
pub use compress::{mmapcmp_, mmfmca8_, mmfmca9_, mmfmcar_, mmfmcb5_, mmfmtb1_};
pub use derivatives::{mmcdriv_, mmdrc11_, mmdrvck_};
pub use evaluation::{mmmpocur_, mmpocrb_};
pub use gauss::{GaussIntegrand, mmcglc1_, mmgaus1_, mmrtptt_};
pub use jacobi::{mmjacan_, mmjaccv_};
pub use linear::{mmcvctx_, mmmrslwd_};
pub use reparam::{mmarcin_, mmcvinv_};
pub use sorting::{mmbulld_, mminltt_, mmsrre2_, mvsheld_};
pub use sparse::{ProfileSystem, mmposui_, mmresol_};
pub use truncation::mmtrpjj_;
pub use vectors::{mdsptpt_, mmunivt_, mmvncol_};

use super::f2c::{F2cArray, pow_by_loop};
use std::sync::RwLock;

/// The precision common `MMPRCSN`: tolerances and iteration counts of the kernels.
///
/// `mmprcsn().eps1 == 1e-9`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mmprcsn {
    /// Metric tolerance (`eps1`), read by [`mmeps1_`].
    pub eps1: f64,
    /// Relative tolerance (`eps2`).
    pub eps2: f64,
    /// Parametric tolerance (`eps3`), read by [`mmveps3_`].
    pub eps3: f64,
    /// Angular tolerance (`eps4`).
    pub eps4: f64,
    /// Maximum iterations of the fast methods (`niterm`).
    pub niterm: i32,
    /// Maximum iterations of the slow methods (`niterr`).
    pub niterr: i32,
}

/// The values the native static initialiser of AdvApp2Var_MathBase.cxx passes to
/// `mmwprcs_` when the library loads.
pub const MMPRCSN_DEFAULT: Mmprcsn = Mmprcsn {
    eps1: 1.0e-9,
    eps2: 1.0e-8,
    eps3: 1.0e-9,
    eps4: 1.0e-4,
    niterm: 8,
    niterr: 40,
};

// Process-global on purpose: `mmwprcs_` is a public entry point whose native
// contract is to change what every later `mmeps1_`/`mmveps3_` returns, including the
// calls deep inside `AdvApp2Var_ApproxAFunc2Var`; a per-call value could not honour it.
static MMPRCSN: RwLock<Mmprcsn> = RwLock::new(MMPRCSN_DEFAULT);

/// A copy of the current precision common; `mmprcsn().eps3`.
pub fn mmprcsn() -> Mmprcsn {
    *MMPRCSN
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Replaces the precision common (`mmwprcs_`); it stays in force for the whole
/// process, as natively. `mmwprcs_(1e-9, 1e-8, 1e-9, 1e-4, 8, 40)`.
pub fn mmwprcs_(epsil1: f64, epsil2: f64, epsil3: f64, epsil4: f64, niter1: i32, niter2: i32) {
    let mut common = MMPRCSN
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *common = Mmprcsn {
        eps1: epsil1,
        eps2: epsil2,
        eps3: epsil3,
        eps4: epsil4,
        niterm: niter1,
        niterr: niter2,
    };
}

/// The metric tolerance `eps1` (`mmeps1_`); returns 0. `mmeps1_(&mut eps)`.
pub fn mmeps1_(epsilo: &mut f64) -> i32 {
    *epsilo = mmprcsn().eps1;
    0
}

/// The parametric tolerance `eps3` (`mmveps3_`); returns 0. `mmveps3_(&mut eps)`.
pub fn mmveps3_(eps03: &mut f64) -> i32 {
    *eps03 = mmprcsn().eps3;
    0
}

/// `x` to the integer power `n` by repeated multiplication (`pow__di`), inverted for
/// a negative `n`; `pow__di(2.0, -1) == 0.5`.
#[allow(non_snake_case)] // The f2c name, kept for traceability.
pub fn pow__di(x: f64, n: i32) -> f64 {
    pow_by_loop(x, n)
}

/// The integer power of the file-static `pow__ii`, with C integer division for a
/// negative exponent; `pow__ii(2, 3) == 8`.
#[allow(non_snake_case)] // The f2c name, kept for traceability.
fn pow__ii(x: i32, n: i32) -> i32 {
    let mut result: i32 = 1;
    for _ in 0..n.unsigned_abs() {
        result *= x;
    }
    if n < 0 {
        result = 1 / result;
    }
    result
}

/// The scalar product of the first `ndimen` components (`msc_`);
/// `msc_(2, &[1.0, 2.0], &[3.0, 4.0]) == 11.0`.
pub fn msc_(ndimen: i32, vecte1: &[f64], vecte2: &[f64]) -> f64 {
    let (vecte1, vecte2) = (F2cArray::new(vecte1, 1), F2cArray::new(vecte2, 1));
    let mut x: f64 = 0.;
    for i in 1..=ndimen {
        x += vecte1[i] * vecte2[i];
    }
    x
}

/// The Euclidean norm of the first `ndimen` components, scaled by the largest one
/// when it is at least 1 to avoid overflow (`mzsnorm_`); `mzsnorm_(2, &[3.0, 4.0])`.
pub fn mzsnorm_(ndimen: i32, vecteu: &[f64]) -> f64 {
    let vecteu = F2cArray::new(vecteu, 1);
    let mut irmax: i32 = 1;
    for i in 2..=ndimen {
        if vecteu[irmax].abs() < vecteu[i].abs() {
            irmax = i;
        }
    }
    let mut xsom: f64 = 0.;
    if vecteu[irmax].abs() < 1. {
        for i in 1..=ndimen {
            let d = vecteu[i];
            xsom += d * d;
        }
        return xsom.sqrt();
    }
    for i in 1..=ndimen {
        if i == irmax {
            xsom += 1.;
        } else {
            let d = vecteu[i] / vecteu[irmax];
            xsom += d * d;
        }
    }
    vecteu[irmax].abs() * xsom.sqrt()
}

/// The factorials `f[k-1] = k!` for `k = 1..=n` (file-static `mfac_`), `f[0]` being
/// the single-precision literal 1.
fn mfac_(f: &mut [f64], n: i32) {
    f[0] = 1.0_f32 as f64;
    for i in 2..=n {
        let k: usize = (i - 1) as usize;
        f[k] = f64::from(i) * f[k - 1];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn norms_scale_by_the_largest_component() {
        assert_eq!(mzsnorm_(2, &[3.0, 4.0]), 5.0);
        assert_eq!(mzsnorm_(2, &[0.3, 0.4]), (0.3f64 * 0.3 + 0.4 * 0.4).sqrt());
        assert_eq!(mzsnorm_(1, &[-2.0]), 2.0);
        assert_eq!(msc_(2, &[1.0, 2.0], &[3.0, 4.0]), 11.0);
    }

    #[test]
    fn powers_and_factorials_follow_the_integer_loops() {
        assert_eq!(pow__di(2.0, -1), 0.5);
        assert_eq!(pow__ii(2, 5), 32);
        assert_eq!(pow__ii(2, -1), 0);
        let mut f: [f64; 4] = [0.0; 4];
        mfac_(&mut f, 4);
        assert_eq!(f, [1.0, 2.0, 6.0, 24.0]);
    }

    #[test]
    fn the_default_precision_is_the_static_initialiser() {
        let (mut eps1, mut eps3) = (0.0, 0.0);
        mmeps1_(&mut eps1);
        mmveps3_(&mut eps3);
        assert_eq!((eps1, eps3), (1.0e-9, 1.0e-9));
    }
}
