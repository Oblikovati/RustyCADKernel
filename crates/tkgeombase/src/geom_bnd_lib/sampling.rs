// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of GeomBndLib_SamplingHelpers.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_SamplingHelpers`: the sample counts of the sampling and optimal boxes.
//! The upstream tests include this private header, so the functions are public.
use super::live;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_to_int;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};

// The shrink rule of a spline count over a partial range: `du < .9` scales the count,
// with at least 5 samples. `RealToInt(du * N) + 1` wraps like the native int addition.
fn shrink_to_range(count: i32, fraction: f64) -> i32 {
    if fraction < 0.9 {
        let scaled: i32 = real_to_int(fraction * f64::from(count)).wrapping_add(1);
        return scaled.max(5);
    }
    count
}

// `2 * (Degree + 1) * (NbKnots - 1)`, wrapping like the native int arithmetic.
fn spline_count(degree: i32, nb_knots: i32) -> i32 {
    2i32.wrapping_mul(degree.wrapping_add(1))
        .wrapping_mul(nb_knots.wrapping_sub(1))
}

/// `ComputeNbSamples(Adaptor3d_Curve, UMin, UMax)`: 2 per pole for a Bezier curve,
/// 2 per degree+1 per span for a B-spline (both shrunk on a range below 0.9 of the
/// parameter span), 33 otherwise, capped at 500; `compute_nb_samples(&adaptor, 0.0, 1.0)?`.
pub fn compute_nb_samples(
    curve: &dyn Adaptor3dCurve,
    u_min: f64,
    u_max: f64,
) -> Result<i32, Failure> {
    let count: i32 = match curve.get_type()? {
        CurveType::BezierCurve => shrink_to_range(2 * curve.nb_poles()?, u_max - u_min),
        CurveType::BSplineCurve => {
            let handle = curve.bspline()?;
            let bspline = live(&handle, "Adaptor3d_Curve::BSpline")?;
            let count: i32 = spline_count(bspline.degree(), bspline.nb_knots());
            let span: f64 = bspline.last_parameter() - bspline.first_parameter();
            shrink_to_range(count, (u_max - u_min) / span)
        }
        _ => 33,
    };
    Ok(count.min(500))
}

/// `ComputeNbSamples2d(Adaptor2d_Curve2d, UMin, UMax)`: the 2D rule, 17 samples for
/// other curves and a cap of 23; `compute_nb_samples_2d(&adaptor, 0.0, 1.0)?`.
pub fn compute_nb_samples_2d(
    curve: &dyn Adaptor2dCurve2d,
    u_min: f64,
    u_max: f64,
) -> Result<i32, Failure> {
    let count: i32 = match curve.get_type()? {
        CurveType::BezierCurve => shrink_to_range(2 * curve.nb_poles()?, u_max - u_min),
        CurveType::BSplineCurve => {
            let handle = curve.bspline()?;
            let bspline = live(&handle, "Adaptor2d_Curve2d::BSpline")?;
            let count: i32 = spline_count(bspline.degree(), bspline.nb_knots());
            let span: f64 = bspline.last_parameter() - bspline.first_parameter();
            shrink_to_range(count, (u_max - u_min) / span)
        }
        _ => 17,
    };
    Ok(count.min(23))
}

// Which surface direction a sample count is computed for.
#[derive(Clone, Copy)]
enum SampleDirection {
    U,
    V,
}

// The count of one direction, shrunk to `[min, max]` when a range is given.
fn surface_samples(
    surface: &dyn Adaptor3dSurface,
    direction: SampleDirection,
    range: Option<(f64, f64)>,
) -> Result<i32, Failure> {
    let count: i32 = match surface.get_type()? {
        SurfaceType::BezierSurface => {
            let poles: i32 = match direction {
                SampleDirection::U => surface.nb_u_poles()?,
                SampleDirection::V => surface.nb_v_poles()?,
            };
            let count: i32 = 2i32.wrapping_mul(poles);
            range.map_or(count, |(min, max)| shrink_to_range(count, max - min))
        }
        SurfaceType::BSplineSurface => bspline_surface_samples(surface, direction, range)?,
        _ => 33,
    };
    Ok(count.min(50))
}

// The B-spline surface count: spans of the knot vector, shrunk against `Bounds`.
fn bspline_surface_samples(
    surface: &dyn Adaptor3dSurface,
    direction: SampleDirection,
    range: Option<(f64, f64)>,
) -> Result<i32, Failure> {
    let handle = surface.bspline()?;
    let bspline = live(&handle, "Adaptor3d_Surface::BSpline")?;
    let (u1, u2, v1, v2) = bspline.bounds();
    let (count, span): (i32, f64) = match direction {
        SampleDirection::U => (
            spline_count(bspline.u_degree(), bspline.nb_u_knots()),
            u2 - u1,
        ),
        SampleDirection::V => (
            spline_count(bspline.v_degree(), bspline.nb_v_knots()),
            v2 - v1,
        ),
    };
    Ok(range.map_or(count, |(min, max)| {
        shrink_to_range(count, (max - min) / span)
    }))
}

/// `ComputeNbUSamples(Adaptor3d_Surface, UMin, UMax)`: the U count of a range, capped
/// at 50; `compute_nb_u_samples(&adaptor, 0.0, 1.0)?`.
pub fn compute_nb_u_samples(
    surface: &dyn Adaptor3dSurface,
    u_min: f64,
    u_max: f64,
) -> Result<i32, Failure> {
    surface_samples(surface, SampleDirection::U, Some((u_min, u_max)))
}

/// `ComputeNbVSamples(Adaptor3d_Surface, VMin, VMax)`: the V count of a range, capped
/// at 50; `compute_nb_v_samples(&adaptor, 0.0, 1.0)?`.
pub fn compute_nb_v_samples(
    surface: &dyn Adaptor3dSurface,
    v_min: f64,
    v_max: f64,
) -> Result<i32, Failure> {
    surface_samples(surface, SampleDirection::V, Some((v_min, v_max)))
}

/// `ComputeNbUSamples(Adaptor3d_Surface)`: the full-range U count (no shrinking);
/// `compute_nb_u_samples_full(&adaptor)?`.
pub fn compute_nb_u_samples_full(surface: &dyn Adaptor3dSurface) -> Result<i32, Failure> {
    surface_samples(surface, SampleDirection::U, None)
}

/// `ComputeNbVSamples(Adaptor3d_Surface)`: the full-range V count (no shrinking);
/// `compute_nb_v_samples_full(&adaptor)?`.
pub fn compute_nb_v_samples_full(surface: &dyn Adaptor3dSurface) -> Result<i32, Failure> {
    surface_samples(surface, SampleDirection::V, None)
}
