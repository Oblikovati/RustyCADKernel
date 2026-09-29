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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/Law at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package Law: scalar evolution laws (constant, linear, S-shaped,
//! interpolated, B-spline and composite laws) used by sweeps and blends, the scalar
//! B-spline `Law_BSpline` behind them, and the `Law` class functions.
//!
//! `Law_Function` is the trait [`LawFunction`] reached through
//! `Handle<dyn LawFunction>`; the concrete laws are handle-shared structs whose state
//! sits behind one lock (docs/geom-design.md). `Law_S` and `Law_Interpol` embed their
//! `Law_BSpFunc` parent. The `Law` class functions are free functions of this module.
mod bsp_func;
mod bsp_func_intervals;
mod bsp_func_laws;
mod bspline;
mod composite;
mod function;
mod interpolate;
mod interpolate_perform;
mod mix;

pub use bsp_func::LawBSpFunc;
pub use bsp_func_intervals::LawBSplineKnotSplitting;
pub use bsp_func_laws::{LawInterpol, LawS};
pub use bspline::LawBSpline;
pub use composite::LawComposite;
pub use function::{LawConstant, LawFunction, LawLinear};
pub use interpolate::LawInterpolate;
pub use mix::{mix_bnd, mix_bnd_law, mix_tgt, reparametrize, scale, scale_cub};
