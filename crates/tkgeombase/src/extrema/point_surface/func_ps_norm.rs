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
// Rust translation of Extrema_FuncPSNorm.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_FuncPSNorm`: `F(u, v) = ((S(u,v) - P).Su, (S(u,v) - P).Sv)`, whose roots
//! are the extrema of the distance between a point and a surface.
use crate::extrema::failures::{sequence_value, type_mismatch};
use crate::extrema::pon::POnSurf;
use crate::extrema::surface_ref::SurfaceRef;
use crate::failure_capture::FailureCapture;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector};

/// `Extrema_FuncPSNorm`: the math function set of the point-surface extrema. The
/// native members raise Standard_TypeMismatch before the point and the surface are
/// set; the math callbacks keep the first failure ([`FuncPSNorm::take_failure`]);
/// `FuncPSNorm::with_point_surface(point, &surface)`.
#[derive(Default)]
pub struct FuncPSNorm<'a> {
    p: Point3,
    s: Option<SurfaceRef<'a>>,
    u: f64,
    v: f64,
    ps: Point3,
    sq_dist: Vec<f64>,
    points: Vec<POnSurf>,
    p_init: bool,
    s_init: bool,
    failure: FailureCapture,
}

impl<'a> FuncPSNorm<'a> {
    /// `Extrema_FuncPSNorm()`: neither point nor surface; `FuncPSNorm::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_FuncPSNorm(P, S)`; `FuncPSNorm::with_point_surface(point, &surface)`.
    pub fn with_point_surface(point: Point3, surface: &'a dyn Adaptor3dSurface) -> Self {
        Self::with_point_surface_ref(point, SurfaceRef::Borrowed(surface))
    }
    /// The constructor on a borrowed or a shared surface.
    pub(crate) fn with_point_surface_ref(point: Point3, surface: SurfaceRef<'a>) -> Self {
        Self {
            p: point,
            s: Some(surface),
            p_init: true,
            s_init: true,
            ..Self::default()
        }
    }
    /// `Initialize(S)`: sets the surface and clears the extrema;
    /// `function.initialize(&surface)`.
    pub fn initialize(&mut self, surface: &'a dyn Adaptor3dSurface) {
        self.initialize_ref(SurfaceRef::Borrowed(surface));
    }
    /// `Initialize(S)` on a borrowed or a shared surface.
    pub(crate) fn initialize_ref(&mut self, surface: SurfaceRef<'a>) {
        self.s = Some(surface);
        self.s_init = true;
        self.points.clear();
        self.sq_dist.clear();
    }
    /// `SetPoint(P)`: sets the point and clears the extrema; `function.set_point(point)`.
    pub fn set_point(&mut self, point: Point3) {
        self.p = point;
        self.p_init = true;
        self.points.clear();
        self.sq_dist.clear();
    }
    /// `NbVariables()`: 2; `function.nb_variables()`.
    pub fn nb_variables(&self) -> i32 {
        2
    }
    /// `NbEquations()`: 2; `function.nb_equations()`.
    pub fn nb_equations(&self) -> i32 {
        2
    }
    // The surface, Standard_TypeMismatch before the point and the surface are set.
    fn surface(&self, member: &str) -> Result<&dyn Adaptor3dSurface, Failure> {
        match &self.s {
            Some(surface) if self.p_init && self.s_init => Ok(surface.get()),
            _ => Err(type_mismatch(&format!("Extrema_FuncPSNorm::{member}"))),
        }
    }
    /// `Value(UV, F)`; `function.value(&uv, &mut f)?`.
    pub fn value(&mut self, uv: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        let d1 = self.surface("Value()")?.d1(uv[1], uv[2])?;
        self.u = uv[1];
        self.v = uv[2];
        self.ps = d1.point;
        let pps: Vector3 = Vector3::between(self.p, self.ps);
        f[1] = pps.dot(d1.d1u);
        f[2] = pps.dot(d1.d1v);
        Ok(true)
    }
    /// `Derivatives(UV, DF)`: `Values` with a scratch F;
    /// `function.derivatives(&uv, &mut df)?`.
    pub fn derivatives(&mut self, uv: &Vector<'_>, df: &mut Matrix<'_>) -> Result<bool, Failure> {
        let mut f: Vector<'static> = Vector::with_bounds(1, 2)?;
        self.values(uv, &mut f, df)
    }
    /// `Values(UV, F, DF)`; `function.values(&uv, &mut f, &mut df)?`.
    pub fn values(
        &mut self,
        uv: &Vector<'_>,
        f: &mut Vector<'_>,
        df: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        let d2 = self.surface("Values()")?.d2(uv[1], uv[2])?;
        self.u = uv[1];
        self.v = uv[2];
        self.ps = d2.point;
        let pps: Vector3 = Vector3::between(self.p, self.ps);
        let (dus, dvs) = (d2.d1u, d2.d1v);
        df[(1, 1)] = dus.square_magnitude() + pps.dot(d2.d2u);
        df[(1, 2)] = dvs.dot(dus) + pps.dot(d2.d2uv);
        df[(2, 1)] = df[(1, 2)];
        df[(2, 2)] = dvs.square_magnitude() + pps.dot(d2.d2v);
        f[1] = pps.dot(dus);
        f[2] = pps.dot(dvs);
        Ok(true)
    }
    /// `GetStateNumber()`: saves the last point unless a saved one is within
    /// PConfusion in `(u, v)`; returns 0; `function.get_state_number()?`.
    pub fn get_state_number(&mut self) -> Result<i32, Failure> {
        self.surface("GetStateNumber()")?;
        let tol2d: f64 = precision::p_confusion() * precision::p_confusion();
        let known = self.points.iter().any(|point| {
            let (au, av) = point.parameter();
            ((self.u - au) * (self.u - au) + (self.v - av) * (self.v - av)) <= tol2d
        });
        if !known {
            self.sq_dist.push(self.ps.square_distance(&self.p));
            self.points.push(POnSurf::new(self.u, self.v, self.ps));
        }
        Ok(0)
    }
    /// `NbExt()`: the number of saved points; `function.nb_ext()`.
    pub fn nb_ext(&self) -> i32 {
        self.sq_dist.len() as i32
    }
    /// `SquareDistance(N)`; `function.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        self.surface("SquareDistance()")?;
        sequence_value("Extrema_FuncPSNorm::SquareDistance()", &self.sq_dist, n)
    }
    /// `Point(N)`; `function.point(1)?`.
    pub fn point(&self, n: i32) -> Result<POnSurf, Failure> {
        self.surface("Point()")?;
        sequence_value("Extrema_FuncPSNorm::Point()", &self.points, n)
    }
    /// The first failure a math callback kept since the last call;
    /// `function.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.failure.take()
    }
}

impl FunctionSet for FuncPSNorm<'_> {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, residual: &mut Vector<'_>) -> bool {
        let result = FuncPSNorm::value(self, point, residual);
        self.failure.keep(result).unwrap_or(false)
    }
    fn state_number(&mut self) -> i32 {
        let result = self.get_state_number();
        self.failure.keep(result).unwrap_or(0)
    }
}

impl FunctionSetWithDerivatives for FuncPSNorm<'_> {
    fn derivatives(&mut self, point: &Vector<'_>, jacobian: &mut Matrix<'_>) -> bool {
        let result = FuncPSNorm::derivatives(self, point, jacobian);
        self.failure.keep(result).unwrap_or(false)
    }
    fn values(
        &mut self,
        point: &Vector<'_>,
        residual: &mut Vector<'_>,
        jacobian: &mut Matrix<'_>,
    ) -> bool {
        let result = FuncPSNorm::values(self, point, residual, jacobian);
        self.failure.keep(result).unwrap_or(false)
    }
}
