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
// Rust translation of Extrema_FuncExtCS.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_FuncExtCS`: `F(t, u, v) = ((C(t) - S(u,v)).C'(t), (C(t) - S(u,v)).Su,
//! (C(t) - S(u,v)).Sv)`, whose roots are the extrema of the distance between a curve
//! and a surface.
use crate::extrema::failures::{sequence_value, type_mismatch};
use crate::extrema::pon::{POnCurv, POnSurf};
use crate::failure_capture::FailureCapture;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector};

/// `Extrema_FuncExtCS`; the members raise Standard_TypeMismatch before the curve and
/// the surface are set, and the math callbacks keep the first failure
/// ([`FuncExtCS::take_failure`]); `FuncExtCS::with_curve_surface(&curve, &surface)`.
#[derive(Default)]
pub struct FuncExtCS<'a> {
    c: Option<&'a dyn Adaptor3dCurve>,
    s: Option<&'a dyn Adaptor3dSurface>,
    p1: Point3,
    p2: Point3,
    t: f64,
    u: f64,
    v: f64,
    sq_dist: Vec<f64>,
    point1: Vec<POnCurv>,
    point2: Vec<POnSurf>,
    failure: FailureCapture,
}

impl<'a> FuncExtCS<'a> {
    /// `Extrema_FuncExtCS()`: neither curve nor surface; `FuncExtCS::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_FuncExtCS(C, S)`; `FuncExtCS::with_curve_surface(&curve, &surface)`.
    pub fn with_curve_surface(
        curve: &'a dyn Adaptor3dCurve,
        surface: &'a dyn Adaptor3dSurface,
    ) -> Self {
        let mut function: Self = Self::new();
        function.initialize(curve, surface);
        function
    }
    /// `Initialize(C, S)`: sets both and clears the extrema;
    /// `function.initialize(&curve, &surface)`.
    pub fn initialize(&mut self, curve: &'a dyn Adaptor3dCurve, surface: &'a dyn Adaptor3dSurface) {
        self.c = Some(curve);
        self.s = Some(surface);
        self.point1.clear();
        self.point2.clear();
        self.sq_dist.clear();
    }
    /// `NbVariables()`: 3; `function.nb_variables()`.
    pub fn nb_variables(&self) -> i32 {
        3
    }
    /// `NbEquations()`: 3; `function.nb_equations()`.
    pub fn nb_equations(&self) -> i32 {
        3
    }
    // The curve and the surface, Standard_TypeMismatch before Initialize.
    fn geometry(
        &self,
        member: &str,
    ) -> Result<(&'a dyn Adaptor3dCurve, &'a dyn Adaptor3dSurface), Failure> {
        match (self.c, self.s) {
            (Some(curve), Some(surface)) => Ok((curve, surface)),
            _ => Err(type_mismatch(&format!("Extrema_FuncExtCS::{member}"))),
        }
    }
    /// `Value(UV, F)`; `function.value(&tuv, &mut f)?`.
    pub fn value(&mut self, uv: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        let (curve, surface) = self.geometry("Value()")?;
        (self.t, self.u, self.v) = (uv[1], uv[2], uv[3]);
        let (p1, dtc) = curve.d1(self.t)?;
        let d1 = surface.d1(self.u, self.v)?;
        (self.p1, self.p2) = (p1, d1.point);
        let p1p2: Vector3 = Vector3::between(self.p2, self.p1);
        f[1] = p1p2.dot(dtc);
        f[2] = p1p2.dot(d1.d1u);
        f[3] = p1p2.dot(d1.d1v);
        Ok(true)
    }
    /// `Derivatives(UV, DF)`: `Values` with a scratch F;
    /// `function.derivatives(&tuv, &mut df)?`.
    pub fn derivatives(&mut self, uv: &Vector<'_>, df: &mut Matrix<'_>) -> Result<bool, Failure> {
        let mut f: Vector<'static> = Vector::with_bounds(1, 3)?;
        self.values(uv, &mut f, df)
    }
    /// `Values(UV, F, DF)`; `function.values(&tuv, &mut f, &mut df)?`.
    pub fn values(
        &mut self,
        uv: &Vector<'_>,
        f: &mut Vector<'_>,
        df: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        let (curve, surface) = self.geometry("Values()")?;
        (self.t, self.u, self.v) = (uv[1], uv[2], uv[3]);
        let (p1, dtc, dttc) = curve.d2(self.t)?;
        let d2 = surface.d2(self.u, self.v)?;
        (self.p1, self.p2) = (p1, d2.point);
        let (dus, dvs) = (d2.d1u, d2.d1v);
        let p1p2: Vector3 = Vector3::between(self.p2, self.p1);
        f[1] = p1p2.dot(dtc);
        f[2] = p1p2.dot(dus);
        f[3] = p1p2.dot(dvs);
        df[(1, 1)] = dtc.square_magnitude() + p1p2.dot(dttc);
        df[(1, 2)] = -dus.dot(dtc);
        df[(1, 3)] = -dvs.dot(dtc);
        df[(2, 1)] = -df[(1, 2)];
        df[(2, 2)] = -dus.square_magnitude() + p1p2.dot(d2.d2u);
        df[(2, 3)] = -dvs.dot(dus) + p1p2.dot(d2.d2uv);
        df[(3, 1)] = -df[(1, 3)];
        df[(3, 2)] = df[(2, 3)];
        df[(3, 3)] = -dvs.square_magnitude() + p1p2.dot(d2.d2v);
        Ok(true)
    }
    /// `GetStateNumber()`: saves the last point unless a saved one has its curve
    /// parameter within PConfusion; returns 0; `function.get_state_number()?`.
    pub fn get_state_number(&mut self) -> Result<i32, Failure> {
        self.geometry("GetStateNumber()")?;
        let tol2d: f64 = precision::square_p_confusion();
        let known = self.point1.iter().any(|point| {
            let mut at: f64 = point.parameter();
            at -= self.t;
            at *= at;
            at <= tol2d
        });
        if !known {
            self.sq_dist.push(self.p1.square_distance(&self.p2));
            self.point1.push(POnCurv::new(self.t, self.p1));
            self.point2.push(POnSurf::new(self.u, self.v, self.p2));
        }
        Ok(0)
    }
    /// `NbExt()`; `function.nb_ext()`.
    pub fn nb_ext(&self) -> i32 {
        self.sq_dist.len() as i32
    }
    /// `SquareDistance(N)`; `function.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        self.geometry("SquareDistance()")?;
        sequence_value("Extrema_FuncExtCS::SquareDistance()", &self.sq_dist, n)
    }
    /// `PointOnCurve(N)`; `function.point_on_curve(1)?`.
    pub fn point_on_curve(&self, n: i32) -> Result<POnCurv, Failure> {
        self.geometry("PointOnCurve()")?;
        sequence_value("Extrema_FuncExtCS::PointOnCurve()", &self.point1, n)
    }
    /// `PointOnSurface(N)`; `function.point_on_surface(1)?`.
    pub fn point_on_surface(&self, n: i32) -> Result<POnSurf, Failure> {
        self.geometry("PointOnSurface()")?;
        sequence_value("Extrema_FuncExtCS::PointOnSurface()", &self.point2, n)
    }
    /// `SquareDistances()`: the saved square distances, to edit;
    /// `function.square_distances().clear()`.
    pub fn square_distances(&mut self) -> &mut Vec<f64> {
        &mut self.sq_dist
    }
    /// `PointsOnCurve()`: the saved points on the curve, to edit;
    /// `function.points_on_curve().clear()`.
    pub fn points_on_curve(&mut self) -> &mut Vec<POnCurv> {
        &mut self.point1
    }
    /// `PointsOnSurf()`: the saved points on the surface, to edit;
    /// `function.points_on_surf().clear()`.
    pub fn points_on_surf(&mut self) -> &mut Vec<POnSurf> {
        &mut self.point2
    }
    /// The first failure a math callback kept since the last call;
    /// `function.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.failure.take()
    }
}

impl FunctionSet for FuncExtCS<'_> {
    fn variables(&self) -> i32 {
        3
    }
    fn equations(&self) -> i32 {
        3
    }
    fn value(&mut self, point: &Vector<'_>, residual: &mut Vector<'_>) -> bool {
        let result = FuncExtCS::value(self, point, residual);
        self.failure.keep(result).unwrap_or(false)
    }
    fn state_number(&mut self) -> i32 {
        let result = self.get_state_number();
        self.failure.keep(result).unwrap_or(0)
    }
}

impl FunctionSetWithDerivatives for FuncExtCS<'_> {
    fn derivatives(&mut self, point: &Vector<'_>, jacobian: &mut Matrix<'_>) -> bool {
        let result = FuncExtCS::derivatives(self, point, jacobian);
        self.failure.keep(result).unwrap_or(false)
    }
    fn values(
        &mut self,
        point: &Vector<'_>,
        residual: &mut Vector<'_>,
        jacobian: &mut Matrix<'_>,
    ) -> bool {
        let result = FuncExtCS::values(self, point, residual, jacobian);
        self.failure.keep(result).unwrap_or(false)
    }
}
