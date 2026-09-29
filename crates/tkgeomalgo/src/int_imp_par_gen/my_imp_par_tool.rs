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
// Rust translation of the IntImpParGen_MyImpParTool function class, expanded natively in
// IntCurve_MyImpParToolOfIntImpConicParConic_0.cxx and
// Geom2dInt_MyImpParToolOfTheIntersectorOfTheIntConicCurveOfGInter_0.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntImpParGen_MyImpParTool`: the signed distance from the implicit curve along
//! the parametric curve, the function whose roots the intersector searches.
use super::traits::{ImpTool, IntersectorTypes, ParTool};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Vector2};
use rustycad_tkmath::math::{Function, FunctionWithDerivative};

/// The math_FunctionWithDerivative of one IntImpParGen instantiation (the native
/// `IntCurve_MyImpParToolOfIntImpConicParConic` and
/// `Geom2dInt_MyImpParToolOfTheIntersectorOfTheIntConicCurveOfGInter`): it keeps a
/// copy of the implicit tool and a reference to the parametric curve. The math
/// traits cannot carry an exception, so a curve failure is kept
/// ([`MyImpParTool::take_failure`]) and reported as `false`; the intersector
/// returns it once the root search stops.
///
/// ```
/// use rustycad_tkgeomalgo::int_curve::{IConicTool, MyImpParToolOfIntImpConicParConic, PConic};
/// use rustycad_tkmath::gp::{Line2, Point2, Direction2};
/// use rustycad_tkmath::math::Function;
/// let line = Line2::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
/// let other = Line2::new(Point2::new(0.0, 1.0), Direction2::new(0.0, 1.0).unwrap());
/// let pconic = PConic::from_line(other).unwrap();
/// let mut f = MyImpParToolOfIntImpConicParConic::new(&IConicTool::from_line(line).unwrap(), &pconic);
/// let mut value = 0.0;
/// assert!(f.value(-1.0, &mut value) && value == 0.0);
/// ```
pub struct MyImpParTool<'a, T: IntersectorTypes> {
    the_par_curve: &'a <T::Par as ParTool>::Curve,
    the_imp_tool: T::Imp,
    failure: Option<Failure>,
}

impl<'a, T: IntersectorTypes> MyImpParTool<'a, T> {
    /// `MyImpParTool(IT, PC)`: copies the implicit tool;
    /// `MyImpParToolOfIntImpConicParConic::new(&itool, &pconic)`.
    pub fn new(imp_tool: &T::Imp, par_curve: &'a <T::Par as ParTool>::Curve) -> Self {
        Self {
            the_par_curve: par_curve,
            the_imp_tool: imp_tool.clone(),
            failure: None,
        }
    }
    /// The curve failure raised by the last evaluation, if any (the native
    /// exception); `function.take_failure()`.
    pub fn take_failure(&mut self) -> Option<Failure> {
        self.failure.take()
    }

    // `Value(Param, ApproxDistance)`.
    fn distance_at(&self, param: f64) -> Result<f64, Failure> {
        let point: Point2 = T::Par::value(self.the_par_curve, param)?;
        Ok(self.the_imp_tool.distance(point))
    }
    // `Derivative(Param, D_ApproxDistance_DV)`: the gradient of the distance at the
    // point, dotted with the tangent of the parametric curve.
    fn derivative_at(&self, param: f64) -> Result<f64, Failure> {
        let point: Point2 = T::Par::value(self.the_par_curve, param)?;
        let grad: Vector2 = self.the_imp_tool.grad_distance(point);
        let (mut pt, mut tan_par_curve) = (Point2::default(), Vector2::default());
        T::Par::d1(self.the_par_curve, param, &mut pt, &mut tan_par_curve)?;
        Ok(grad.dot(tan_par_curve))
    }
    // Stores the failure of an evaluation and reports it as `false`.
    fn keep(&mut self, value: Result<f64, Failure>, output: &mut f64) -> bool {
        match value {
            Ok(value) => {
                *output = value;
                true
            }
            Err(failure) => {
                self.failure = Some(failure);
                false
            }
        }
    }
}

impl<T: IntersectorTypes> Function for MyImpParTool<'_, T> {
    fn value(&mut self, param: f64, approx_distance: &mut f64) -> bool {
        let value: Result<f64, Failure> = self.distance_at(param);
        self.keep(value, approx_distance)
    }
}

impl<T: IntersectorTypes> FunctionWithDerivative for MyImpParTool<'_, T> {
    fn derivative(&mut self, param: f64, derivative: &mut f64) -> bool {
        let value: Result<f64, Failure> = self.derivative_at(param);
        self.keep(value, derivative)
    }
    fn values(&mut self, param: f64, approx_distance: &mut f64, deriv: &mut f64) -> bool {
        // The native Values ignores both results and returns true.
        self.value(param, approx_distance);
        self.derivative(param, deriv);
        true
    }
}
