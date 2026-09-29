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
// Rust translation of Geom2dGcc_FunctionTanCuPnt.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGcc_FunctionTanCuPnt`: the function of a line through a point and
//! tangent to a curve.
use super::{CurveTool, EvalFailure};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::gp::{Point2, Vector2};
use rustycad_tkmath::math::{Function, FunctionWithDerivative};

/// `Geom2dGcc_FunctionTanCuPnt`: with `P1` the curve point of parameter `u` and `C`
/// the given point, `F(u) = CP1 ^ T / (|CP1| |T|)`, zero where the line `C P1` is
/// tangent to the curve; a `math_FunctionWithDerivative`.
/// ```
/// use rustycad_tkgeomalgo::geom2d_gcc::FunctionTanCuPnt;
/// use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};
/// use rustycad_tkmath::math::Function;
/// let axis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0)?);
/// let circle: Handle<dyn Geom2dCurve> = Handle::new(Geom2dCircle::from_axis(axis, 1.0, true)?).cast();
/// let adaptor = Geom2dAdaptorCurve::with_curve(&circle)?;
/// let mut f = FunctionTanCuPnt::new(&adaptor, Point2::new(1.0, 5.0));
/// let mut value = 1.0;
/// assert!(f.value(0.0, &mut value) && value.abs() < 1e-15);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct FunctionTanCuPnt<'a> {
    the_curv: &'a Geom2dAdaptorCurve,
    the_point: Point2,
    failure: EvalFailure,
}

impl<'a> FunctionTanCuPnt<'a> {
    /// `Geom2dGcc_FunctionTanCuPnt(C, Point)`; `FunctionTanCuPnt::new(&adaptor, point)`.
    pub fn new(c: &'a Geom2dAdaptorCurve, point: Point2) -> Self {
        Self {
            the_curv: c,
            the_point: point,
            failure: EvalFailure::default(),
        }
    }

    /// The exception a curve evaluation raised natively, after a member answered
    /// `false`; `f.raised()?`.
    pub fn raised(&self) -> Result<(), Failure> {
        self.failure.check()
    }

    // (F, F') at the curve point `point` of derivatives `vec1`, `vec2`.
    fn values_at(&self, point: Point2, vec1: Vector2, vec2: Vector2) -> (f64, f64) {
        let the_direction = Vector2::between(self.the_point, point);
        let norme_d1: f64 = vec1.magnitude();
        let norme_dir: f64 = the_direction.magnitude();
        let fval: f64 = the_direction.crossed(vec1) / (norme_d1 * norme_dir);
        let deriv: f64 = the_direction.crossed(vec2) / (norme_d1 * norme_dir)
            - (the_direction.crossed(vec1) / (norme_d1 * norme_dir))
                * (vec1.dot(vec2) / (norme_d1 * norme_d1)
                    + vec1.dot(the_direction) / (norme_dir * norme_dir));
        (fval, deriv)
    }
}

impl Function for FunctionTanCuPnt<'_> {
    fn value(&mut self, x: f64, f: &mut f64) -> bool {
        let Some((point, vect)) = self.failure.keep(CurveTool::d1(self.the_curv, x)) else {
            return false;
        };
        let norme_d1: f64 = vect.magnitude();
        let the_direction = Vector2::between(self.the_point, point);
        let norme_dir: f64 = the_direction.magnitude();
        *f = the_direction.crossed(vect) / (norme_d1 * norme_dir);
        true
    }
}

impl FunctionWithDerivative for FunctionTanCuPnt<'_> {
    fn derivative(&mut self, x: f64, deriv: &mut f64) -> bool {
        let Some((point, vec1, vec2)) = self.failure.keep(CurveTool::d2(self.the_curv, x)) else {
            return false;
        };
        *deriv = self.values_at(point, vec1, vec2).1;
        true
    }

    fn values(&mut self, x: f64, f: &mut f64, deriv: &mut f64) -> bool {
        let Some((point, vec1, vec2)) = self.failure.keep(CurveTool::d2(self.the_curv, x)) else {
            return false;
        };
        (*f, *deriv) = self.values_at(point, vec1, vec2);
        true
    }
}
