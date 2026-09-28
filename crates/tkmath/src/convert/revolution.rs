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
// Rust translation of the ComputePoles helpers and knot loops of Convert_CylinderToBSplineSurface.cxx, Convert_ConeToBSplineSurface.cxx, Convert_SphereToBSplineSurface.cxx and Convert_TorusToBSplineSurface.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The surfaces of revolution share one construction: a meridian profile of
//! `(x, z)` pairs swept around Z by rational quadratic spans of at most 150
//! degrees (`PI / 1.2`), then moved into the surface frame.
use crate::bvh::BvhScalar;
use crate::gp::Point3;
use crate::plib::access::{array_cell, grid_cell};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use std::f64::consts::PI;

/// Number of spans of at most 150 degrees and their half opening for an angular
/// range; `spans(PI) == (2, PI / 4.0)`.
pub(super) fn spans(delta: f64) -> (i32, f64) {
    // (int)std::trunc(...) is cvttsd2si on the oracle: INT_MIN when out of range.
    let count: i32 = (1.2 * delta / PI).trunc().truncate_to_int().wrapping_add(1);
    (count, delta / f64::from(count.wrapping_mul(2)))
}

/// Meridian `(x, z)` at the span ends and middles of `[v1, v1 + delta]`, in a
/// fixed buffer of `capacity` entries (the native `std::array`); the middle points
/// are the tangent intersections, divided by the half-opening cosine;
/// `arc_profile(v1, v2, 5, |a| (r * a.cos(), r * a.sin()), |a, c| (r * a.cos() / c, r * a.sin() / c))?`.
pub(super) fn arc_profile(
    v1: f64,
    v2: f64,
    capacity: usize,
    end: impl Fn(f64) -> (f64, f64),
    middle: impl Fn(f64, f64) -> (f64, f64),
) -> Result<Vec<(f64, f64)>, Failure> {
    let (nb_v_spans, alfa_v) = spans(v2 - v1);
    let last: i64 = 2 * i64::from(nb_v_spans);
    if last >= capacity as i64 {
        // The native loop writes x[last] and z[last] past its std::array.
        return Err(Failure::out_of_range(&format!(
            "ComputePoles: {} meridian poles exceed the {capacity}-entry buffer",
            last + 1
        )));
    }
    let mut profile: Vec<(f64, f64)> = vec![end(v1)];
    let mut v_start: f64 = v1;
    for _ in 1..=nb_v_spans {
        profile.push(middle(v_start + alfa_v, alfa_v.cos()));
        profile.push(end(v_start + 2.0 * alfa_v));
        v_start += 2.0 * alfa_v;
    }
    Ok(profile)
}

/// Sweeps the first `count` profile points around Z over `[u1, u2]` into pole
/// rows 1, 2, ... (the U loops of every ComputePoles); `revolve(&profile, 3, 0.0, PI, &mut poles)?`.
pub(super) fn revolve(
    profile: &[(f64, f64)],
    count: i32,
    u1: f64,
    u2: f64,
    poles: &mut Array2<'_, Point3>,
) -> Result<(), Failure> {
    let (nb_u_spans, alfa_u) = spans(u2 - u1);
    let mut u_start: f64 = u1;
    write_row(poles, 1, profile, count, |x| {
        (x * u_start.cos(), x * u_start.sin())
    })?;
    for i in 1..=nb_u_spans {
        let (middle, end) = (u_start + alfa_u, u_start + 2.0 * alfa_u);
        write_row(poles, 2 * i, profile, count, |x| {
            (
                x * middle.cos() / alfa_u.cos(),
                x * middle.sin() / alfa_u.cos(),
            )
        })?;
        write_row(poles, 2 * i + 1, profile, count, |x| {
            (x * end.cos(), x * end.sin())
        })?;
        u_start += 2.0 * alfa_u;
    }
    Ok(())
}

// Row `row` of poles (x * cos, x * sin, z) for the first `count` profile points.
fn write_row(
    poles: &mut Array2<'_, Point3>,
    row: i32,
    profile: &[(f64, f64)],
    count: i32,
    rotate: impl Fn(f64) -> (f64, f64),
) -> Result<(), Failure> {
    for j in 0..count.max(0) {
        let (x, z) = profile[j as usize];
        let (px, py) = rotate(x);
        *grid_cell(poles, row, j + 1)? = Point3::new(px, py, z);
    }
    Ok(())
}

/// Knots `start + (i - 1) * 2 * alfa` of multiplicity 2, the end ones raised to 3
/// (the trimmed knot loops); `trimmed_knots(&mut knots, &mut mults, 0.0, 3, PI / 4.0)?`.
pub(super) fn trimmed_knots(
    knots: &mut Array1<'_, f64>,
    mults: &mut Array1<'_, i32>,
    start: f64,
    count: i32,
    alfa: f64,
) -> Result<(), Failure> {
    for i in 1..=count {
        *array_cell(knots, i)? = start + f64::from((i - 1) * 2) * alfa;
        *array_cell(mults, i)? = 2;
    }
    *array_cell(mults, 1)? += 1;
    *array_cell(mults, count)? += 1;
    Ok(())
}

/// Knots `(i - 1) * 2 PI / 3` of multiplicity 2 of the periodic directions;
/// `periodic_knots(&mut knots, &mut mults)?`.
pub(super) fn periodic_knots(
    knots: &mut Array1<'_, f64>,
    mults: &mut Array1<'_, i32>,
) -> Result<(), Failure> {
    for i in 1..=4 {
        *array_cell(knots, i)? = f64::from(i - 1) * 2. * PI / 3.;
        *array_cell(mults, i)? = 2;
    }
    Ok(())
}
