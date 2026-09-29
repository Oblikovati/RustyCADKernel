// Created by: Nikolai BUKHALOV
// Copyright (c) 2015 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib_CheckCurveOnSurface.cxx
// (FillSubIntervals) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FillSubIntervals`: the sub-intervals between the merged knots of the 3D curve
//! and the pcurve, and the particle count of their degrees.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::geom_abs::CurveType;

// At most this many knots of a B-spline are used, otherwise a uniform sampling.
const A_MAX_KNOTS: usize = 101;

/// The sub-intervals of `[first, last]` (the count, and their bounds when asked) and
/// the number of particles (3, or the higher B-spline degree).
pub(super) struct SubIntervals {
    pub count: i32,
    pub nb_particles: i32,
    pub bounds: Vec<f64>,
}

/// `FillSubIntervals(theCurve3d, theCurve2d, theFirst, theLast, theNbParticles,
/// theSubIntervals)`: the knots of each B-spline (a uniform sampling of 101 values
/// when it has more knots), or `[first, last]`, merged within
/// Precision::PConfusion(); the count is 0 when the merge fails (the native catch).
pub(super) fn fill_sub_intervals(
    curve3d: &dyn Adaptor3dCurve,
    curve2d: &Handle<dyn Adaptor2dCurve2d>,
    first: f64,
    last: f64,
) -> Result<SubIntervals, Failure> {
    let mut nb_particles: i32 = 3;
    let (knots_3d, degree_3d) = if curve3d.get_type()? == CurveType::BSplineCurve {
        let bs = curve3d.bspline()?;
        let bs = bs.get().ok_or_else(|| {
            crate::native_failure::failure("Standard_NullObject", "FillSubIntervals: null B-spline")
        })?;
        (
            spline_knots(
                &bs.knots(),
                (bs.first_u_knot_index()?, bs.last_u_knot_index()?),
                first,
                last,
            ),
            Some(bs.degree()),
        )
    } else {
        (vec![first, last], None)
    };
    let c2d = curve2d.get().ok_or_else(|| {
        crate::native_failure::failure("Standard_NullObject", "FillSubIntervals: null pcurve")
    })?;
    let (knots_2d, degree_2d) = if c2d.get_type()? == CurveType::BSplineCurve {
        let bs = c2d.bspline()?;
        let bs = bs.get().ok_or_else(|| {
            crate::native_failure::failure("Standard_NullObject", "FillSubIntervals: null B-spline")
        })?;
        (
            spline_knots(
                &bs.knots(),
                (bs.first_u_knot_index()?, bs.last_u_knot_index()?),
                first,
                last,
            ),
            Some(bs.degree()),
        )
    } else {
        (vec![first, last], None)
    };
    let bounds: Vec<f64> = merged_bounds(&knots_3d, &knots_2d, first, last);
    if let Some(degree) = degree_3d {
        nb_particles = nb_particles.max(degree);
    }
    if let Some(degree) = degree_2d {
        nb_particles = nb_particles.max(degree);
    }
    Ok(SubIntervals {
        count: bounds.len() as i32 - 1,
        nb_particles,
        bounds,
    })
}

// The knots of a B-spline, or 101 uniform values of [first, last] when it has more
// than 101 knots between its first and last knot indices (the native isTrimmed flags
// are always false, so only this count matters).
fn spline_knots(
    knots: &rustycad_tkernel::ncollection::Array1<'static, f64>,
    (first_index, last_index): (i32, i32),
    first: f64,
    last: f64,
) -> Vec<f64> {
    let all: Vec<f64> = knots.iter().copied().collect();
    if all.len() <= A_MAX_KNOTS {
        return all;
    }
    let knot_count: i32 = last_index - first_index + 1;
    if knot_count as usize <= A_MAX_KNOTS {
        return all;
    }
    let mut values: Vec<f64> = vec![0.0; A_MAX_KNOTS];
    values[0] = first;
    values[A_MAX_KNOTS - 1] = last;
    let dt: f64 = (last - first) / (A_MAX_KNOTS - 1) as f64;
    let mut t: f64 = first + dt;
    for value in values.iter_mut().take(A_MAX_KNOTS - 1).skip(1) {
        *value = t;
        t += dt;
    }
    values
}

// The merge of the two sorted knot lists: `first`, every value strictly inside
// (first, last), a pair within Precision::PConfusion() counted once, then `last`.
fn merged_bounds(knots_3d: &[f64], knots_2d: &[f64], first: f64, last: f64) -> Vec<f64> {
    let mut bounds: Vec<f64> = vec![first];
    let (mut index_3d, mut index_2d): (usize, usize) = (0, 0);
    while index_3d < knots_3d.len() && index_2d < knots_2d.len() {
        let (val_3d, val_2d): (f64, f64) = (knots_3d[index_3d], knots_2d[index_2d]);
        let delta: f64 = val_3d - val_2d;
        if delta < precision::p_confusion() {
            // aVal3D <= aVal2D
            if val_3d > first && val_3d < last {
                bounds.push(val_3d);
            }
            index_3d += 1;
            if -delta < precision::p_confusion() {
                // aVal3D == aVal2D
                index_2d += 1;
            }
        } else {
            // aVal2D < aVal3D
            if val_2d > first && val_2d < last {
                bounds.push(val_2d);
            }
            index_2d += 1;
        }
    }
    bounds.push(last);
    bounds
}
