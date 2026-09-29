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
// Rust translation of ModelingData/TKGeomBase/GeomConvert at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GeomConvert: conversion of Geom curves and surfaces to B-spline and Bezier
//! forms, splitting, concatenation, approximation, and the recognition of
//! canonical (analytic) curves and surfaces.
//!
//! The static members of the native `GeomConvert` class are free functions
//! (`GeomConvert::CurveToBSplineCurve` is [`curve_to_bspline_curve`]); each class of
//! the package is a struct. A native exception is a [`Failure`] with the same
//! exception type; the output reference parameters of the concatenation members stay
//! `&mut` parameters, because the native members assign them before they can raise.
//!
//! `let bspline = geom_convert::curve_to_bspline_curve(&curve, ParameterisationType::TgtThetaOver2)?;`
mod approx_curve;
mod approx_surface;
mod bspline_surface_to_bezier;
mod bspline_to_bezier;
mod c0_to_c1;
mod comp_bezier_surfaces;
mod comp_bezier_surfaces_perform;
mod comp_curve;
mod concat;
mod concat_support;
mod conv_type;
mod curve_to_ana_conic;
mod curve_to_ana_curve;
mod curve_to_ana_select;
mod curve_to_bspline;
mod func_ls_dist;
mod knot_splitting;
mod reparametrize;
mod split_surface;
mod surf_to_ana_convert;
mod surf_to_ana_gauss;
mod surf_to_ana_surf;
mod surf_to_ana_try;
mod surface_to_bspline;
mod surface_to_bspline_trimmed;
mod units;

pub use approx_curve::ApproxCurve;
pub(crate) use approx_curve::missing_table;
pub use approx_surface::ApproxSurface;
pub use bspline_surface_to_bezier::BSplineSurfaceToBezierSurface;
pub use bspline_to_bezier::BSplineCurveToBezierCurve;
pub use c0_to_c1::{
    c0_bspline_to_array_of_c1_bspline_curve, c0_bspline_to_array_of_c1_bspline_curve_with_angular,
    c0_bspline_to_c1_bspline_curve,
};
pub use comp_bezier_surfaces::CompBezierSurfacesToBSplineSurface;
pub use comp_curve::CompCurveToBSplineCurve;
pub use concat::{ConcatOutputs, concat_c1, concat_c1_with_angular, concat_g1};
pub use conv_type::ConvType;
pub use curve_to_ana_curve::{AnaCurveFit, CurveToAnaCurve};
pub use curve_to_bspline::curve_to_bspline_curve;
pub use func_ls_dist::{FuncConeLSDist, FuncCylinderLSDist, FuncSphereLSDist, HArrayOfXyz};
pub use knot_splitting::{BSplineCurveKnotSplitting, BSplineSurfaceKnotSplitting};
pub use split_surface::{
    split_bspline_surface, split_bspline_surface_knots, split_bspline_surface_one_direction,
    split_bspline_surface_one_direction_knots,
};
pub use surf_to_ana_surf::SurfToAnaSurf;
pub use surface_to_bspline::surface_to_bspline_surface;
pub use units::Units;

use crate::native_failure::failure;
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBSplineSurface, GeomCurve, GeomGeometry};

/// The shared array of B-splines the concatenation members fill
/// (`occ::handle<NCollection_HArray1<occ::handle<Geom_BSplineCurve>>>`).
pub type HArrayOfBSplineCurves = Handle<HArray1<Handle<GeomBSplineCurve>>>;

/// `GeomConvert::SplitBSplineCurve(C, FromK1, ToK2, SameOrientation = true)`: the arc
/// of `curve` between the knots of indices `from_k1` and `to_k2`, as a new curve.
///
/// A periodic curve keeps its orientation unless `same_orientation` is false; any
/// other curve runs from `from_k1` to `to_k2`. Standard_DomainError when the indices
/// are equal or one lies outside `[FirstUKnotIndex, LastUKnotIndex]` (the 3D member
/// raises Standard_DomainError for both, where Geom2dConvert raises
/// Standard_OutOfRange for the second);
/// `geom_convert::split_bspline_curve_knots(&curve, 1, 3, true)?`.
pub fn split_bspline_curve_knots(
    curve: &Handle<GeomBSplineCurve>,
    from_k1: i32,
    to_k2: i32,
    same_orientation: bool,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let source: &GeomBSplineCurve = live_bspline(curve)?;
    let (first, last) = (source.first_u_knot_index()?, source.last_u_knot_index()?);
    if from_k1 == to_k2 {
        return Err(failure(
            "Standard_DomainError",
            &format!(
                "GeomConvert::SplitBSplineCurve: FromK1 = ToK2 = {from_k1}; expected distinct knots"
            ),
        ));
    }
    let (first_k, last_k) = (from_k1.min(to_k2), from_k1.max(to_k2));
    if first_k < first || last_k > last {
        return Err(failure(
            "Standard_DomainError",
            &format!(
                "GeomConvert::SplitBSplineCurve: knots {from_k1}..{to_k2}; expected within {first}..{last}"
            ),
        ));
    }
    let c1: Handle<GeomBSplineCurve> = copy_bspline(source)?;
    live_bspline(&c1)?.segment(
        source.knot(first_k)?,
        source.knot(last_k)?,
        precision::p_confusion(),
    )?;
    orient_split(&c1, source.is_periodic(), same_orientation, from_k1 > to_k2)?;
    Ok(c1)
}

/// `GeomConvert::SplitBSplineCurve(C, FromU1, ToU2, ParametricTolerance,
/// SameOrientation = true)`: the arc of `curve` between two parameters.
///
/// The native member ignores `parametric_tolerance` and checks no bound: the
/// B-spline `Segment` raises for parameters it rejects;
/// `geom_convert::split_bspline_curve(&curve, 0.2, 0.8, 1e-9, true)?`.
pub fn split_bspline_curve(
    curve: &Handle<GeomBSplineCurve>,
    from_u1: f64,
    to_u2: f64,
    _parametric_tolerance: f64,
    same_orientation: bool,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let source: &GeomBSplineCurve = live_bspline(curve)?;
    let (first_u, last_u) = (from_u1.min(to_u2), from_u1.max(to_u2));
    let c1: Handle<GeomBSplineCurve> = copy_bspline(source)?;
    live_bspline(&c1)?.segment(first_u, last_u, precision::p_confusion())?;
    // The native test is `FromU1 > ToU2`, false for NaN.
    orient_split(&c1, source.is_periodic(), same_orientation, from_u1 > to_u2)?;
    Ok(c1)
}

// Reverses a split arc: a periodic curve when the other orientation is asked, any
// other curve when its bounds were given in decreasing order.
fn orient_split(
    curve: &Handle<GeomBSplineCurve>,
    periodic: bool,
    same_orientation: bool,
    decreasing: bool,
) -> Result<(), Failure> {
    let reverse: bool = if periodic {
        !same_orientation
    } else {
        decreasing
    };
    if reverse {
        live_bspline(curve)?.reverse()?;
    }
    Ok(())
}

/// The object of a non-null handle; Standard_NullObject where the native code
/// dereferences a null handle.
pub(crate) fn live_bspline(curve: &Handle<GeomBSplineCurve>) -> Result<&GeomBSplineCurve, Failure> {
    curve.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "GeomConvert: null Geom_BSplineCurve handle; expected a curve",
        )
    })
}

/// The object of a non-null B-spline surface handle; Standard_NullObject where the
/// native code dereferences a null handle.
pub(crate) fn live_bspline_surface(
    surface: &Handle<GeomBSplineSurface>,
) -> Result<&GeomBSplineSurface, Failure> {
    surface.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "GeomConvert: null Geom_BSplineSurface handle; expected a surface",
        )
    })
}

/// `occ::down_cast<Geom_BSplineCurve>(C->Copy())`.
pub(crate) fn copy_bspline(curve: &GeomBSplineCurve) -> Result<Handle<GeomBSplineCurve>, Failure> {
    Ok(curve.copy()?.cast::<GeomBSplineCurve>())
}

/// `occ::down_cast<Geom_BSplineSurface>(S->Copy())`.
pub(crate) fn copy_bspline_surface(
    surface: &GeomBSplineSurface,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    Ok(surface.copy()?.cast::<GeomBSplineSurface>())
}

/// `Handle<GeomBSplineCurve>` seen as a `Geom_Curve`, as the native implicit up cast.
pub(crate) fn as_curve(curve: &Handle<GeomBSplineCurve>) -> Handle<dyn GeomCurve> {
    curve.cast::<dyn GeomCurve>()
}
