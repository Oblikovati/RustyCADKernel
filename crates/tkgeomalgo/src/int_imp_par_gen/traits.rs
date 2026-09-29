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
// Rust translation of the template parameters of IntImpParGen_Intersector.gxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The type parameters of the IntImpParGen template (docs/geomalgo-design.md): the
//! `ImpTool`, `ParCurve`/`ParTool` and `ProjectOnPCurveTool` macros, as traits.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Vector2};

/// The `ImpTool` of the template: an implicit curve with a parametrisation and a
/// signed distance (IntCurve_IConicTool). D1 and D2 write into the caller's values,
/// as the native output references, so that a curve kind that does not set an
/// output leaves it unchanged; `tool.d2(0.5, &mut p, &mut t, &mut n)`.
pub trait ImpTool: Clone {
    /// `Value(X)`: the point of parameter `x`.
    fn value(&self, x: f64) -> Point2;
    /// `D1(U, P, T)`: the point and the first derivative.
    fn d1(&self, u: f64, p: &mut Point2, t: &mut Vector2);
    /// `D2(U, P, T, N)`: the point and the first two derivatives.
    fn d2(&self, u: f64, p: &mut Point2, t: &mut Vector2, n: &mut Vector2);
    /// `Distance(P)`: the signed distance from the point to the curve.
    fn distance(&self, p: Point2) -> f64;
    /// `GradDistance(P)`: the gradient of the signed distance at the point.
    fn grad_distance(&self, p: Point2) -> Vector2;
    /// `FindParameter(P)`: the parameter of the point of the curve matching `p`
    /// (a gp exception for a point at the centre of a circle or an ellipse).
    fn find_parameter(&self, p: Point2) -> Result<f64, Failure>;
}

/// The `ParCurve` and `ParTool` of the template: a bounded parametric curve and the
/// static tool that samples and evaluates it (IntCurve_PConic with
/// IntCurve_PConicTool, Adaptor2d_Curve2d with Geom2dInt_Geom2dCurveTool). The
/// curve's own exceptions are returned as `Err`;
/// `T::d2(&curve, 0.5, &mut p, &mut t, &mut n)?`.
pub trait ParTool {
    /// The parametric curve type.
    type Curve: ?Sized;
    /// `EpsX(C)`: the parametric tolerance of the root search.
    fn eps_x(curve: &Self::Curve) -> f64;
    /// `NbSamples(C, U0, U1)`: the samples of the root search on `[u0, u1]`.
    fn nb_samples(curve: &Self::Curve, u0: f64, u1: f64) -> Result<i32, Failure>;
    /// `Value(C, X)`: the point of parameter `x`.
    fn value(curve: &Self::Curve, x: f64) -> Result<Point2, Failure>;
    /// `D1(C, U, P, T)`: the point and the first derivative.
    fn d1(curve: &Self::Curve, u: f64, p: &mut Point2, t: &mut Vector2) -> Result<(), Failure>;
    /// `D2(C, U, P, T, N)`: the point and the first two derivatives.
    fn d2(
        curve: &Self::Curve,
        u: f64,
        p: &mut Point2,
        t: &mut Vector2,
        n: &mut Vector2,
    ) -> Result<(), Failure>;
}

/// The `ProjectOnPCurveTool` of the template: the parameter of a point near the
/// parametric curve (IntCurve_ProjectOnPConicTool, Geom2dInt_TheProjPCurOfGInter);
/// `P::find_parameter(&curve, point, 1e-7)?`.
pub trait ProjectOnPCurveTool {
    /// The parametric curve type.
    type Curve: ?Sized;
    /// `FindParameter(C, Pnt, Tol)`: on the natural domain of the curve.
    fn find_parameter(curve: &Self::Curve, pnt: Point2, tol: f64) -> Result<f64, Failure>;
    /// `FindParameter(C, Pnt, LowParameter, HighParameter, Tol)`: the parameter
    /// certainly lies between the two bounds.
    fn find_parameter_in(
        curve: &Self::Curve,
        pnt: Point2,
        low_parameter: f64,
        high_parameter: f64,
        tol: f64,
    ) -> Result<f64, Failure>;
}

/// One instantiation of IntImpParGen_Intersector.gxx: the four macro types.
pub trait IntersectorTypes: 'static {
    /// `ImpTool`.
    type Imp: ImpTool;
    /// `ParTool` (with its `ParCurve`).
    type Par: ParTool;
    /// `ProjectOnPCurveTool`, over the same curve.
    type Project: ProjectOnPCurveTool<Curve = <Self::Par as ParTool>::Curve>;
}
