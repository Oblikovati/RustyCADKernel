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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntImp/IntImp_IntCS.gxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntImp_IntCS`: a curve-surface intersection point near a start point, found by
//! the bounded Newton solver on `IntImp_ZerCSParFunc`.
use super::{TheCurveTool, TheFunction, ThePSurfaceTool};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math::{FunctionSetRoot, Vector};

/// The exact curve-surface intersection solved on the distance function `F`.
///
/// `IntCurveSurface_TheExactHInter` is `IntImpIntCS<IntImpZerCSParFunc<HSurfaceTool, _>>`
/// and `IntPatch_CurvIntSurf` an instance on `IntPatch_CSFunction`;
/// `IntImpIntCS::new([u, v, w], &function, 1e-7, 0.0)?`.
pub struct IntImpIntCS<F: TheFunction> {
    done: bool,
    empty: bool,
    my_function: F,
    w: f64,
    u: f64,
    v: f64,
    tol: f64,
}

/// The widened parameter range `[first, last]` by `coef` of its length, unless a
/// bound is infinite.
fn with_margin(first: f64, last: f64, coef: f64) -> (f64, f64) {
    if precision::is_infinite(first) || precision::is_infinite(last) {
        return (first, last);
    }
    let mut marg: f64 = (last - first) * coef;
    if first > last {
        marg = -marg;
    }
    (first - marg, last + marg)
}

/// The patch tool of a function.
type PTool<F> = <F as TheFunction>::SurfaceTool;
/// The curve tool of a function.
type CTool<F> = <F as TheFunction>::CurveTool;

impl<F: TheFunction> IntImpIntCS<F> {
    /// Prepares the search on a copy of `f`; the squared tangency tolerance is at
    /// least `Precision::SquareConfusion()`; `IntImpIntCS::from_function(&f, 1e-7)`.
    pub fn from_function(f: &F, tol_tangency: f64) -> Self {
        let mut tol: f64 = tol_tangency * tol_tangency;
        if tol < precision::square_confusion() {
            tol = precision::square_confusion();
        }
        Self {
            done: true,
            empty: true,
            my_function: f.clone(),
            w: 0.0,
            u: 0.0,
            v: 0.0,
            tol,
        }
    }
    /// Computes the point near `(u, v, w)` within the curve range and the patch
    /// range widened by `margin_coef` (when positive), with a default solver.
    pub fn new(
        [u, v, w]: [f64; 3],
        f: &F,
        tol_tangency: f64,
        margin_coef: f64,
    ) -> Result<Self, Failure> {
        let mut intersection = Self::from_function(f, tol_tangency);
        let mut rsnld = FunctionSetRoot::without_tolerance(&intersection.my_function, 100)?;
        let (s, c) = (f.auxillar_surface(), f.auxillar_curve());
        let (w0, w1) = (
            CTool::<F>::first_parameter(c)?,
            CTool::<F>::last_parameter(c)?,
        );
        let (u0, v0) = (
            PTool::<F>::first_u_parameter(s)?,
            PTool::<F>::first_v_parameter(s)?,
        );
        let (u1, v1) = (
            PTool::<F>::last_u_parameter(s)?,
            PTool::<F>::last_v_parameter(s)?,
        );
        let ((u0, u1), (v0, v1)) = if margin_coef > 0.0 {
            (
                with_margin(u0, u1, margin_coef),
                with_margin(v0, v1, margin_coef),
            )
        } else {
            ((u0, u1), (v0, v1))
        };
        intersection.perform([u, v, w], &mut rsnld, [u0, u1, v0, v1, w0, w1])?;
        Ok(intersection)
    }
    /// `Perform(U, V, W, Rsnld, u0, u1, v0, v1, w0, w1)`: from `(u, v, w)`, then from
    /// the curve ends `w0` and `w1`, until a root within the tolerance. Quirk kept:
    /// the native declaration names the bounds `u0, v0, u1, v1`, but the definition
    /// (and every caller) passes `u0, u1, v0, v1`.
    pub fn perform(
        &mut self,
        [u, v, w]: [f64; 3],
        rsnld: &mut FunctionSetRoot,
        [u0, u1, v0, v1, w0, w1]: [f64; 6],
    ) -> Result<(), Failure> {
        self.done = true;
        let mut uvap: Vector<'static> = Vector::filled(1, 3, 0.0)?;
        (uvap[1], uvap[2], uvap[3]) = (u, v, w);
        let mut born_inf: Vector<'static> = Vector::filled(1, 3, 0.0)?;
        let mut born_sup: Vector<'static> = Vector::filled(1, 3, 0.0)?;
        (born_inf[1], born_inf[2], born_sup[1], born_sup[2]) = (u0, v0, u1, v1);
        (born_inf[3], born_sup[3]) = (w0, w1);
        let conf: f64 = precision::confusion();
        let (s, c) = (
            self.my_function.auxillar_surface(),
            self.my_function.auxillar_curve(),
        );
        let mut tolerance: Vector<'static> = Vector::filled(1, 3, 0.0)?;
        tolerance[1] = PTool::<F>::u_resolution(s, conf)?;
        tolerance[2] = PTool::<F>::v_resolution(s, conf)?;
        tolerance[3] = CTool::<F>::resolution(c, conf)?;
        rsnld.set_tolerance(&tolerance)?;
        self.done = false;
        for attempt in 0..3 {
            match attempt {
                1 => uvap[3] = w0,
                2 => uvap[3] = w1,
                _ => {}
            }
            self.attempt(rsnld, &mut uvap, [&born_inf, &born_sup])?;
            if self.done {
                break;
            }
        }
        Ok(())
    }
    /// One solve from `uvap`; keeps the root when it is within the tolerance.
    fn attempt(
        &mut self,
        rsnld: &mut FunctionSetRoot,
        uvap: &mut Vector<'_>,
        [born_inf, born_sup]: [&Vector<'_>; 2],
    ) -> Result<(), Failure> {
        let function = &mut self.my_function;
        let solved = rsnld.perform_bounded(function, uvap, born_inf, born_sup, false);
        function.take_failure()?;
        solved?;
        if rsnld.is_done() && function.root().abs() <= self.tol {
            rsnld.copy_root(uvap)?;
            (self.u, self.v, self.w) = (uvap[1], uvap[2], uvap[3]);
            (self.empty, self.done) = (false, true);
        }
        Ok(())
    }
    /// `IsDone()`; `intersection.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    fn check(&self, need_point: bool) -> Result<(), Failure> {
        if !self.done {
            return Err(Failure::with_message(
                "StdFail_NotDone",
                "IntImp_IntCS: no root found",
            ));
        }
        if need_point && self.empty {
            return Err(errors::domain("IntImp_IntCS: the intersection is empty"));
        }
        Ok(())
    }
    /// `IsEmpty()`; `StdFail_NotDone` when not done; `intersection.is_empty()?`.
    pub fn is_empty(&self) -> Result<bool, Failure> {
        self.check(false)?;
        Ok(self.empty)
    }
    /// `Point()`: the middle of the patch and curve points; `intersection.point()?`.
    pub fn point(&self) -> Result<Point3, Failure> {
        self.check(true)?;
        Ok(self.my_function.point())
    }
    /// `ParameterOnSurface(U, V)`; `intersection.parameter_on_surface()?`.
    pub fn parameter_on_surface(&self) -> Result<(f64, f64), Failure> {
        self.check(true)?;
        Ok((self.u, self.v))
    }
    /// `ParameterOnCurve()`; `intersection.parameter_on_curve()?`.
    pub fn parameter_on_curve(&self) -> Result<f64, Failure> {
        self.check(true)?;
        Ok(self.w)
    }
    /// `Function()`: the distance function; `intersection.function()`.
    pub fn function(&mut self) -> &mut F {
        &mut self.my_function
    }
}
