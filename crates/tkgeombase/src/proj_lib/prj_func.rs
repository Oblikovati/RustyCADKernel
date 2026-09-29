// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/ProjLib/ProjLib_PrjFunc.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ProjLib_PrjFunc`: `F = ((S(u,v) - C(t)).Su, (S(u,v) - C(t)).Sv) * Norm` in two
//! of the three variables (t, u, v), the third being fixed; its roots are the
//! orthogonal projections of the curve on the surface.
use crate::failure_capture::FailureCapture;
use crate::native_failure::failure;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::GeomSurfaceD2;
use rustycad_tkmath::gp::{Point2, Vector3};
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector};

/// `ProjLib_PrjFunc`: `fix` 1 fixes t (variables u, v), 2 fixes u (variables t, v)
/// and 3 fixes v (variables t, u). The math callbacks keep the first failure an
/// adaptor raises ([`PrjFunc::take_failure`]).
///
/// ```ignore
/// let mut function = PrjFunc::new(&curve, 0.5, &surface, 1)?;
/// function.values(&x, &mut f, &mut d)?;
/// let uv = function.solution();
/// ```
pub struct PrjFunc<'a> {
    curve: &'a dyn Adaptor3dCurve,
    surface: &'a dyn Adaptor3dSurface,
    t: f64,
    u: f64,
    v: f64,
    fix: i32,
    norm: f64,
    failure: FailureCapture,
}

impl<'a> PrjFunc<'a> {
    /// `ProjLib_PrjFunc(C, FixVal, S, Fix)`: Standard_ConstructionError unless `fix`
    /// is 1, 2 or 3; `PrjFunc::new(&curve, 0.5, &surface, 1)?`.
    pub fn new(
        curve: &'a dyn Adaptor3dCurve,
        fix_val: f64,
        surface: &'a dyn Adaptor3dSurface,
        fix: i32,
    ) -> Result<Self, Failure> {
        let resolution: f64 = surface.u_resolution(1.)?.min(surface.v_resolution(1.)?);
        let mut function: Self = Self {
            curve,
            surface,
            t: 0.,
            u: 0.,
            v: 0.,
            fix,
            norm: 1.0_f64.min(resolution),
            failure: FailureCapture::default(),
        };
        match fix {
            1 => function.t = fix_val,
            2 => function.u = fix_val,
            3 => function.v = fix_val,
            _ => return Err(bad_fix(fix)),
        }
        Ok(function)
    }
    /// `NbVariables()`: 2; `function.nb_variables()`.
    pub fn nb_variables(&self) -> i32 {
        2
    }
    /// `NbEquations()`: 2; `function.nb_equations()`.
    pub fn nb_equations(&self) -> i32 {
        2
    }
    /// `Value(X, F)`: `Values` with a scratch matrix; `function.value(&x, &mut f)?`.
    pub fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        let mut d: Matrix<'static> = Matrix::with_bounds(1, 2, 1, 2)
            .map_err(|error| error.into_failure("ProjLib_PrjFunc::Value"))?;
        self.values(x, f, &mut d)
    }
    /// `Derivatives(X, D)`: `Values` with a scratch vector;
    /// `function.derivatives(&x, &mut d)?`.
    pub fn derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure> {
        let mut f: Vector<'static> = Vector::with_bounds(1, 2)?;
        self.values(x, &mut f, d)
    }
    /// `Solution()`: the (u, v), (t, v) or (t, u) of the last evaluation;
    /// `function.solution()`.
    pub fn solution(&self) -> Point2 {
        match self.fix {
            1 => Point2::new(self.u, self.v),
            2 => Point2::new(self.t, self.v),
            _ => Point2::new(self.t, self.u),
        }
    }
    /// The first failure a math callback kept since the last call;
    /// `function.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.failure.take()
    }
}

fn bad_fix(fix: i32) -> Failure {
    failure(
        "Standard_ConstructionError",
        &format!("ProjLib_PrjFunc: fixed variable index {fix}; expected 1, 2 or 3"),
    )
}

impl PrjFunc<'_> {
    /// `Values(X, F, D)`: stores the evaluated (t, u, v) and always returns true;
    /// `function.values(&x, &mut f, &mut d)?`.
    pub fn values(
        &mut self,
        x: &Vector<'_>,
        f: &mut Vector<'_>,
        d: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        let (t, u, v): (f64, f64, f64) = match self.fix {
            1 => (self.t, x[1], x[2]),
            2 => (x[1], self.u, x[2]),
            _ => (x[1], x[2], self.v),
        };
        let (c, dc1_t) = self.curve.d1(t)?;
        let s: GeomSurfaceD2 = self.surface.d2(u, v)?;
        let ort: Vector3 = Vector3::between(c, s.point);
        f[1] = ort.dot(s.d1u) * self.norm;
        f[2] = ort.dot(s.d1v) * self.norm;
        self.fill_jacobian(d, &s, ort, dc1_t);
        (self.u, self.v, self.t) = (u, v, t);
        Ok(true)
    }

    // The Jacobian of the two equations in the free variables, in the native order.
    fn fill_jacobian(&self, d: &mut Matrix<'_>, s: &GeomSurfaceD2, ort: Vector3, dc1_t: Vector3) {
        let norm: f64 = self.norm;
        let neg_dc: Vector3 = dc1_t.reversed();
        let de1_du: f64 = (s.d1u.square_magnitude() + ort.dot(s.d2u)) * norm;
        let de_uv: f64 = (s.d1v.dot(s.d1u) + ort.dot(s.d2uv)) * norm;
        let de2_dv: f64 = (s.d1v.square_magnitude() + ort.dot(s.d2v)) * norm;
        match self.fix {
            1 => {
                d[(1, 1)] = de1_du;
                d[(1, 2)] = de_uv;
                d[(2, 1)] = d[(1, 2)];
                d[(2, 2)] = de2_dv;
            }
            2 => {
                d[(1, 1)] = neg_dc.dot(s.d1u) * norm;
                d[(1, 2)] = de_uv;
                d[(2, 1)] = neg_dc.dot(s.d1v) * norm;
                d[(2, 2)] = de2_dv;
            }
            _ => {
                d[(1, 1)] = neg_dc.dot(s.d1u) * norm;
                d[(1, 2)] = de1_du;
                d[(2, 1)] = neg_dc.dot(s.d1v) * norm;
                d[(2, 2)] = de_uv;
            }
        }
    }
}

impl FunctionSet for PrjFunc<'_> {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, residual: &mut Vector<'_>) -> bool {
        let result = PrjFunc::value(self, point, residual);
        self.failure.keep(result).unwrap_or(false)
    }
}

impl FunctionSetWithDerivatives for PrjFunc<'_> {
    fn derivatives(&mut self, point: &Vector<'_>, jacobian: &mut Matrix<'_>) -> bool {
        let result = PrjFunc::derivatives(self, point, jacobian);
        self.failure.keep(result).unwrap_or(false)
    }
    fn values(
        &mut self,
        point: &Vector<'_>,
        residual: &mut Vector<'_>,
        jacobian: &mut Matrix<'_>,
    ) -> bool {
        let result = PrjFunc::values(self, point, residual, jacobian);
        self.failure.keep(result).unwrap_or(false)
    }
}
