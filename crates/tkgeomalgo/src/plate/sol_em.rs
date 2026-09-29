// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/Plate/Plate_Plate.cxx (SolEm) and
// Plate_Plate.lxx (Polm) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The two kernels of the plate: `SolEm`, the derivatives of the fundamental solution
//! of the iterated Laplacian, and `Polm`, the derivatives of the monomials.
use super::PlatePlate;
use super::sol_em_terms::{SolEmTerms, duv};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::gp::Xy;

/// The `(Uold, Vold, U2, R, L)` members that `SolEm` updates through a `const_cast`:
/// the last scaled point, `U * U`, `R = U2 + V * V` and `ln R`.
#[derive(Clone, Copy, Debug)]
pub(super) struct SolEmCache {
    u_old: f64,
    v_old: f64,
    u2: f64,
    r: f64,
    l: f64,
}

impl Default for SolEmCache {
    /// The constructor values: `Uold = Vold = 1e20` and null `U2`, `R`, `L`.
    fn default() -> Self {
        Self {
            u_old: 1.0e20,
            v_old: 1.0e20,
            u2: 0.0,
            r: 0.0,
            l: 0.0,
        }
    }
}

impl SolEmCache {
    /// The cached `(U2, R, L)` of `(u, v)`, updating the cache on a new point; `None`
    /// when `R < 1e-20`. A fresh cache holds `(1e20, 1e20)` with `R = 0`, so a first
    /// call at exactly that point returns `None`, as natively.
    fn terms(&mut self, u: f64, v: f64) -> Option<(f64, f64, f64)> {
        if u == self.u_old && v == self.v_old {
            if self.r < 1.0e-20 {
                return None;
            }
        } else {
            self.u_old = u;
            self.v_old = v;
            self.u2 = u * u;
            self.r = self.u2 + v * v;
            if self.r < 1.0e-20 {
                return None;
            }
            self.l = self.r.ln();
        }
        Some((self.u2, self.r, self.l))
    }
}

/// `R^expo` by repeated products (`1 / R^-expo` for a negative exponent), as the native
/// code computes it instead of `pow`.
fn integer_power(r: f64, expo: i32) -> f64 {
    if expo == 0 {
        return 1.0;
    }
    let mut pr: f64 = r;
    for _ in 1..expo.abs() {
        pr *= r;
    }
    if expo < 0 { 1.0 / pr } else { pr }
}

impl PlatePlate {
    /// `SolEm`: the `(iu, iv)` derivative of the fundamental solution at `point2d`,
    /// scaled by `ddu[iu] * ddv[iv]`; the case table is symmetric, so `iv > iu` swaps
    /// the coordinates.
    ///
    /// Standard_OutOfRange for a derivative order outside `0..=9`, where the native
    /// `ddu`/`ddv` read is undefined.
    pub(super) fn sol_em(&self, point2d: Xy, iu: i32, iv: i32) -> Result<f64, Failure> {
        let (ddu1, ddv1) = (self.ddu[1], self.ddv[1]);
        let (big_iu, big_iv, u, v) = if iv > iu {
            (iv, iu, point2d.y() * ddv1, point2d.x() * ddu1)
        } else {
            (iu, iv, point2d.x() * ddu1, point2d.y() * ddv1)
        };
        let cached = self
            .sol_em_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .terms(u, v);
        let Some((u2, r, l)) = cached else {
            return Ok(0.0);
        };
        let (ddu_iu, ddv_iv) = (self.scale_u(iu)?, self.scale_v(iv)?);
        let m: i32 = self.order;
        let terms = SolEmTerms {
            u,
            v,
            u2,
            r,
            l,
            pr: integer_power(r, m - 1 - big_iu - big_iv),
            m: f64::from(m),
            mm1: f64::from(m - 1),
        };
        Ok(duv(big_iu, big_iv, &terms) * ddu_iu * ddv_iv)
    }

    /// `ddu[iu]`, Standard_OutOfRange outside the ten scale factors.
    pub(super) fn scale_u(&self, iu: i32) -> Result<f64, Failure> {
        scale_factor(&self.ddu, iu, "U")
    }

    /// `ddv[iv]`, Standard_OutOfRange outside the ten scale factors.
    pub(super) fn scale_v(&self, iv: i32) -> Result<f64, Failure> {
        scale_factor(&self.ddv, iv, "V")
    }

    /// `Polm`: the `(idu, idv)` derivative of the monomial `u^iu v^iv` at `point2d`,
    /// scaled by `ddu[iu] * ddv[iv]` (which the native comment says improves the
    /// conditioning).
    pub(super) fn polm(
        &self,
        point2d: Xy,
        iu: i32,
        iv: i32,
        idu: i32,
        idv: i32,
    ) -> Result<f64, Failure> {
        if idu > iu || idv > iv {
            return Ok(0.0);
        }
        let (u, v) = (point2d.x(), point2d.y());
        let mut value: f64 = 1.0;
        let degu: i32 = iu - idu;
        for _ in 0..degu {
            value *= u;
        }
        for i in (degu + 1..=iu).rev() {
            value *= f64::from(i);
        }
        let degv: i32 = iv - idv;
        for _ in 0..degv {
            value *= v;
        }
        for i in (degv + 1..=iv).rev() {
            value *= f64::from(i);
        }
        Ok(value * self.scale_u(iu)? * self.scale_v(iv)?)
    }
}

fn scale_factor(factors: &[f64; 10], order: i32, direction: &str) -> Result<f64, Failure> {
    let slot = usize::try_from(order).ok().and_then(|i| factors.get(i));
    slot.copied().ok_or_else(|| {
        errors::out_of_range(&format!(
            "Plate_Plate: derivative order {order} in {direction}; expected in [0, 9]"
        ))
    })
}
