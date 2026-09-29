// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/AppBlend/AppBlend_Approx.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppBlend_Approx`: the result of a surface approximation, a rational B-spline
//! surface and B-spline curves in its parameters.
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3};

/// The arrays of the approximated surface (`Surface(TPoles, TWeights, ...)`).
#[derive(Clone)]
pub struct AppBlendSurface {
    /// Poles, U in rows and V in columns.
    pub poles: Array2<'static, Point3>,
    /// Weights, like the poles.
    pub weights: Array2<'static, f64>,
    /// U knots.
    pub u_knots: Array1<'static, f64>,
    /// V knots.
    pub v_knots: Array1<'static, f64>,
    /// U multiplicities.
    pub u_mults: Array1<'static, i32>,
    /// V multiplicities.
    pub v_mults: Array1<'static, i32>,
}

/// The arrays of an approximated 2d curve (`Curve2d(Index, TPoles, TKnots, TMults)`).
#[derive(Clone)]
pub struct AppBlendCurve2d {
    /// Poles.
    pub poles: Array1<'static, Point2>,
    /// Knots (the V knots of the surface).
    pub knots: Array1<'static, f64>,
    /// Multiplicities (the V multiplicities of the surface).
    pub mults: Array1<'static, i32>,
}

/// `AppBlend_Approx`: the queries of a surface approximation. The accessors raise
/// `StdFail_NotDone` before a successful computation, and those of the 2d curves
/// `Standard_DomainError` when there is none; `approx.u_degree()?`.
pub trait AppBlendApprox {
    /// `IsDone()`.
    fn is_done(&self) -> bool;
    /// `SurfShape`: `(UDegree, VDegree, NbUPoles, NbVPoles, NbUKnots, NbVKnots)`.
    fn surf_shape(&self) -> Result<[i32; 6], Failure>;
    /// `Surface(...)`: copies of the surface arrays.
    fn surface(&self) -> Result<AppBlendSurface, Failure>;
    /// `UDegree()`.
    fn u_degree(&self) -> Result<i32, Failure>;
    /// `VDegree()`.
    fn v_degree(&self) -> Result<i32, Failure>;
    /// `SurfPoles()`.
    fn surf_poles(&self) -> Result<&Array2<'static, Point3>, Failure>;
    /// `SurfWeights()`.
    fn surf_weights(&self) -> Result<&Array2<'static, f64>, Failure>;
    /// `SurfUKnots()`.
    fn surf_u_knots(&self) -> Result<&Array1<'static, f64>, Failure>;
    /// `SurfVKnots()`.
    fn surf_v_knots(&self) -> Result<&Array1<'static, f64>, Failure>;
    /// `SurfUMults()`.
    fn surf_u_mults(&self) -> Result<&Array1<'static, i32>, Failure>;
    /// `SurfVMults()`.
    fn surf_v_mults(&self) -> Result<&Array1<'static, i32>, Failure>;
    /// `NbCurves2d()`.
    fn nb_curves2d(&self) -> Result<i32, Failure>;
    /// `Curves2dShape`: `(Degree, NbPoles, NbKnots)`.
    fn curves2d_shape(&self) -> Result<[i32; 3], Failure>;
    /// `Curve2d(Index, ...)`: copies of the arrays of curve `index`.
    fn curve2d(&self, index: i32) -> Result<AppBlendCurve2d, Failure>;
    /// `Curves2dDegree()`.
    fn curves2d_degree(&self) -> Result<i32, Failure>;
    /// `Curve2dPoles(Index)`.
    fn curve2d_poles(&self, index: i32) -> Result<&Array1<'static, Point2>, Failure>;
    /// `Curves2dKnots()`.
    fn curves2d_knots(&self) -> Result<&Array1<'static, f64>, Failure>;
    /// `Curves2dMults()`.
    fn curves2d_mults(&self) -> Result<&Array1<'static, i32>, Failure>;
    /// `TolReached`: `(Tol3d, Tol2d)`.
    fn tol_reached(&self) -> Result<(f64, f64), Failure>;
    /// `TolCurveOnSurf(Index)`.
    fn tol_curve_on_surf(&self, index: i32) -> f64;
}
