// Copyright (c) 1994-1999 Matra Datavision
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
// Rust translation of the static helpers UnifyByInsertingAllKnots and
// UnifyBySettingMiddleKnots of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Profiler.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The two ways `GeomFill_Profiler::Perform` gives its curves one knot vector.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{one_based, put};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::geom::GeomBSplineCurve;

/// `theCurves(Index)`: Standard_OutOfRange outside `1..=Length`, as the native
/// sequence; `curve_at(&curves, 1)?`.
pub(super) fn curve_at(
    curves: &[Handle<GeomBSplineCurve>],
    index: i32,
) -> Result<&GeomBSplineCurve, Failure> {
    let handle = usize::try_from(index - 1)
        .ok()
        .and_then(|i| curves.get(i))
        .ok_or_else(|| {
            errors::out_of_range(&format!(
                "NCollection_Sequence::Value: index {index}; expected 1..={}",
                curves.len()
            ))
        })?;
    handle
        .get()
        .ok_or_else(|| errors::null_object("GeomFill_Profiler: null section curve"))
}

/// `UnifyByInsertingAllKnots`: the knots of every curve go into the first one, the
/// first one's knots into the others, then the weights of each rational curve are
/// divided by their mean ("essai" of the native code); `unify_by_inserting_all_knots(&c, 1e-9)?`.
pub(super) fn unify_by_inserting_all_knots(
    curves: &[Handle<GeomBSplineCurve>],
    ptol: f64,
) -> Result<(), Failure> {
    let first: &GeomBSplineCurve = curve_at(curves, 1)?;
    for i in 2..=curves.len() as i32 {
        let other: &GeomBSplineCurve = curve_at(curves, i)?;
        first.insert_knots(&other.knots(), &other.multiplicities(), ptol, false)?;
    }
    let (new_knots, new_mults) = (first.knots(), first.multiplicities());
    for i in 2..=curves.len() as i32 {
        curve_at(curves, i)?.insert_knots(&new_knots, &new_mults, ptol, false)?;
    }
    for i in 1..=curves.len() as i32 {
        normalize_weights(curve_at(curves, i)?)?;
    }
    Ok(())
}

/// The weights of a rational curve divided by their mean.
fn normalize_weights(curve: &GeomBSplineCurve) -> Result<(), Failure> {
    if !curve.is_rational() {
        return Ok(());
    }
    let np: i32 = curve.nb_poles();
    let mut sigma: f64 = 0.0;
    for j in 1..=np {
        sigma += curve.weight(j)?;
    }
    sigma /= f64::from(np);
    for j in 1..=np {
        curve.set_weight(j, curve.weight(j)? / sigma)?;
    }
    Ok(())
}

/// `UnifyBySettingMiddleKnots`: every curve takes the mean of the inner knots of
/// all curves, between the bounds of the first one; a curve with fewer knots
/// raises Standard_OutOfRange, one with another knot count Standard_DimensionError
/// (`SetKnots`); `unify_by_setting_middle_knots(&curves)?`.
pub(super) fn unify_by_setting_middle_knots(
    curves: &[Handle<GeomBSplineCurve>],
) -> Result<(), Failure> {
    let first: &GeomBSplineCurve = curve_at(curves, 1)?;
    let nb_knots: i32 = first.nb_knots();
    let u_last: f64 = first.knot(first.last_u_knot_index()?)?;
    let u_first: f64 = first.knot(first.first_u_knot_index()?)?;
    let mut new_knots: Array1<'static, f64> = one_based(nb_knots)?;
    put(&mut new_knots, 1, u_first)?;
    put(&mut new_knots, nb_knots, u_last)?;
    for j in 2..nb_knots {
        let mut mid_knot: f64 = 0.0;
        for i in 1..=curves.len() as i32 {
            mid_knot += curve_at(curves, i)?.knot(j)?;
        }
        mid_knot /= curves.len() as f64;
        put(&mut new_knots, j, mid_knot)?;
    }
    for i in 1..=curves.len() as i32 {
        curve_at(curves, i)?.set_knots(&new_knots)?;
    }
    Ok(())
}
