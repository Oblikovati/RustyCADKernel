// Copyright (c) 1998-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_FunctionDraft.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_FunctionDraft`: the system `C(w) - S(u, v) = 0` that intersects a
//! draft generatrix `C` with a stop surface `S`, and its derivatives with respect
//! to the sweep parameter.
use super::sweep_failures::live;
use super::tensor::GeomFillTensor;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::Vector3;
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector};

/// `GeomFill_FunctionDraft`: a `math_FunctionSetWithDerivatives` of `(w, u, v)`.
///
/// A curve or surface exception inside a solver callback is kept and the callback
/// answers false; [`GeomFillFunctionDraft::take_failure`] raises it after the solver
/// (docs/geomalgo-design.md, section 14).
/// `let mut f = GeomFillFunctionDraft::new(&surface, &line);`.
pub struct GeomFillFunctionDraft {
    the_curve: Handle<dyn Adaptor3dCurve>,
    the_surface: Handle<dyn Adaptor3dSurface>,
    capture: FailureCapture,
}

fn coords(v: Vector3) -> [f64; 3] {
    [v.x(), v.y(), v.z()]
}

/// `F(i) = a(i) + W * dN(i) * sin(teta)` for `i` in 1..3.
fn set_shifted(
    f: &mut Vector<'_>,
    a: Vector3,
    w: f64,
    dn: Vector3,
    teta: f64,
) -> Result<(), Failure> {
    let (a, dn) = (coords(a), coords(dn));
    for i in 0..3 {
        *f.change_value(i as i32 + 1)? = a[i] + w * dn[i] * teta.sin();
    }
    Ok(())
}

impl GeomFillFunctionDraft {
    /// `GeomFill_FunctionDraft(S, C)`; `GeomFillFunctionDraft::new(&surface, &curve)`.
    pub fn new(s: &Handle<dyn Adaptor3dSurface>, c: &Handle<dyn Adaptor3dCurve>) -> Self {
        Self {
            the_curve: c.clone(),
            the_surface: s.clone(),
            capture: FailureCapture::default(),
        }
    }

    /// The exception a solver callback met, if any; `f.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.capture.take()
    }

    fn curve(&self) -> Result<&dyn Adaptor3dCurve, Failure> {
        live(&self.the_curve, "GeomFill_FunctionDraft", "TheCurve")
    }

    fn surface(&self) -> Result<&dyn Adaptor3dSurface, Failure> {
        live(&self.the_surface, "GeomFill_FunctionDraft", "TheSurface")
    }

    /// `Value(X, F)`: `C(X(1)) - S(X(2), X(3))`.
    pub fn try_value(&self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        let p = self.curve()?.d0(*x.value(1)?)?;
        let p1 = self.surface()?.d0(*x.value(2)?, *x.value(3)?)?;
        *f.change_value(1)? = p.x() - p1.x();
        *f.change_value(2)? = p.y() - p1.y();
        *f.change_value(3)? = p.z() - p1.z();
        Ok(true)
    }

    /// `Derivatives(X, D)`: the Jacobian `[C', -S_u, -S_v]`.
    pub fn try_derivatives(&self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure> {
        let (_, dp) = self.curve()?.d1(*x.value(1)?)?;
        let s = self.surface()?.d1(*x.value(2)?, *x.value(3)?)?;
        let (dp, du, dv) = (coords(dp), coords(s.d1u), coords(s.d1v));
        for i in 0..3 {
            let row = i as i32 + 1;
            *d.change_value(row, 1)? = dp[i];
            *d.change_value(row, 2)? = -du[i];
            *d.change_value(row, 3)? = -dv[i];
        }
        Ok(true)
    }

    /// `Values(X, F, D)`: both in one evaluation.
    pub fn try_values(
        &self,
        x: &Vector<'_>,
        f: &mut Vector<'_>,
        d: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        let (p, dp) = self.curve()?.d1(*x.value(1)?)?;
        let s = self.surface()?.d1(*x.value(2)?, *x.value(3)?)?;
        let (pc, p1) = (
            coords(Vector3::from(p.xyz())),
            coords(Vector3::from(s.point.xyz())),
        );
        let (dp, du, dv) = (coords(dp), coords(s.d1u), coords(s.d1v));
        for i in 0..3 {
            let row = i as i32 + 1;
            *f.change_value(row)? = pc[i] - p1[i];
            *d.change_value(row, 1)? = dp[i];
            *d.change_value(row, 2)? = -du[i];
            *d.change_value(row, 3)? = -dv[i];
        }
        Ok(true)
    }

    /// `DerivT(C, Param, W, dN, teta, F)`: `dE/dt = C'(Param) + W sin(teta) dN`.
    pub fn deriv_t(
        &self,
        c: &Handle<dyn Adaptor3dCurve>,
        param: f64,
        w: f64,
        dn: Vector3,
        teta: f64,
        f: &mut Vector<'_>,
    ) -> Result<bool, Failure> {
        let (_, dp) = live(c, "GeomFill_FunctionDraft::DerivT", "C")?.d1(param)?;
        set_shifted(f, dp, w, dn, teta)?;
        Ok(true)
    }

    /// `Deriv2T(C, Param, W, d2N, teta, F)`: `d2E/dt2 = C''(Param) + W sin(teta) d2N`.
    pub fn deriv2_t(
        &self,
        c: &Handle<dyn Adaptor3dCurve>,
        param: f64,
        w: f64,
        d2n: Vector3,
        teta: f64,
        f: &mut Vector<'_>,
    ) -> Result<bool, Failure> {
        let (_, _, d2p) = live(c, "GeomFill_FunctionDraft::Deriv2T", "C")?.d2(param)?;
        set_shifted(f, d2p, w, d2n, teta)?;
        Ok(true)
    }

    /// `DerivTX(dN, teta, D)`: `d2E/dtdx`, only the `w` column is not null.
    pub fn deriv_tx(&self, dn: Vector3, teta: f64, d: &mut Matrix<'_>) -> Result<bool, Failure> {
        let dn = coords(dn);
        for (i, component) in dn.iter().enumerate() {
            let row = i as i32 + 1;
            *d.change_value(row, 1)? = component * teta.sin();
            *d.change_value(row, 2)? = 0.0;
            *d.change_value(row, 3)? = 0.0;
        }
        Ok(true)
    }

    /// `Deriv2X(X, T)`: `d2E/dx2`, the second derivatives of the surface.
    pub fn deriv2_x(&self, x: &Vector<'_>, t: &mut GeomFillTensor) -> Result<bool, Failure> {
        let s = self.surface()?.d2(*x.value(2)?, *x.value(3)?)?;
        t.init(0.0);
        let (d2u, d2v, d2uv) = (coords(s.d2u), coords(s.d2v), coords(s.d2uv));
        for i in 0..3 {
            let row = i as i32 + 1;
            *t.change_value(row, 2, 2)? = -d2u[i];
            *t.change_value(row, 2, 3)? = -d2uv[i];
            *t.change_value(row, 3, 2)? = -d2uv[i];
            *t.change_value(row, 3, 3)? = -d2v[i];
        }
        Ok(true)
    }
}

impl FunctionSet for GeomFillFunctionDraft {
    /// Always 3.
    fn variables(&self) -> i32 {
        3
    }
    /// Always 3.
    fn equations(&self) -> i32 {
        3
    }
    fn value(&mut self, point: &Vector<'_>, residual: &mut Vector<'_>) -> bool {
        let result = self.try_value(point, residual);
        self.capture.keep(result).unwrap_or(false)
    }
}

impl FunctionSetWithDerivatives for GeomFillFunctionDraft {
    fn derivatives(&mut self, point: &Vector<'_>, jacobian: &mut Matrix<'_>) -> bool {
        let result = self.try_derivatives(point, jacobian);
        self.capture.keep(result).unwrap_or(false)
    }
    fn values(
        &mut self,
        point: &Vector<'_>,
        residual: &mut Vector<'_>,
        jacobian: &mut Matrix<'_>,
    ) -> bool {
        let result = self.try_values(point, residual, jacobian);
        self.capture.keep(result).unwrap_or(false)
    }
}
