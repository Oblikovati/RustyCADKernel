// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of SplitDS, BuildCurveSplits, SplitOnDirection and FindSplitPoint
// (ProjLib_CompProjectedCurve.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The curve parameters where the projection crosses a periodic bound of the
//! surface: the 3D curve of the bound is intersected with the curve (one solution
//! at a time, recursively on both sides), and a solution counts when the point
//! projects on the bound.
use super::derivatives::Pair;
use crate::extrema::{ExtCC, ExtFlag, ExtPS};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkmath::gp::{Direction2, Point2};

/// `BuildCurveSplits(Curve, Surface, TolU, TolV, Splits)`: the sorted split
/// parameters (`surface` is the handle of `pair.surface`);
/// `build_curve_splits(pair, &surface, (1e-7, 1e-7))?`.
pub(super) fn build_curve_splits(
    pair: Pair<'_>,
    surface: &Handle<dyn Adaptor3dSurface>,
    (tol_u, tol_v): (f64, f64),
) -> Result<Vec<f64>, Failure> {
    let s: &dyn Adaptor3dSurface = pair.surface;
    let mut ext_ps: ExtPS<'_> = ExtPS::new();
    let u = (s.first_u_parameter()?, s.last_u_parameter()?);
    let v = (s.first_v_parameter()?, s.last_v_parameter()?);
    ext_ps.initialize(s, u, v, (tol_u, tol_v))?;
    ext_ps.set_flag(ExtFlag::Min);
    let mut splits: Vec<f64> = Vec::new();
    if s.is_u_periodic()? {
        split_on_direction(pair, surface, 0, &mut splits, &mut ext_ps)?;
    }
    if s.is_v_periodic()? {
        split_on_direction(pair, surface, 1, &mut splits, &mut ext_ps)?;
    }
    splits.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    Ok(splits)
}

/// `SplitOnDirection(DS)`: the bound of direction `dir` as a curve on the surface,
/// intersected with the whole curve range.
fn split_on_direction<'a>(
    pair: Pair<'a>,
    surface: &Handle<dyn Adaptor3dSurface>,
    dir: i32,
    splits: &mut Vec<f64>,
    ext_ps: &mut ExtPS<'a>,
) -> Result<(), Failure> {
    let s: &dyn Adaptor3dSurface = pair.surface;
    let start: Point2 = Point2::new(s.first_u_parameter()?, s.first_v_parameter()?);
    let direction: Direction2 = Direction2::new(f64::from(dir), f64::from(i32::from(dir == 0)))?;
    let (per_min, per_max) = if dir == 0 {
        (s.first_u_parameter()?, s.last_u_parameter()?)
    } else {
        (s.first_v_parameter()?, s.last_v_parameter()?)
    };
    let last_2d: f64 = if dir != 0 {
        s.last_u_parameter()? - s.first_u_parameter()?
    } else {
        s.last_v_parameter()? - s.first_v_parameter()?
    };
    let line: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(start, direction)).cast();
    let bound2d: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_range(&line, 0., last_2d)?).cast();
    let bound = Adaptor3dCurveOnSurface::with_curve(&bound2d, surface)?;
    let (first, last) = (pair.curve.first_parameter()?, pair.curve.last_parameter()?);
    find_split_point(
        pair,
        (dir, per_min, per_max, last_2d),
        &bound,
        splits,
        ext_ps,
        (first, last),
    )
}

/// `FindSplitPoint(DS, MinParam, MaxParam)`: the single extremum of the bound
/// curve and the curve in `[min, max]`; a point that projects on the bound is a
/// split, and both sides are searched again, 1% of the ranges away.
fn find_split_point<'a>(
    pair: Pair<'a>,
    (dir, per_min, per_max, last_2d): (i32, f64, f64, f64),
    bound: &Adaptor3dCurveOnSurface,
    splits: &mut Vec<f64>,
    ext_ps: &mut ExtPS<'a>,
    (min_param, max_param): (f64, f64),
) -> Result<(), Failure> {
    // A fresh extrema at each level of the recursion.
    let mut ext_cc: ExtCC<'_> = ExtCC::new(1.0e-10, 1.0e-10);
    ext_cc.set_curve(1, bound)?;
    ext_cc.set_curve(2, pair.curve)?;
    // Search only one solution since multiple invocations are needed.
    ext_cc.set_single_solution_flag(true);
    ext_cc.set_range(1, 0., last_2d)?;
    ext_cc.set_range(2, min_param, max_param)?;
    ext_cc.perform()?;
    if !ext_cc.is_done() || ext_cc.is_parallel()? {
        return Ok(());
    }
    for an_idx in 1..=ext_cc.nb_ext()? {
        let (_, on_curve) = ext_cc.points(an_idx)?;
        ext_ps.perform(*on_curve.value())?;
        if !ext_ps.is_done() {
            return Ok(());
        }
        let proj_param: f64 = projected_param(ext_ps, dir)?;
        let pconf: f64 = precision::p_confusion();
        if (proj_param - per_min).abs() >= pconf && (proj_param - per_max).abs() >= pconf {
            continue;
        }
        let param: f64 = on_curve.parameter();
        let (cf, cl) = (pair.curve.first_parameter()?, pair.curve.last_parameter()?);
        if param > cf + pconf && param < cl - pconf {
            // Only inner points.
            splits.push(param);
        }
        let delta: f64 = (max_param - min_param + cl - cf) * 0.01;
        let data = (dir, per_min, per_max, last_2d);
        if param - delta > min_param + pconf {
            find_split_point(
                pair,
                data,
                bound,
                splits,
                ext_ps,
                (min_param, param - delta),
            )?;
        }
        if param + delta < max_param - pconf {
            find_split_point(
                pair,
                data,
                bound,
                splits,
                ext_ps,
                (param + delta, max_param),
            )?;
        }
    }
    Ok(())
}

// The U (or V) of the nearest projection, avoiding false positives by the minimal
// Euclidean distance; `Point(-1)` natively raises without extrema.
fn projected_param(ext_ps: &ExtPS<'_>, dir: i32) -> Result<f64, Failure> {
    let mut min_idx: i32 = -1;
    let mut min_sq_dist: f64 = f64::MAX;
    for p_idx in 1..=ext_ps.nb_ext()? {
        let sq: f64 = ext_ps.square_distance(p_idx)?;
        if sq < min_sq_dist {
            (min_sq_dist, min_idx) = (sq, p_idx);
        }
    }
    let (u, v) = ext_ps.point(min_idx)?.parameter();
    Ok(if dir != 0 { v } else { u })
}
