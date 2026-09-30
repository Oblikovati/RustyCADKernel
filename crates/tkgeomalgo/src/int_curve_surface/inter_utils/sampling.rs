// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of DoSurface and DoNewBounds (IntCurveSurface_InterUtils.pxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! A 50 x 50 sampling of a surface patch and the parameter box it gives around the
//! corners of a bounding box (`DoSurface`, `DoNewBounds`; the private members of
//! IntCurveSurface_HInter call them).
use crate::int_curve_surface::tools::InterSurfaceTool;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point3;

/// The sample count of each direction.
const SAMPLES: i32 = 50;

/// The parameter of sample `i` (0-based): the first bound, the last one for the
/// last sample, the first bound plus `i` steps otherwise.
fn sample_parameter(i: i32, first: f64, last: f64, step: f64) -> f64 {
    if i == 0 {
        first
    } else if i == SAMPLES - 1 {
        last
    } else {
        first + step * f64::from(i)
    }
}

/// `DoSurface`: the 50 x 50 points of the patch (`pnts(iu, iv)` from 1, which must
/// hold them), each added to `bnd`; returns the gap, the larger of the parametric
/// resolutions of a sample step; `do_surface::<HSurfaceTool>(&s, [0.0, 1.0, 0.0, 1.0], &mut pnts, &mut bnd)?`.
pub fn do_surface<S: InterSurfaceTool>(
    surface: &S::Surface,
    [u0, u1, v0, v1]: [f64; 4],
    pnts: &mut Array2<'_, Point3>,
    bnd: &mut BoundingBox3,
) -> Result<f64, Failure> {
    let (du, dv) = ((u1 - u0) / 50.0, (v1 - v0) / 50.0);
    for iu in 0..SAMPLES {
        let u: f64 = sample_parameter(iu, u0, u1, du);
        for iv in 0..SAMPLES {
            let v: f64 = sample_parameter(iv, v0, v1, dv);
            let p: Point3 = S::d0(surface, u, v)?;
            bnd.add_point(p);
            pnts.set_value(iu + 1, iv + 1, p)
                .map_err(|e| e.into_failure("NCollection_Array2::SetValue"))?;
        }
    }
    let ures: f64 = S::u_resolution(surface, du)?;
    let vres: f64 = S::v_resolution(surface, dv)?;
    Ok(std_max(ures, vres))
}

/// The grid indices `(iu, iv)` of the sample nearest to `point` (0, 0 when every
/// distance is NaN or beyond 1e100); Standard_OutOfRange when `pnts` lacks a sample.
fn nearest_sample(pnts: &Array2<'_, Point3>, point: Point3) -> Result<(i32, i32), Failure> {
    let mut dist_min: f64 = 1.0e100;
    let (mut diu, mut div) = (0, 0);
    for iu in 1..=SAMPLES {
        for iv in 1..=SAMPLES {
            let sample: &Point3 = pnts
                .value(iu, iv)
                .map_err(|e| e.into_failure("NCollection_Array2::Value"))?;
            let dist: f64 = sample.square_distance(&point);
            if dist < dist_min {
                (dist_min, diu, div) = (dist, iu, iv);
            }
        }
    }
    Ok((diu, div))
}

/// `DoNewBounds`: the patch bounds narrowed, in each direction that is neither
/// closed nor periodic, to the samples nearest to the eight corners `(x(i), y(j),
/// z(k))` widened by 1.5 steps and clipped to the patch; returns `[u0, u1, v0, v1]`;
/// `do_new_bounds::<HSurfaceTool>(&s, [0.0, 1.0, 0.0, 1.0], &pnts, [&x, &y, &z])?`.
pub fn do_new_bounds<S: InterSurfaceTool>(
    surface: &S::Surface,
    [u0, u1, v0, v1]: [f64; 4],
    pnts: &Array2<'_, Point3>,
    [x, y, z]: [&Array1<'_, f64>; 3],
) -> Result<[f64; 4], Failure> {
    let mut bounds: [f64; 4] = [u0, u1, v0, v1];
    let check_u: bool = !(S::is_u_closed(surface)? || S::is_u_periodic(surface)?);
    let check_v: bool = !(S::is_v_closed(surface)? || S::is_v_periodic(surface)?);
    let (mut iu_min, mut iv_min, mut iu_max, mut iv_max) = (50, 50, 1, 1);
    for i in 1..=2 {
        for j in 1..=2 {
            for k in 1..=2 {
                let corner = Point3::new(coordinate(x, i)?, coordinate(y, j)?, coordinate(z, k)?);
                let (diu, div) = nearest_sample(pnts, corner)?;
                if diu > 0 {
                    iu_min = iu_min.min(diu);
                    iu_max = iu_max.max(diu);
                }
                if div > 0 {
                    iv_min = iv_min.min(div);
                    iv_max = iv_max.max(div);
                }
            }
        }
    }
    if check_u {
        [bounds[0], bounds[1]] = narrowed([u0, u1], iu_min, iu_max);
    }
    if check_v {
        [bounds[2], bounds[3]] = narrowed([v0, v1], iv_min, iv_max);
    }
    Ok(bounds)
}

/// `values(i)`: Standard_OutOfRange outside the array.
fn coordinate(values: &Array1<'_, f64>, i: i32) -> Result<f64, Failure> {
    values
        .value(i)
        .copied()
        .map_err(|e| e.into_failure("NCollection_Array1::Value"))
}

/// The sample range `[i_min, i_max]` as parameters, ordered, widened by 1.5 steps
/// and clipped to `[first, last]`.
fn narrowed([first, last]: [f64; 2], i_min: i32, i_max: i32) -> [f64; 2] {
    let d: f64 = (last - first) / 50.0;
    let mut smin: f64 = first + d * f64::from(i_min - 1);
    let mut smax: f64 = first + d * f64::from(i_max - 1);
    if smin > smax {
        std::mem::swap(&mut smin, &mut smax);
    }
    smin -= 1.5 * d;
    if smin < first {
        smin = first;
    }
    smax += 1.5 * d;
    if smax > last {
        smax = last;
    }
    [smin, smax]
}
