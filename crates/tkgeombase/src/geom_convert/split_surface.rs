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
// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_1.cxx
// (the four SplitBSplineSurface overloads) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomConvert::SplitBSplineSurface`: a patch of a B-spline surface between knot
//! indices or parameters, in both directions or in one.
use super::{copy_bspline_surface, live_bspline_surface};
use crate::native_failure::failure;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomSurface};

/// `GeomConvert::SplitBSplineSurface(S, FromUK1, ToUK2, FromVK1, ToVK2,
/// SameUOrientation = true, SameVOrientation = true)`: the patch of `surface` between
/// the U knots of indices `from_uk1`, `to_uk2` and the V knots `from_vk1`, `to_vk2`.
///
/// Each direction of a periodic surface keeps its orientation unless the matching
/// `same_*_orientation` is false; any other direction runs from the first index to
/// the second. Standard_DomainError for equal indices or an index outside the knot
/// range; `geom_convert::split_bspline_surface_knots(&s, (1, 3), (1, 2), (true, true))?`.
pub fn split_bspline_surface_knots(
    surface: &Handle<GeomBSplineSurface>,
    (from_uk1, to_uk2): (i32, i32),
    (from_vk1, to_vk2): (i32, i32),
    (same_u_orientation, same_v_orientation): (bool, bool),
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let s: &GeomBSplineSurface = live_bspline_surface(surface)?;
    let (first_u, first_v) = (s.first_u_knot_index()?, s.first_v_knot_index()?);
    let (last_u, last_v) = (s.last_u_knot_index()?, s.last_v_knot_index()?);
    if from_uk1 == to_uk2 || from_vk1 == to_vk2 {
        return Err(domain_error(&format!(
            "knots {from_uk1}..{to_uk2} x {from_vk1}..{to_vk2}; expected distinct"
        )));
    }
    let (first_uk, last_uk) = (from_uk1.min(to_uk2), from_uk1.max(to_uk2));
    let (first_vk, last_vk) = (from_vk1.min(to_vk2), from_vk1.max(to_vk2));
    if first_uk < first_u || last_uk > last_u || first_vk < first_v || last_vk > last_v {
        return Err(domain_error(&format!(
            "knots {from_uk1}..{to_uk2} x {from_vk1}..{to_vk2}; expected within {first_u}..{last_u} x {first_v}..{last_v}"
        )));
    }
    let s1: Handle<GeomBSplineSurface> = copy_bspline_surface(s)?;
    let copy: &GeomBSplineSurface = live_bspline_surface(&s1)?;
    let u = (copy.u_knot(first_uk)?, copy.u_knot(last_uk)?);
    let v = (copy.v_knot(first_vk)?, copy.v_knot(last_vk)?);
    segment(copy, u, v)?;
    orient_u(
        copy,
        s.is_u_periodic(),
        same_u_orientation,
        from_uk1 > to_uk2,
    )?;
    orient_v(
        copy,
        s.is_v_periodic(),
        same_v_orientation,
        from_vk1 > to_vk2,
    )?;
    Ok(s1)
}

/// `GeomConvert::SplitBSplineSurface(S, FromK1, ToK2, USplit, SameOrientation =
/// true)`: the patch between the knots `from_k1` and `to_k2` of the U direction
/// (`u_split`) or of the V direction, the whole other direction kept.
///
/// Standard_DomainError for equal indices or an index outside the knot range;
/// `geom_convert::split_bspline_surface_one_direction_knots(&s, 1, 3, true, true)?`.
pub fn split_bspline_surface_one_direction_knots(
    surface: &Handle<GeomBSplineSurface>,
    from_k1: i32,
    to_k2: i32,
    u_split: bool,
    same_orientation: bool,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let s: &GeomBSplineSurface = live_bspline_surface(surface)?;
    if from_k1 == to_k2 {
        return Err(domain_error(&format!(
            "FromK1 = ToK2 = {from_k1}; expected distinct knots"
        )));
    }
    let s1: Handle<GeomBSplineSurface> = copy_bspline_surface(s)?;
    let copy: &GeomBSplineSurface = live_bspline_surface(&s1)?;
    let (first_k, last_k) = (from_k1.min(to_k2), from_k1.max(to_k2));
    let (first, last) = if u_split {
        (s.first_u_knot_index()?, s.last_u_knot_index()?)
    } else {
        (s.first_v_knot_index()?, s.last_v_knot_index()?)
    };
    if first_k < first || last_k > last {
        return Err(domain_error(&format!(
            "knots {from_k1}..{to_k2}; expected within {first}..{last}"
        )));
    }
    let whole_u = (
        copy.u_knot(copy.first_u_knot_index()?)?,
        copy.u_knot(copy.last_u_knot_index()?)?,
    );
    let whole_v = (
        copy.v_knot(copy.first_v_knot_index()?)?,
        copy.v_knot(copy.last_v_knot_index()?)?,
    );
    if u_split {
        segment(copy, (copy.u_knot(first_k)?, copy.u_knot(last_k)?), whole_v)?;
        orient_u(copy, s.is_u_periodic(), same_orientation, from_k1 > to_k2)?;
    } else {
        segment(copy, whole_u, (copy.v_knot(first_k)?, copy.v_knot(last_k)?))?;
        orient_v(copy, s.is_v_periodic(), same_orientation, from_k1 > to_k2)?;
    }
    Ok(s1)
}

/// `GeomConvert::SplitBSplineSurface(S, FromU1, ToU2, FromV1, ToV2,
/// ParametricTolerance, SameUOrientation = true, SameVOrientation = true)`: the patch
/// between two U and two V parameters.
///
/// The native member ignores `parametric_tolerance` and checks no bound: the
/// B-spline `Segment` raises for parameters it rejects;
/// `geom_convert::split_bspline_surface(&s, (0.2, 0.8), (0.1, 0.9), 1e-9, (true, true))?`.
pub fn split_bspline_surface(
    surface: &Handle<GeomBSplineSurface>,
    (from_u1, to_u2): (f64, f64),
    (from_v1, to_v2): (f64, f64),
    _parametric_tolerance: f64,
    (same_u_orientation, same_v_orientation): (bool, bool),
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let s: &GeomBSplineSurface = live_bspline_surface(surface)?;
    let new_surface: Handle<GeomBSplineSurface> = copy_bspline_surface(s)?;
    let copy: &GeomBSplineSurface = live_bspline_surface(&new_surface)?;
    let u = (std_min(from_u1, to_u2), std_max(from_u1, to_u2));
    let v = (std_min(from_v1, to_v2), std_max(from_v1, to_v2));
    segment(copy, u, v)?;
    orient_u(copy, s.is_u_periodic(), same_u_orientation, from_u1 > to_u2)?;
    orient_v(copy, s.is_v_periodic(), same_v_orientation, from_v1 > to_v2)?;
    Ok(new_surface)
}

/// `GeomConvert::SplitBSplineSurface(S, FromParam1, ToParam2, USplit,
/// ParametricTolerance, SameOrientation = true)`: the patch between two parameters of
/// the U direction (`u_split`) or of the V direction, the whole other direction kept.
///
/// Standard_DomainError when `|FromParam1 - ToParam2| <= |ParametricTolerance|`.
/// Natively the V split still orients the result in U (it tests IsUPeriodic and
/// calls UReverse), and the port does the same;
/// `geom_convert::split_bspline_surface_one_direction(&s, 0.2, 0.8, true, 1e-9, true)?`.
pub fn split_bspline_surface_one_direction(
    surface: &Handle<GeomBSplineSurface>,
    from_param1: f64,
    to_param2: f64,
    u_split: bool,
    parametric_tolerance: f64,
    same_orientation: bool,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    if (from_param1 - to_param2).abs() <= parametric_tolerance.abs() {
        return Err(domain_error(&format!(
            "parameters {from_param1}, {to_param2}; expected apart by more than {parametric_tolerance}"
        )));
    }
    let s: &GeomBSplineSurface = live_bspline_surface(surface)?;
    let new_surface: Handle<GeomBSplineSurface> = copy_bspline_surface(s)?;
    let copy: &GeomBSplineSurface = live_bspline_surface(&new_surface)?;
    let range = (
        std_min(from_param1, to_param2),
        std_max(from_param1, to_param2),
    );
    if u_split {
        let whole_v = (
            s.v_knot(s.first_v_knot_index()?)?,
            s.v_knot(s.last_v_knot_index()?)?,
        );
        segment(copy, range, whole_v)?;
    } else {
        let whole_u = (
            s.u_knot(s.first_u_knot_index()?)?,
            s.u_knot(s.last_u_knot_index()?)?,
        );
        segment(copy, whole_u, range)?;
    }
    // Both branches orient in U, as natively.
    orient_u(
        copy,
        s.is_u_periodic(),
        same_orientation,
        from_param1 > to_param2,
    )?;
    Ok(new_surface)
}

fn domain_error(detail: &str) -> Failure {
    failure(
        "Standard_DomainError",
        &format!("GeomConvert::SplitBSplineSurface: {detail}"),
    )
}

// Segment with the default parametric tolerances.
fn segment(surface: &GeomBSplineSurface, u: (f64, f64), v: (f64, f64)) -> Result<(), Failure> {
    surface.segment(u, v, (precision::p_confusion(), precision::p_confusion()))
}

// UReverse for a periodic direction asked in the other orientation, or for bounds
// given in decreasing order in a non-periodic direction.
fn orient_u(
    surface: &GeomBSplineSurface,
    periodic: bool,
    same: bool,
    decreasing: bool,
) -> Result<(), Failure> {
    if if periodic { !same } else { decreasing } {
        surface.u_reverse()?;
    }
    Ok(())
}

// The VReverse twin of orient_u.
fn orient_v(
    surface: &GeomBSplineSurface,
    periodic: bool,
    same: bool,
    decreasing: bool,
) -> Result<(), Failure> {
    if if periodic { !same } else { decreasing } {
        surface.v_reverse()?;
    }
    Ok(())
}
