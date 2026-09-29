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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_*Of*.hxx/_0.cxx, AppDef_The*.hxx/_0.cxx,
// AppDef_Compute.hxx/_0.cxx and AppDef_BSplineCompute.hxx/_0.cxx (the generic instantiations)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The AppDef instantiations of the AppParCurves and Approx generics over
//! AppDef_MultiLine and AppDef_MyLineTool. Each native `_0.cxx` defines the generic
//! parameters and includes the `.gxx`; the port's generics take any
//! [`crate::app_par_curves::LineTool`], so every instantiation is an alias of the
//! generic over [`MultiLine`] (the generics that take the line per call are the same
//! type for every line).
use super::multi_line::MultiLine;
use crate::app_par_curves::{
    BSpFunction, BSpGradient, Function, Gradient, GradientBfgs, LeastSquare, ResolConstraint,
};
use crate::approx::{BSplComputeLine, ComputeLine};

/// `AppDef_Compute`: `Approx_ComputeLine` over AppDef_MultiLine, the Bezier
/// approximation of a MultiLine with cutting;
/// `Compute::with_line(&line, ComputeLineOptions::default())?`.
pub type Compute = ComputeLine;
/// `AppDef_BSplineCompute`: `Approx_BSplComputeLine` over AppDef_MultiLine, the
/// B-spline approximation; `BSplineCompute::with_line(&line, options)?`.
pub type BSplineCompute = BSplComputeLine;

/// `AppDef_TheLeastSquares`: `AppParCurves_LeastSquare` over AppDef_MultiLine;
/// `TheLeastSquares::new(&line, 1, 5, first, last, &u, 4)?`.
pub type TheLeastSquares = LeastSquare;
/// `AppDef_ParLeastSquareOfTheGradient`: the least squares of [`TheGradient`].
pub type ParLeastSquareOfTheGradient = LeastSquare;
/// `AppDef_ParLeastSquareOfMyGradientOfCompute`: the least squares of
/// [`MyGradientOfCompute`].
pub type ParLeastSquareOfMyGradientOfCompute = LeastSquare;
/// `AppDef_ParLeastSquareOfMyGradientbisOfBSplineCompute`: the least squares of
/// [`MyGradientbisOfBSplineCompute`].
pub type ParLeastSquareOfMyGradientbisOfBSplineCompute = LeastSquare;
/// `AppDef_BSpParLeastSquareOfMyBSplGradientOfBSplineCompute`: the B-spline least
/// squares of [`MyBSplGradientOfBSplineCompute`].
pub type BSpParLeastSquareOfMyBSplGradientOfBSplineCompute = LeastSquare;

/// `AppDef_TheResol`: `AppParCurves_ResolConstraint` over AppDef_MultiLine;
/// `TheResol::new(&line, &mut curve, 1, 5, &constraints, &a, &da, 1e-10)?`.
pub type TheResol = ResolConstraint;
/// `AppDef_ResConstraintOfTheGradient`: the constraint solver of [`TheGradient`].
pub type ResConstraintOfTheGradient = ResolConstraint;
/// `AppDef_ResConstraintOfMyGradientOfCompute`: the constraint solver of
/// [`MyGradientOfCompute`].
pub type ResConstraintOfMyGradientOfCompute = ResolConstraint;
/// `AppDef_ResConstraintOfMyGradientbisOfBSplineCompute`: the constraint solver of
/// [`MyGradientbisOfBSplineCompute`].
pub type ResConstraintOfMyGradientbisOfBSplineCompute = ResolConstraint;

/// `AppDef_TheFunction`: `AppParCurves_Function` over AppDef_MultiLine;
/// `TheFunction::new(&line, 1, 5, &constraints, &u, 3)?`.
pub type TheFunction = Function<MultiLine>;
/// `AppDef_ParFunctionOfTheGradient`: the function of [`TheGradient`].
pub type ParFunctionOfTheGradient = Function<MultiLine>;
/// `AppDef_ParFunctionOfMyGradientOfCompute`: the function of [`MyGradientOfCompute`].
pub type ParFunctionOfMyGradientOfCompute = Function<MultiLine>;
/// `AppDef_ParFunctionOfMyGradientbisOfBSplineCompute`: the function of
/// [`MyGradientbisOfBSplineCompute`].
pub type ParFunctionOfMyGradientbisOfBSplineCompute = Function<MultiLine>;
/// `AppDef_BSpParFunctionOfMyBSplGradientOfBSplineCompute`: `AppParCurves_BSpFunction`
/// over AppDef_MultiLine, the function of [`MyBSplGradientOfBSplineCompute`].
pub type BSpParFunctionOfMyBSplGradientOfBSplineCompute = BSpFunction<MultiLine>;

/// `AppDef_TheGradient`: `AppParCurves_Gradient` over AppDef_MultiLine;
/// `TheGradient::new(&line, 1, 5, &constraints, &mut u, 3, 1e-3, 1e-6, 100)?`.
pub type TheGradient = Gradient;
/// `AppDef_MyGradientOfCompute`: the parameter optimizer of [`Compute`].
pub type MyGradientOfCompute = Gradient;
/// `AppDef_MyGradientbisOfBSplineCompute`: the Bezier parameter optimizer of
/// [`BSplineCompute`].
pub type MyGradientbisOfBSplineCompute = Gradient;
/// `AppDef_MyBSplGradientOfBSplineCompute`: `AppParCurves_BSpGradient` over
/// AppDef_MultiLine, the B-spline parameter optimizer of [`BSplineCompute`].
pub type MyBSplGradientOfBSplineCompute = BSpGradient;

/// `AppDef_Gradient_BFGSOfTheGradient`: `AppParCurves_Gradient_BFGS`, math_BFGS
/// stopping on the fit tolerances too.
pub type GradientBfgsOfTheGradient = GradientBfgs;
/// `AppDef_Gradient_BFGSOfMyGradientOfCompute`: see [`GradientBfgsOfTheGradient`].
pub type GradientBfgsOfMyGradientOfCompute = GradientBfgs;
/// `AppDef_Gradient_BFGSOfMyGradientbisOfBSplineCompute`: see
/// [`GradientBfgsOfTheGradient`].
pub type GradientBfgsOfMyGradientbisOfBSplineCompute = GradientBfgs;
/// `AppDef_BSpGradient_BFGSOfMyBSplGradientOfBSplineCompute`:
/// `AppParCurves_BSpGradient_BFGS`, the same criterion over the B-spline function.
pub type BSpGradientBfgsOfMyBSplGradientOfBSplineCompute = GradientBfgs;
