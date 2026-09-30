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
// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_FunctionH.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Bisector_FunctionH`: the function H of the point of a curve whose tangent makes
//! the same angle with the chord to a fixed point as a fixed tangent.
use super::eval_failure::EvalFailure;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::gp::{Point2, Vector2};
use rustycad_tkmath::math::{Function, FunctionWithDerivative};

/// `Bisector_FunctionH`: with `P2`, `T2` the point and tangent of `C2` at `x` and
/// `t1` the unit fixed tangent, `H(x) = (p1 - P2) . (|T2| t1 - T2)`; a
/// `math_FunctionWithDerivative`. A curve evaluation failure makes the member answer
/// `false` and is raised by [`BisectorFunctionH::raised`].
/// ```
/// use rustycad_tktopalgo::bisector::BisectorFunctionH;
/// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Direction2, Point2, Vector2};
/// use rustycad_tkmath::math::Function;
/// let line: Handle<dyn Geom2dCurve> =
///     Handle::new(Geom2dLine::from_point(Point2::new(0.0, 1.0), Direction2::new(1.0, 0.0)?)).cast();
/// let mut h = BisectorFunctionH::new(&line, Point2::new(0.0, 0.0), Vector2::new(1.0, 0.0))?;
/// let mut value = 1.0;
/// assert!(h.value(3.0, &mut value) && value == 0.0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BisectorFunctionH {
    curve2: Handle<dyn Geom2dCurve>,
    p1: Point2,
    t1: Vector2,
    failure: EvalFailure,
}

impl BisectorFunctionH {
    /// `Bisector_FunctionH(C2, P1, T1)`: `T1` is normalized, so a null vector raises
    /// gp_VectorWithNullMagnitude; `BisectorFunctionH::new(&curve, p1, t1)?`.
    pub fn new(c2: &Handle<dyn Geom2dCurve>, p1: Point2, t1: Vector2) -> Result<Self, Failure> {
        Ok(Self {
            curve2: c2.clone(),
            p1,
            t1: t1.normalized()?,
            failure: EvalFailure::default(),
        })
    }
    /// The exception a curve evaluation raised, after a member answered `false`;
    /// `h.raised()?`.
    pub fn raised(&self) -> Result<(), Failure> {
        self.failure.check()
    }
    // `(Ax, Ay)`: `|T2| t1 - T2`.
    fn a_terms(&self, t2: Vector2) -> (f64, f64) {
        let norm_t2: f64 = t2.magnitude();
        (
            norm_t2 * self.t1.x() - t2.x(),
            norm_t2 * self.t1.y() - t2.y(),
        )
    }
    // `F` from the curve point and the A terms.
    fn f_value(&self, p2: Point2, ax: f64, ay: f64) -> f64 {
        (self.p1.x() - p2.x()) * ax + (self.p1.y() - p2.y()) * ay
    }
    // `(F, D)` from the point and the two derivatives of the curve.
    fn f_and_d(&self, p2: Point2, t2: Vector2, t2v: Vector2) -> (f64, f64) {
        let norm_t2: f64 = t2.magnitude();
        let (ax, ay) = self.a_terms(t2);
        let f: f64 = self.f_value(p2, ax, ay);
        let scal: f64 = t2.dot(t2v) / norm_t2;
        let dax: f64 = scal * self.t1.x() - t2v.x();
        let day: f64 = scal * self.t1.y() - t2v.y();
        let d: f64 = -t2.x() * ax - t2.y() * ay
            + (self.p1.x() - p2.x()) * dax
            + (self.p1.y() - p2.y()) * day;
        (f, d)
    }
}

impl Function for BisectorFunctionH {
    fn value(&mut self, x: f64, f: &mut f64) -> bool {
        let Some((p2, t2)) = self.failure.keep(self.curve2.d1(x)) else {
            return false;
        };
        let (ax, ay) = self.a_terms(t2);
        *f = self.f_value(p2, ax, ay);
        true
    }
}

impl FunctionWithDerivative for BisectorFunctionH {
    fn derivative(&mut self, x: f64, d: &mut f64) -> bool {
        let mut f: f64 = 0.0;
        self.values(x, &mut f, d)
    }
    fn values(&mut self, x: f64, f: &mut f64, d: &mut f64) -> bool {
        let Some((p2, t2, t2v)) = self.failure.keep(self.curve2.d2(x)) else {
            return false;
        };
        (*f, *d) = self.f_and_d(p2, t2, t2v);
        true
    }
}
