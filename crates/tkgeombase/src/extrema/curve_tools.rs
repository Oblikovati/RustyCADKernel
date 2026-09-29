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
// Rust translation of Extrema_CurveTool.hxx/.cxx and Extrema_Curve2dTool.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The two tool classes: [`CurveTool`] (`Extrema_CurveTool`, 3D) and [`Curve2dTool`]
//! (`Extrema_Curve2dTool`). They differ only in their types and in `IsPeriodic`.
use super::curve_tool::{ExtremaCurveTool, ToolD1, ToolD2, ToolD3};
use super::{ExtPElC, ExtPElC2d};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBezierCurve};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{
    Circle2, Circle3, Ellipse2, Ellipse3, Hyperbola2, Hyperbola3, Line2, Line3, Parabola2,
    Parabola3, Point2, Point3, Vector2, Vector3,
};

/// `Extrema_CurveTool`: the static access to an `Adaptor3d_Curve`;
/// `CurveTool::is_periodic(&adaptor)?`.
pub struct CurveTool;

/// `Extrema_Curve2dTool`: the static access to an `Adaptor2d_Curve2d`;
/// `Curve2dTool::is_periodic(&adaptor)?`.
pub struct Curve2dTool;

// The members that both tools forward to the adaptor unchanged.
macro_rules! forwarding_tool_members {
    () => {
        fn first_parameter(curve: &Self::Curve) -> Result<f64, Failure> {
            curve.first_parameter()
        }
        fn last_parameter(curve: &Self::Curve) -> Result<f64, Failure> {
            curve.last_parameter()
        }
        fn continuity(curve: &Self::Curve) -> Result<Shape, Failure> {
            curve.continuity()
        }
        fn nb_intervals(curve: &Self::Curve, shape: Shape) -> Result<i32, Failure> {
            curve.nb_intervals(shape)
        }
        fn intervals(
            curve: &Self::Curve,
            bounds: &mut Array1<'_, f64>,
            shape: Shape,
        ) -> Result<(), Failure> {
            curve.intervals(bounds, shape)
        }
        fn is_closed(curve: &Self::Curve) -> Result<bool, Failure> {
            curve.is_closed()
        }
        fn period(curve: &Self::Curve) -> Result<f64, Failure> {
            curve.period()
        }
        fn resolution(curve: &Self::Curve, r3d: f64) -> Result<f64, Failure> {
            curve.resolution(r3d)
        }
        fn get_type(curve: &Self::Curve) -> Result<CurveType, Failure> {
            curve.get_type()
        }
        fn value(curve: &Self::Curve, u: f64) -> Result<Self::Point, Failure> {
            curve.value(u)
        }
        fn d0(curve: &Self::Curve, u: f64) -> Result<Self::Point, Failure> {
            curve.d0(u)
        }
        fn d1(curve: &Self::Curve, u: f64) -> Result<ToolD1<Self>, Failure> {
            curve.d1(u)
        }
        fn d2(curve: &Self::Curve, u: f64) -> Result<ToolD2<Self>, Failure> {
            curve.d2(u)
        }
        fn d3(curve: &Self::Curve, u: f64) -> Result<ToolD3<Self>, Failure> {
            curve.d3(u)
        }
        fn dn(curve: &Self::Curve, u: f64, n: i32) -> Result<Self::Vector, Failure> {
            curve.dn(u, n)
        }
        fn line(curve: &Self::Curve) -> Result<Self::Line, Failure> {
            curve.line()
        }
        fn circle(curve: &Self::Curve) -> Result<Self::Circle, Failure> {
            curve.circle()
        }
        fn ellipse(curve: &Self::Curve) -> Result<Self::Ellipse, Failure> {
            curve.ellipse()
        }
        fn hyperbola(curve: &Self::Curve) -> Result<Self::Hyperbola, Failure> {
            curve.hyperbola()
        }
        fn parabola(curve: &Self::Curve) -> Result<Self::Parabola, Failure> {
            curve.parabola()
        }
        fn degree(curve: &Self::Curve) -> Result<i32, Failure> {
            curve.degree()
        }
        fn is_rational(curve: &Self::Curve) -> Result<bool, Failure> {
            curve.is_rational()
        }
        fn nb_poles(curve: &Self::Curve) -> Result<i32, Failure> {
            curve.nb_poles()
        }
        fn nb_knots(curve: &Self::Curve) -> Result<i32, Failure> {
            curve.nb_knots()
        }
        fn bezier(curve: &Self::Curve) -> Result<Handle<Self::Bezier>, Failure> {
            curve.bezier()
        }
        fn bspline(curve: &Self::Curve) -> Result<Handle<Self::BSpline>, Failure> {
            curve.bspline()
        }
    };
}

impl ExtremaCurveTool for CurveTool {
    type Curve = dyn Adaptor3dCurve;
    type Point = Point3;
    type Vector = Vector3;
    type Line = Line3;
    type Circle = Circle3;
    type Ellipse = Ellipse3;
    type Hyperbola = Hyperbola3;
    type Parabola = Parabola3;
    type Bezier = GeomBezierCurve;
    type BSpline = GeomBSplineCurve;
    type ExtPElC = ExtPElC;
    forwarding_tool_members!();
    /// `Extrema_CurveTool::IsPeriodic`: circles and ellipses are periodic whatever
    /// their adaptor says.
    fn is_periodic(curve: &Self::Curve) -> Result<bool, Failure> {
        let kind: CurveType = Self::get_type(curve)?;
        if kind == CurveType::Circle || kind == CurveType::Ellipse {
            return Ok(true);
        }
        curve.is_periodic()
    }
}

impl ExtremaCurveTool for Curve2dTool {
    type Curve = dyn Adaptor2dCurve2d;
    type Point = Point2;
    type Vector = Vector2;
    type Line = Line2;
    type Circle = Circle2;
    type Ellipse = Ellipse2;
    type Hyperbola = Hyperbola2;
    type Parabola = Parabola2;
    type Bezier = Geom2dBezierCurve;
    type BSpline = Geom2dBSplineCurve;
    type ExtPElC = ExtPElC2d;
    forwarding_tool_members!();
    /// `Extrema_Curve2dTool::IsPeriodic`: the adaptor's answer.
    fn is_periodic(curve: &Self::Curve) -> Result<bool, Failure> {
        curve.is_periodic()
    }
}
