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
// Rust translation of Geom2dGcc_FunctionTanObl.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGcc_FunctionTanObl`: the angle function of a line of given direction
//! tangent to a curve.
use super::{CurveTool, EvalFailure};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::gp::{Direction2, Vector2};
use rustycad_tkmath::math::{Function, FunctionWithDerivative};

/// `Geom2dGcc_FunctionTanObl`: `F(u) = Dir ^ T(u) / |T(u)|`, zero where the
/// tangent of the curve is parallel to `Dir`; a `math_FunctionWithDerivative`.
/// ```
/// use rustycad_tkgeomalgo::geom2d_gcc::FunctionTanObl;
/// use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};
/// use rustycad_tkmath::math::Function;
/// let axis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0)?);
/// let circle: Handle<dyn Geom2dCurve> = Handle::new(Geom2dCircle::from_axis(axis, 1.0, true)?).cast();
/// let adaptor = Geom2dAdaptorCurve::with_curve(&circle)?;
/// let mut f = FunctionTanObl::new(&adaptor, Direction2::new(0.0, 1.0)?);
/// let mut value = 1.0;
/// assert!(f.value(0.0, &mut value) && value.abs() < 1e-15);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct FunctionTanObl<'a> {
    the_curv: &'a Geom2dAdaptorCurve,
    the_direction: Direction2,
    failure: EvalFailure,
}

impl<'a> FunctionTanObl<'a> {
    /// `Geom2dGcc_FunctionTanObl(Curve, Dir)`; `FunctionTanObl::new(&adaptor, dir)`.
    pub fn new(curve: &'a Geom2dAdaptorCurve, dir: Direction2) -> Self {
        Self {
            the_curv: curve,
            the_direction: dir,
            failure: EvalFailure::default(),
        }
    }

    /// The exception a curve evaluation raised natively, after a member answered
    /// `false`; `f.raised()?`.
    pub fn raised(&self) -> Result<(), Failure> {
        self.failure.check()
    }

    // TheDirection ^ Vec2 / |Vec1| - (Vec1 . Vec2) * (TheDirection ^ Vec1) / |Vec1|.
    fn derivative_of(&self, vec1: Vector2, vec2: Vector2) -> f64 {
        let dir = self.the_direction.xy();
        let norme_d1: f64 = vec1.magnitude();
        dir.crossed(vec2.xy()) / norme_d1
            - vec1.xy().dot(vec2.xy()) * dir.crossed(vec1.xy()) / norme_d1
    }
}

impl Function for FunctionTanObl<'_> {
    fn value(&mut self, x: f64, f: &mut f64) -> bool {
        let Some((_, vect)) = self.failure.keep(CurveTool::d1(self.the_curv, x)) else {
            return false;
        };
        let norme_d1: f64 = vect.magnitude();
        *f = self.the_direction.xy().crossed(vect.xy()) / norme_d1;
        true
    }
}

impl FunctionWithDerivative for FunctionTanObl<'_> {
    fn derivative(&mut self, x: f64, deriv: &mut f64) -> bool {
        let Some((_, vec1, vec2)) = self.failure.keep(CurveTool::d2(self.the_curv, x)) else {
            return false;
        };
        *deriv = self.derivative_of(vec1, vec2);
        true
    }

    fn values(&mut self, x: f64, f: &mut f64, deriv: &mut f64) -> bool {
        let Some((_, vec1, vec2)) = self.failure.keep(CurveTool::d2(self.the_curv, x)) else {
            return false;
        };
        let norme_d1: f64 = vec1.magnitude();
        *f = self.the_direction.xy().crossed(vec1.xy()) / norme_d1;
        *deriv = self.derivative_of(vec1, vec2);
        true
    }
}
