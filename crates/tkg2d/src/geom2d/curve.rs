// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_Curve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_Curve`: parameterized curves of the plane.
use super::errors;
use super::geometry::{Geom2dGeometry, dump_base, dump_class_name, dump_geometry};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Transform2, Vector2};
use std::io;

/// Point and first derivative (`Geom2d_Curve::ResD1`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Geom2dCurveD1 {
    /// Point of the curve.
    pub point: Point2,
    /// First derivative.
    pub d1: Vector2,
}
/// Point and first two derivatives (`Geom2d_Curve::ResD2`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Geom2dCurveD2 {
    /// Point of the curve.
    pub point: Point2,
    /// First derivative.
    pub d1: Vector2,
    /// Second derivative.
    pub d2: Vector2,
}
/// Point and first three derivatives (`Geom2d_Curve::ResD3`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Geom2dCurveD3 {
    /// Point of the curve.
    pub point: Point2,
    /// First derivative.
    pub d1: Vector2,
    /// Second derivative.
    pub d2: Vector2,
    /// Third derivative.
    pub d3: Vector2,
}

/// A parameterized 2D curve: evaluation, parameter range, continuity and orientation.
///
/// The evaluators return the native exceptions as `Err`;
/// `let point: Point2 = curve.value(0.5)?;`.
pub trait Geom2dCurve: Geom2dGeometry {
    /// Reverse the parameterization (`Reverse`); `curve.reverse()?`.
    fn reverse(&self) -> Result<(), Failure>;
    /// Parameter on the reversed curve of the point of parameter `u` (`ReversedParameter`);
    /// `curve.reversed_parameter(0.25)`.
    fn reversed_parameter(&self, u: f64) -> f64;
    /// Parameter on the curve transformed by `transform` (`TransformedParameter`); `u` by default;
    /// `curve.transformed_parameter(0.5, transform)`.
    fn transformed_parameter(&self, u: f64, _transform: Transform2) -> f64 {
        u
    }
    /// Ratio between parameters before and after `transform` (`ParametricTransformation`); 1 by default;
    /// `curve.parametric_transformation(transform)`.
    fn parametric_transformation(&self, _transform: Transform2) -> f64 {
        1.
    }
    /// Reversed copy (`Reversed`); `let reversed = curve.reversed()?;`.
    fn reversed(&self) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let copy: Handle<dyn Geom2dCurve> = self.copy()?.cast::<dyn Geom2dCurve>();
        copy.reverse()?;
        Ok(copy)
    }
    /// First parameter; may be `-Precision::Infinite()` (`FirstParameter`); `curve.first_parameter()`.
    fn first_parameter(&self) -> f64;
    /// Last parameter; may be `Precision::Infinite()` (`LastParameter`); `curve.last_parameter()`.
    fn last_parameter(&self) -> f64;
    /// Whether the curve is closed (`IsClosed`); `curve.is_closed()?`.
    fn is_closed(&self) -> Result<bool, Failure>;
    /// Whether the parameterization is periodic (`IsPeriodic`); `curve.is_periodic()`.
    fn is_periodic(&self) -> bool;
    /// Period; Standard_NoSuchObject when the curve is not periodic (`Period`); `curve.period()?`.
    fn period(&self) -> Result<f64, Failure> {
        if !self.is_periodic() {
            return Err(errors::no_such_object(
                "Geom2d_Curve::Period: the curve is not periodic; expected IsPeriodic()",
            ));
        }
        Ok(self.last_parameter() - self.first_parameter())
    }
    /// Global continuity (`Continuity`); `curve.continuity()`.
    fn continuity(&self) -> Shape;
    /// Whether the continuity is at least `n` (`IsCN`); `curve.is_cn(2)?`.
    fn is_cn(&self, n: i32) -> Result<bool, Failure>;
    /// Point of parameter `u` (`EvalD0`); `curve.eval_d0(0.5)?`.
    fn eval_d0(&self, u: f64) -> Result<Point2, Failure>;
    /// Point and first derivative (`EvalD1`); `curve.eval_d1(0.5)?.d1`.
    fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure>;
    /// Point and two derivatives (`EvalD2`); `curve.eval_d2(0.5)?.d2`.
    fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure>;
    /// Point and three derivatives (`EvalD3`); `curve.eval_d3(0.5)?.d3`.
    fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure>;
    /// Derivative of order `n >= 1` (`EvalDN`); `curve.eval_dn(0.5, 4)?`.
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure>;
    /// Point of parameter `u` (`D0`); `curve.d0(0.5)?`.
    fn d0(&self, u: f64) -> Result<Point2, Failure> {
        self.eval_d0(u)
    }
    /// Point and first derivative as a pair (`D1`); `let (p, v1) = curve.d1(0.5)?;`.
    fn d1(&self, u: f64) -> Result<(Point2, Vector2), Failure> {
        let result: Geom2dCurveD1 = self.eval_d1(u)?;
        Ok((result.point, result.d1))
    }
    /// Point and two derivatives (`D2`); `let (p, v1, v2) = curve.d2(0.5)?;`.
    fn d2(&self, u: f64) -> Result<(Point2, Vector2, Vector2), Failure> {
        let result: Geom2dCurveD2 = self.eval_d2(u)?;
        Ok((result.point, result.d1, result.d2))
    }
    /// Point and three derivatives (`D3`); `let (p, v1, v2, v3) = curve.d3(0.5)?;`.
    fn d3(&self, u: f64) -> Result<(Point2, Vector2, Vector2, Vector2), Failure> {
        let result: Geom2dCurveD3 = self.eval_d3(u)?;
        Ok((result.point, result.d1, result.d2, result.d3))
    }
    /// Derivative of order `n` (`DN`); `curve.dn(0.5, 2)?`.
    fn dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        self.eval_dn(u, n)
    }
    /// Point of parameter `u`, through D0 (`Value`); `curve.value(0.5)?`.
    fn value(&self, u: f64) -> Result<Point2, Failure> {
        self.d0(u)
    }
}
impl_transient_interface!(dyn Geom2dCurve, "Geom2d_Curve", dyn Geom2dGeometry);

/// `Geom2d_Curve::DumpJson`; `dump_curve(&mut stream, -1)?`.
pub fn dump_curve(stream: &mut DumpStream, depth: i32) -> io::Result<()> {
    dump_class_name(stream, "Geom2d_Curve")?;
    dump_base(stream, depth, dump_geometry)
}
