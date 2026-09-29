// Created on: 2016-05-10
// Created by: Alexander MALYSHEV
// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2016 OPEN CASCADE SAS
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
// Rust translation of Extrema_FuncPSDist.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_FuncPSDist`: the square distance `|S(u,v) - P|²` and the half gradient
//! `((P - S).Su, (P - S).Sv)`, for math_BFGS; false outside the surface bounds.
use crate::failure_capture::FailureCapture;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkmath::math::{MultipleVarFunction, MultipleVarFunctionWithGradient, Vector};

/// `Extrema_FuncPSDist`; `FuncPSDist::new(&surface, point)`.
pub struct FuncPSDist<'a> {
    surf: &'a dyn Adaptor3dSurface,
    p: Point3,
    failure: FailureCapture,
}

impl<'a> FuncPSDist<'a> {
    /// `Extrema_FuncPSDist(theS, theP)`; `FuncPSDist::new(&surface, point)`.
    pub fn new(surface: &'a dyn Adaptor3dSurface, point: Point3) -> Self {
        Self {
            surf: surface,
            p: point,
            failure: FailureCapture::default(),
        }
    }
    /// `NbVariables()`: 2; `function.nb_variables()`.
    pub fn nb_variables(&self) -> i32 {
        2
    }
    // `IsInside(X)`: X within the surface bounds.
    fn is_inside(&self, x: &Vector<'_>) -> Result<bool, Failure> {
        Ok(x[1] >= self.surf.first_u_parameter()?
            && x[1] <= self.surf.last_u_parameter()?
            && x[2] >= self.surf.first_v_parameter()?
            && x[2] <= self.surf.last_v_parameter()?)
    }
    /// `Value(X, F)`: the square distance; `function.value(&x, &mut f)?`.
    pub fn value(&self, x: &Vector<'_>, f: &mut f64) -> Result<bool, Failure> {
        if !self.is_inside(x)? {
            return Ok(false);
        }
        *f = self.surf.value(x[1], x[2])?.square_distance(&self.p);
        Ok(true)
    }
    /// `Gradient(X, G)`: `((P - S).Su, (P - S).Sv)`; `function.gradient(&x, &mut g)?`.
    pub fn gradient(&self, x: &Vector<'_>, g: &mut Vector<'_>) -> Result<bool, Failure> {
        if !self.is_inside(x)? {
            return Ok(false);
        }
        self.half_gradient(x, g)?;
        Ok(true)
    }
    // The gradient members of Gradient and Values.
    fn half_gradient(&self, x: &Vector<'_>, g: &mut Vector<'_>) -> Result<(), Failure> {
        let d1 = self.surf.d1(x[1], x[2])?;
        let p1p2: Vector3 = Vector3::between(d1.point, self.p);
        g[1] = p1p2.dot(d1.d1u);
        g[2] = p1p2.dot(d1.d1v);
        Ok(())
    }
    /// `Values(X, F, G)`: the gradient, then the value by a second evaluation;
    /// `function.values(&x, &mut f, &mut g)?`.
    pub fn values(&self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> Result<bool, Failure> {
        if !self.is_inside(x)? {
            return Ok(false);
        }
        self.half_gradient(x, g)?;
        *f = self.surf.value(x[1], x[2])?.square_distance(&self.p);
        Ok(true)
    }
    /// The first failure a math callback kept since the last call;
    /// `function.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.failure.take()
    }
}

impl MultipleVarFunction for FuncPSDist<'_> {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let result = FuncPSDist::value(self, point, value);
        self.failure.keep(result).unwrap_or(false)
    }
}

impl MultipleVarFunctionWithGradient for FuncPSDist<'_> {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        let result = FuncPSDist::gradient(self, point, gradient);
        self.failure.keep(result).unwrap_or(false)
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        let result = FuncPSDist::values(self, point, value, gradient);
        self.failure.keep(result).unwrap_or(false)
    }
}
