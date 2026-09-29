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
// Rust translation of Extrema_GenLocateExtCC.hxx and Extrema_LocateExtCC.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_GenLocateExtCC` (`Extrema_LocECC`): the curve-curve extremum nearest to
//! a start pair, by math_FunctionSetRoot on [`GFuncExtCC`]; and `Extrema_LocateExtCC`,
//! which calls it on 3D curves with the resolutions of Precision::Confusion().
use super::func_ext_cc::GFuncExtCC;
use crate::extrema::curve_tool::ExtremaCurveTool;
use crate::extrema::curve_tools::CurveTool;
use crate::extrema::failures::not_done;
use crate::extrema::pon::{POnCurv, POnCurve};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::math::{FunctionSetRoot, Vector};

/// `Extrema_GenLocateExtCC` over one [`ExtremaCurveTool`];
/// `GenLocateExtCC::<CurveTool>::new(&c1, &c2, 0.5, 0.5, 1e-9, 1e-9)?.is_done()`.
#[derive(Clone, Copy, Debug)]
pub struct GenLocateExtCC<P> {
    done: bool,
    sq_dist: f64,
    point1: POnCurve<P>,
    point2: POnCurve<P>,
}

/// `Extrema_LocECC`: the 3D instantiation.
pub type LocECC = GenLocateExtCC<rustycad_tkmath::gp::Point3>;

// The ordered parameter range of a curve.
fn ordered_range<T: ExtremaCurveTool>(curve: &T::Curve) -> Result<(f64, f64), Failure> {
    let (first, last) = (T::first_parameter(curve)?, T::last_parameter(curve)?);
    Ok(if first > last {
        (last, first)
    } else {
        (first, last)
    })
}

// A 1-based math_Vector of two values.
fn pair(first: f64, second: f64) -> Result<Vector<'static>, Failure> {
    let mut vector: Vector<'static> = Vector::with_bounds(1, 2)?;
    (vector[1], vector[2]) = (first, second);
    Ok(vector)
}

impl<P: Copy + Default> GenLocateExtCC<P> {
    /// `Extrema_GenLocateExtCC(C1, C2, U0, V0, TolU, TolV)`: math_FunctionSetRoot from
    /// `(U0, V0)` within the curve ranges; done when a root was saved and F at it is
    /// below 1e-10 in both components (signed, not in absolute value);
    /// `GenLocateExtCC::new::<CurveTool>(&c1, &c2, 0.5, 0.5, 1e-9, 1e-9)?`.
    pub fn new<T: ExtremaCurveTool<Point = P>>(
        c1: &T::Curve,
        c2: &T::Curve,
        (u0, v0): (f64, f64),
        (tol_u, tol_v): (f64, f64),
    ) -> Result<Self, Failure> {
        let mut extrema = Self {
            done: false,
            sq_dist: f64::MAX,
            point1: POnCurve::default(),
            point2: POnCurve::default(),
        };
        let (uinf, usup) = ordered_range::<T>(c1)?;
        let (vinf, vsup) = ordered_range::<T>(c2)?;
        let mut function = GFuncExtCC::<T>::with_curves(c1, c2, 1.0e-10)?;
        let mut solver = FunctionSetRoot::new(&function, &pair(tol_u, tol_v)?, 100)?;
        let start: Vector<'static> = pair(u0, v0)?;
        let (lower, upper) = (pair(uinf, vinf)?, pair(usup, vsup)?);
        solver.perform_bounded(&mut function, &start, &lower, &upper, false)?;
        function.take_failure()?;
        if solver.is_done() && function.nb_ext() > 0 {
            extrema.check_root(&mut function)?;
        }
        Ok(extrema)
    }

    // Keeps the first saved root, done when F at it is below 1e-10.
    fn check_root<T: ExtremaCurveTool<Point = P>>(
        &mut self,
        function: &mut GFuncExtCC<'_, T>,
    ) -> Result<(), Failure> {
        self.sq_dist = function.square_distance(1)?;
        (self.point1, self.point2) = function.points(1)?;
        let start: Vector<'static> = pair(self.point1.parameter(), self.point2.parameter())?;
        let mut ff: Vector<'static> = Vector::with_bounds(1, 2)?;
        function.value(&start, &mut ff)?;
        let tolf: f64 = 1.0e-10;
        if (ff[1] < tolf) && (ff[2] < tolf) {
            self.done = true;
        }
        Ok(())
    }
    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `SquareDistance()`: StdFail_NotDone when not done; `extrema.square_distance()?`.
    pub fn square_distance(&self) -> Result<f64, Failure> {
        if !self.done {
            return Err(not_done("Extrema_GenLocateExtCC::SquareDistance()"));
        }
        Ok(self.sq_dist)
    }
    /// `Point(P1, P2)`, returned as `(P1, P2)`; `extrema.point()?`.
    pub fn point(&self) -> Result<(POnCurve<P>, POnCurve<P>), Failure> {
        if !self.done {
            return Err(not_done("Extrema_GenLocateExtCC::Point()"));
        }
        Ok((self.point1, self.point2))
    }
}

/// `Extrema_LocateExtCC`: the extremum of two 3D curves nearest to `(U0, V0)`, by
/// Extrema_LocECC with the curve resolutions of Precision::Confusion();
/// `LocateExtCC::new(&c1, &c2, 0.5, 0.5)?.is_done()`.
#[derive(Clone, Copy, Debug)]
pub struct LocateExtCC {
    done: bool,
    sq_dist: f64,
    point1: POnCurv,
    point2: POnCurv,
}

impl LocateExtCC {
    /// `Extrema_LocateExtCC(C1, C2, U0, V0)`; `LocateExtCC::new(&c1, &c2, 0.5, 0.5)?`.
    pub fn new(
        c1: &dyn Adaptor3dCurve,
        c2: &dyn Adaptor3dCurve,
        u0: f64,
        v0: f64,
    ) -> Result<Self, Failure> {
        let tol_u: f64 = c1.resolution(precision::confusion())?;
        let tol_v: f64 = c2.resolution(precision::confusion())?;
        // Not implemented for now: the call to Extrema_ELCC.
        let xtrem = LocECC::new::<CurveTool>(c1, c2, (u0, v0), (tol_u, tol_v))?;
        let mut extrema = Self {
            done: xtrem.is_done(),
            sq_dist: f64::MAX,
            point1: POnCurv::default(),
            point2: POnCurv::default(),
        };
        if xtrem.is_done() {
            extrema.sq_dist = xtrem.square_distance()?;
            (extrema.point1, extrema.point2) = xtrem.point()?;
        }
        Ok(extrema)
    }
    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `SquareDistance()`: StdFail_NotDone when not done; `extrema.square_distance()?`.
    pub fn square_distance(&self) -> Result<f64, Failure> {
        if !self.done {
            return Err(not_done("Extrema_LocateExtCC::SquareDistance()"));
        }
        Ok(self.sq_dist)
    }
    /// `Point(P1, P2)`, returned as `(P1 on C1, P2 on C2)`; `extrema.point()?`.
    pub fn point(&self) -> Result<(POnCurv, POnCurv), Failure> {
        if !self.done {
            return Err(not_done("Extrema_LocateExtCC::Point()"));
        }
        Ok((self.point1, self.point2))
    }
}
