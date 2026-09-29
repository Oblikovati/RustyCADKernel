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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BezierCurve.cxx, Geom2d_BSplineCurve_1.cxx and ModelingData/TKG3d/Geom/Geom_BezierCurve.cxx, Geom_BSplineCurve_1.cxx (the evaluation representation calls of the spline curves) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The dimension of a spline curve class (Geom2d or Geom): its point type, its
//! evaluation representation and the RepUtils calls that the Bezier and B-spline
//! evaluators try before their own evaluation.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib::{CurveD3, CurvePoint};

/// A curve derivative vector of the kind's point type.
pub type KindVector<K> = <<K as SplineCurveKind>::Point as CurvePoint>::Vector;
/// A point and its first derivative.
pub type KindD1<K> = (<K as SplineCurveKind>::Point, KindVector<K>);
/// A point and its first two derivatives.
pub type KindD2<K> = (<K as SplineCurveKind>::Point, KindVector<K>, KindVector<K>);
/// A point and its first three derivatives.
pub type KindD3<K> = CurveD3<<K as SplineCurveKind>::Point>;

/// The package of a spline curve class; each curve crate implements it on one
/// marker type, e.g. `impl SplineCurveKind for Geom2dSplineKind { type Point = Point2; ... }`.
pub trait SplineCurveKind: Sized + 'static {
    /// The pole type (`gp_Pnt2d` or `gp_Pnt`).
    type Point: CurvePoint;
    /// The evaluation representation handle of the package.
    type EvalRep: Clone + Default;
    /// The package `UndefinedDerivative` exception; `K::undefined_derivative("EvalDN")`.
    fn undefined_derivative(message: &str) -> Failure;
    /// `TryEvalCurveD0` of the package RepUtils; `K::rep_d0(&rep, 0.5)?`.
    fn rep_d0(eval_rep: &Self::EvalRep, u: f64) -> Result<Option<Self::Point>, Failure>;
    /// `TryEvalCurveD1`; `K::rep_d1(&rep, 0.5)?`.
    fn rep_d1(eval_rep: &Self::EvalRep, u: f64) -> Result<Option<KindD1<Self>>, Failure>;
    /// `TryEvalCurveD2`; `K::rep_d2(&rep, 0.5)?`.
    fn rep_d2(eval_rep: &Self::EvalRep, u: f64) -> Result<Option<KindD2<Self>>, Failure>;
    /// `TryEvalCurveD3`; `K::rep_d3(&rep, 0.5)?`.
    fn rep_d3(eval_rep: &Self::EvalRep, u: f64) -> Result<Option<KindD3<Self>>, Failure>;
    /// `TryEvalCurveDN`; `K::rep_dn(&rep, 0.5, 4)?`.
    fn rep_dn(
        eval_rep: &Self::EvalRep,
        u: f64,
        n: i32,
    ) -> Result<Option<KindVector<Self>>, Failure>;
}
