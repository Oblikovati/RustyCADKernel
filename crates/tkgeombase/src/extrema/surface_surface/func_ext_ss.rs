// Created on: 1996-01-09
// Created by: Laurent PAINNOT
// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of Extrema_FuncExtSS.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_FuncExtSS`: `F(u1, v1, u2, v2)`, the projections of `S1(u1,v1) -
//! S2(u2,v2)` on the four first derivatives, whose roots are the extrema of the
//! distance between two surfaces. Every saved state is kept (no deduplication).
use crate::extrema::failures::{sequence_value, type_mismatch};
use crate::extrema::pon::POnSurf;
use crate::failure_capture::FailureCapture;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector};

/// `Extrema_FuncExtSS`; the members raise Standard_TypeMismatch before the surfaces
/// are set, and the math callbacks keep the first failure
/// ([`FuncExtSS::take_failure`]); `FuncExtSS::with_surfaces(&s1, &s2)`.
#[derive(Default)]
pub struct FuncExtSS<'a> {
    s1: Option<&'a dyn Adaptor3dSurface>,
    s2: Option<&'a dyn Adaptor3dSurface>,
    p1: Point3,
    p2: Point3,
    u1: f64,
    v1: f64,
    u2: f64,
    v2: f64,
    sq_dist: Vec<f64>,
    point1: Vec<POnSurf>,
    point2: Vec<POnSurf>,
    failure: FailureCapture,
}

impl<'a> FuncExtSS<'a> {
    /// `Extrema_FuncExtSS()`: no surface; `FuncExtSS::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_FuncExtSS(S1, S2)`; `FuncExtSS::with_surfaces(&s1, &s2)`.
    pub fn with_surfaces(s1: &'a dyn Adaptor3dSurface, s2: &'a dyn Adaptor3dSurface) -> Self {
        Self {
            s1: Some(s1),
            s2: Some(s2),
            ..Self::default()
        }
    }
    /// `Initialize(S1, S2)`: sets both and clears the extrema;
    /// `function.initialize(&s1, &s2)`.
    pub fn initialize(&mut self, s1: &'a dyn Adaptor3dSurface, s2: &'a dyn Adaptor3dSurface) {
        (self.s1, self.s2) = (Some(s1), Some(s2));
        self.point1.clear();
        self.point2.clear();
        self.sq_dist.clear();
    }
    /// `NbVariables()`: 4; `function.nb_variables()`.
    pub fn nb_variables(&self) -> i32 {
        4
    }
    /// `NbEquations()`: 4; `function.nb_equations()`.
    pub fn nb_equations(&self) -> i32 {
        4
    }
    fn surfaces(&self, member: &str) -> Result<(&'a dyn Adaptor3dSurface, &'a dyn Adaptor3dSurface), Failure> {
        match (self.s1, self.s2) {
            (Some(s1), Some(s2)) => Ok((s1, s2)),
            _ => Err(type_mismatch(&format!("Extrema_FuncExtSS::{member}"))),
        }
    }
    fn set_parameters(&mut self, uv: &Vector<'_>) {
        (self.u1, self.v1, self.u2, self.v2) = (uv[1], uv[2], uv[3], uv[4]);
    }
    /// `Value(UV, F)`; `function.value(&uv, &mut f)?`.
    pub fn value(&mut self, uv: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        let (s1, s2) = self.surfaces("Value()")?;
        self.set_parameters(uv);
        let d1 = s1.d1(self.u1, self.v1)?;
        let d2 = s2.d1(self.u2, self.v2)?;
        (self.p1, self.p2) = (d1.point, d2.point);
        let p1p2: Vector3 = Vector3::between(self.p2, self.p1);
        f[1] = p1p2.dot(d1.d1u);
        f[2] = p1p2.dot(d1.d1v);
        f[3] = p1p2.dot(d2.d1u);
        f[4] = p1p2.dot(d2.d1v);
        Ok(true)
    }
    /// `Derivatives(UV, DF)`: `Values` with a scratch F;
    /// `function.derivatives(&uv, &mut df)?`.
    pub fn derivatives(&mut self, uv: &Vector<'_>, df: &mut Matrix<'_>) -> Result<bool, Failure> {
        let mut f: Vector<'static> = Vector::with_bounds(1, 4)?;
        self.values(uv, &mut f, df)
    }
    /// `Values(UV, F, DF)`; `function.values(&uv, &mut f, &mut df)?`.
    pub fn values(&mut self, uv: &Vector<'_>, f: &mut Vector<'_>, df: &mut Matrix<'_>) -> Result<bool, Failure> {
        let (s1, s2) = self.surfaces("Values()")?;
        self.set_parameters(uv);
        let a = s1.d2(self.u1, self.v1)?;
        let b = s2.d2(self.u2, self.v2)?;
        (self.p1, self.p2) = (a.point, b.point);
        let p1p2: Vector3 = Vector3::between(self.p2, self.p1);
        f[1] = p1p2.dot(a.d1u);
        f[2] = p1p2.dot(a.d1v);
        f[3] = p1p2.dot(b.d1u);
        f[4] = p1p2.dot(b.d1v);
        df[(1, 1)] = a.d1u.square_magnitude() + p1p2.dot(a.d2u);
        df[(1, 2)] = a.d1v.dot(a.d1u) + p1p2.dot(a.d2uv);
        df[(1, 3)] = -b.d1u.dot(a.d1u);
        df[(1, 4)] = -b.d1v.dot(a.d1u);
        df[(2, 1)] = df[(1, 2)];
        df[(2, 2)] = a.d1v.square_magnitude() + p1p2.dot(a.d2v);
        df[(2, 3)] = -b.d1u.dot(a.d1v);
        df[(2, 4)] = -b.d1v.dot(a.d1v);
        df[(3, 1)] = -df[(1, 3)];
        df[(3, 2)] = -df[(2, 3)];
        df[(3, 3)] = -b.d1u.square_magnitude() + p1p2.dot(b.d2u);
        df[(3, 4)] = -b.d1v.dot(b.d1u) + p1p2.dot(b.d2uv);
        df[(4, 1)] = -df[(1, 4)];
        df[(4, 2)] = -df[(2, 4)];
        df[(4, 3)] = df[(3, 4)];
        df[(4, 4)] = -b.d1v.square_magnitude() + p1p2.dot(b.d2v);
        Ok(true)
    }
    /// `GetStateNumber()`: saves the last point (always); returns 0;
    /// `function.get_state_number()?`.
    pub fn get_state_number(&mut self) -> Result<i32, Failure> {
        self.surfaces("GetStateNumber()")?;
        self.sq_dist.push(self.p1.square_distance(&self.p2));
        self.point1.push(POnSurf::new(self.u1, self.v1, self.p1));
        self.point2.push(POnSurf::new(self.u2, self.v2, self.p2));
        Ok(0)
    }
    /// `NbExt()`; `function.nb_ext()`.
    pub fn nb_ext(&self) -> i32 {
        self.sq_dist.len() as i32
    }
    /// `SquareDistance(N)`; `function.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        self.surfaces("SquareDistance()")?;
        sequence_value("Extrema_FuncExtSS::SquareDistance()", &self.sq_dist, n)
    }
    /// `PointOnS1(N)`; `function.point_on_s1(1)?`.
    pub fn point_on_s1(&self, n: i32) -> Result<POnSurf, Failure> {
        self.surfaces("PointOnS1()")?;
        sequence_value("Extrema_FuncExtSS::PointOnS1()", &self.point1, n)
    }
    /// `PointOnS2(N)`; `function.point_on_s2(1)?`.
    pub fn point_on_s2(&self, n: i32) -> Result<POnSurf, Failure> {
        self.surfaces("PointOnS2()")?;
        sequence_value("Extrema_FuncExtSS::PointOnS2()", &self.point2, n)
    }
    /// The first failure a math callback kept since the last call;
    /// `function.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.failure.take()
    }
}

impl FunctionSet for FuncExtSS<'_> {
    fn variables(&self) -> i32 {
        4
    }
    fn equations(&self) -> i32 {
        4
    }
    fn value(&mut self, point: &Vector<'_>, residual: &mut Vector<'_>) -> bool {
        let result = FuncExtSS::value(self, point, residual);
        self.failure.keep(result).unwrap_or(false)
    }
    fn state_number(&mut self) -> i32 {
        let result = self.get_state_number();
        self.failure.keep(result).unwrap_or(0)
    }
}

impl FunctionSetWithDerivatives for FuncExtSS<'_> {
    fn derivatives(&mut self, point: &Vector<'_>, jacobian: &mut Matrix<'_>) -> bool {
        let result = FuncExtSS::derivatives(self, point, jacobian);
        self.failure.keep(result).unwrap_or(false)
    }
    fn values(&mut self, point: &Vector<'_>, residual: &mut Vector<'_>, jacobian: &mut Matrix<'_>) -> bool {
        let result = FuncExtSS::values(self, point, residual, jacobian);
        self.failure.keep(result).unwrap_or(false)
    }
}
