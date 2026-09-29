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
// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert.cxx
// (C0BSplineToC1BSplineCurve, C0BSplineToArrayOfC1BSplineCurve)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomConvert::C0BSplineToC1BSplineCurve` and
//! `GeomConvert::C0BSplineToArrayOfC1BSplineCurve`: the C1 pieces of a B-spline
//! with C0 knots, joined again where they are G1.
use super::comp_curve::CompCurveToBSplineCurve;
use super::concat::{ConcatOutputs, concat_c1_with_angular};
use super::concat_support::{cloned_at, stored};
use super::reparametrize::new_curve_table;
use super::{HArrayOfBSplineCurves, copy_bspline, live_bspline};
use crate::array_access::zero_based;
use crate::native_failure::{array_failure, failure};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBoundedCurve, GeomCurve as _};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::Vector3;

/// `GeomConvert::C0BSplineToC1BSplineCurve(BS, tolerance, AngularTolerance = 1e-7)`:
/// replaces `bs` by the concatenation of its C1 pieces (joined within `tolerance`);
/// Standard_ConstructionError ("GeomConvert Concatenation Error") when two pieces do
/// not join; `geom_convert::c0_bspline_to_c1_bspline_curve(&mut curve, 1e-7, 1e-7)?`.
pub fn c0_bspline_to_c1_bspline_curve(
    bs: &mut Handle<GeomBSplineCurve>,
    tolerance: f64,
    angular_tol: f64,
) -> Result<(), Failure> {
    let mut array_of_concatenated: HArrayOfBSplineCurves = Handle::default();
    c0_bspline_to_array_of_c1_bspline_curve_with_angular(
        bs,
        &mut array_of_concatenated,
        angular_tol,
        tolerance,
    )?;
    let table = array_of_concatenated
        .get()
        .ok_or_else(|| failure("Standard_NullObject", "null table"))?;
    let pieces: Array1<'static, Handle<GeomBSplineCurve>> = table.array1().clone();
    let mut c = CompCurveToBSplineCurve::with_curve(
        &cloned_at(&pieces, 0)?.cast::<dyn GeomBoundedCurve>(),
        ParameterisationType::TgtThetaOver2,
    )?;
    if pieces.length() >= 2 {
        for i in 1..pieces.length() {
            if !c.add(
                &cloned_at(&pieces, i)?.cast::<dyn GeomBoundedCurve>(),
                tolerance,
                false,
                true,
                0,
            )? {
                return Err(failure(
                    "Standard_ConstructionError",
                    "GeomConvert Concatenation Error",
                ));
            }
        }
    }
    *bs = c.bspline_curve();
    Ok(())
}

/// `GeomConvert::C0BSplineToArrayOfC1BSplineCurve(BS, tabBS, tolerance)`:
/// [`c0_bspline_to_array_of_c1_bspline_curve_with_angular`] with
/// Precision::Angular(); `geom_convert::c0_bspline_to_array_of_c1_bspline_curve(&curve, &mut table, 1e-7)?`.
pub fn c0_bspline_to_array_of_c1_bspline_curve(
    bs: &Handle<GeomBSplineCurve>,
    tab_bs: &mut HArrayOfBSplineCurves,
    tolerance: f64,
) -> Result<(), Failure> {
    c0_bspline_to_array_of_c1_bspline_curve_with_angular(
        bs,
        tab_bs,
        precision::angular(),
        tolerance,
    )
}

/// `GeomConvert::C0BSplineToArrayOfC1BSplineCurve(BS, tabBS, AngularTolerance,
/// tolerance)`: the pieces of `bs` between its knots of multiplicity Degree,
/// concatenated by ConcatC1 (closed when `bs` is closed with parallel end tangents);
/// a curve without such knots is its own single piece (the handle itself, not a copy);
/// `geom_convert::c0_bspline_to_array_of_c1_bspline_curve_with_angular(&curve, &mut table, 1e-12, 1e-7)?`.
pub fn c0_bspline_to_array_of_c1_bspline_curve_with_angular(
    bs: &Handle<GeomBSplineCurve>,
    tab_bs: &mut HArrayOfBSplineCurves,
    angular_tolerance: f64,
    tolerance: f64,
) -> Result<(), Failure> {
    let curve: &GeomBSplineCurve = live_bspline(bs)?;
    let bs_mults: Array1<'static, i32> = curve.multiplicities();
    let bs_knots: Array1<'static, f64> = curve.knots();
    // Give the number of C1 curves.
    let mut nbcurve_c1: i32 = 1;
    for i in curve.first_u_knot_index()?..=(curve.last_u_knot_index()? - 1) {
        if cloned_at(&bs_mults, i)? == curve.degree() {
            nbcurve_c1 += 1;
        }
    }
    if nbcurve_c1 <= 1 {
        *tab_bs = new_curve_table(1)?;
        let table = tab_bs
            .get()
            .ok_or_else(|| failure("Standard_NullObject", "null table"))?;
        return table
            .change_array1()
            .set_value(0, bs.clone())
            .map_err(|e| array_failure(e, "tabBS"));
    }
    let mut array_of_curves = c1_pieces(curve, &bs_mults, &bs_knots, nbcurve_c1)?;
    let mut array_of_toler: Array1<'static, f64> = zero_based(nbcurve_c1 - 1)?;
    for i in 0..=(nbcurve_c1 - 2) {
        // Filling of the array of tolerances with the variable tolerance.
        stored(&mut array_of_toler, i, tolerance)?;
    }
    let mut array_of_indices: Handle<HArray1<i32>> = Handle::default();
    let (_, v1): (_, Vector3) = curve.d1(curve.first_parameter())?;
    let (_, v2): (_, Vector3) = curve.d1(curve.last_parameter())?;
    // Check if the B-spline is closed G1.
    let mut closed_flag: bool = curve.is_closed()? && v1.is_parallel(v2, angular_tolerance)?;
    let outputs = ConcatOutputs {
        indices: &mut array_of_indices,
        concatenated: tab_bs,
    };
    concat_c1_with_angular(
        &mut array_of_curves,
        &array_of_toler,
        outputs,
        &mut closed_flag,
        tolerance,
        angular_tolerance,
    )
}

// The curves between consecutive knots of multiplicity Degree (segmented copies).
fn c1_pieces(
    curve: &GeomBSplineCurve,
    bs_mults: &Array1<'static, i32>,
    bs_knots: &Array1<'static, f64>,
    nbcurve_c1: i32,
) -> Result<Array1<'static, Handle<GeomBSplineCurve>>, Failure> {
    let mut array_of_curves: Array1<'static, Handle<GeomBSplineCurve>> = zero_based(nbcurve_c1)?;
    let mut u2: f64 = curve.first_parameter();
    let mut j: i32 = curve.first_u_knot_index()? + 1;
    let last: i32 = curve.last_u_knot_index()?;
    for i in 0..nbcurve_c1 {
        // Filling of the array of curves with the curves C1 segmented.
        let u1: f64 = u2;
        while cloned_at(bs_mults, j)? < curve.degree() && j < last {
            j += 1;
        }
        u2 = cloned_at(bs_knots, j)?;
        j += 1;
        let bsbis: Handle<GeomBSplineCurve> = copy_bspline(curve)?;
        live_bspline(&bsbis)?.segment(u1, u2, precision::p_confusion())?;
        stored(&mut array_of_curves, i, bsbis)?;
    }
    Ok(array_of_curves)
}
