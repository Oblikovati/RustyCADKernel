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
// Rust translation of FoundationClasses/TKMath/BSplSLib/BSplSLib.cxx (CacheD0, CacheD1, CacheD2) and BSplSLib.lxx (CoefsD0..CoefsD2) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Evaluation of a cached surface patch: a polynomial in the max-degree parameter
//! whose coefficients are rows of (min degree + 1) points.
//!
//! Quirk kept: the cache is read from `Poles(LowerCol, LowerRow)` (and likewise
//! for the weights), with the bounds swapped; they coincide for 1-based grids.
use super::eval::{SurfaceD2, point, vector};
use super::rational::rational_derivative;
use crate::gp::{Point3, Vector3};
use crate::plib::{eval_polynomial, no_derivative_eval_polynomial};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Failure;

/// Degrees and parametrization of a cached patch.
#[derive(Clone, Copy, Debug)]
pub struct SurfaceCacheSpan {
    /// U degree.
    pub u_degree: i32,
    /// V degree.
    pub v_degree: i32,
    /// U parameter of the cache.
    pub u_cache_parameter: f64,
    /// V parameter of the cache.
    pub v_cache_parameter: f64,
    /// U span length.
    pub u_span_length: f64,
    /// V span length.
    pub v_span_length: f64,
}

impl SurfaceCacheSpan {
    /// The span of Bezier coefficients (the CoefsD* shortcuts): degrees
    /// `RowLength-1`, `ColLength-1` on `[0, 1]`; `SurfaceCacheSpan::bezier(&coefficients)`.
    pub fn bezier(poles: &Array2<'_, Point3>) -> Self {
        Self {
            u_degree: poles.nb_columns() as i32 - 1,
            v_degree: poles.nb_rows() as i32 - 1,
            u_cache_parameter: 0.,
            v_cache_parameter: 0.,
            u_span_length: 1.,
            v_span_length: 1.,
        }
    }

    fn u_min(&self) -> bool {
        self.u_degree <= self.v_degree
    }

    fn degrees(&self) -> (i32, i32) {
        if self.u_min() {
            (self.u_degree, self.v_degree)
        } else {
            (self.v_degree, self.u_degree)
        }
    }
}

// The grid values from (LowerCol, LowerRow), as the native `&Grid(LowerCol, LowerRow)`.
fn swapped_start<T: Copy>(
    grid: &Array2<'_, T>,
    flatten: impl Fn(&T, &mut Vec<f64>),
) -> Result<Vec<f64>, Failure> {
    let (row, col) = (grid.lower_col(), grid.lower_row());
    grid.value(row, col)
        .map_err(|error| Failure::out_of_range(&format!("NCollection_Array2: {error}")))?;
    let offset: usize =
        (row - grid.lower_row()) as usize * grid.nb_columns() + (col - grid.lower_col()) as usize;
    let mut values: Vec<f64> = Vec::new();
    for item in &grid.as_slice()[offset..] {
        flatten(item, &mut values);
    }
    Ok(values)
}

fn pole_values(poles: &Array2<'_, Point3>) -> Result<Vec<f64>, Failure> {
    swapped_start(poles, |p, out| {
        out.extend_from_slice(&[p.x(), p.y(), p.z()])
    })
}

fn weight_values(weights: &Array2<'_, f64>) -> Result<Vec<f64>, Failure> {
    swapped_start(weights, |w, out| out.push(*w))
}

/// Point of a cached patch (`BSplSLib::CacheD0`); `CoefsD0` uses
/// [`SurfaceCacheSpan::bezier`]; `bsplslib::cache_d0(0.5, 0.5, &span, &cache, None)?`.
pub fn cache_d0(
    u: f64,
    v: f64,
    span: &SurfaceCacheSpan,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
) -> Result<Point3, Failure> {
    let (min, max) = span.degrees();
    let u_param: f64 = (u - span.u_cache_parameter) / span.u_span_length;
    let v_param: f64 = (v - span.v_cache_parameter) / span.v_span_length;
    let (p_max, p_min) = if span.u_min() {
        (v_param, u_param)
    } else {
        (u_param, v_param)
    };
    let dimension: i32 = 3 * (min + 1);
    let mut local: Vec<f64> = vec![0.0; dimension.max(0) as usize];
    no_derivative_eval_polynomial(
        p_max,
        max,
        dimension,
        max * dimension,
        &pole_values(poles)?,
        &mut local,
    )?;
    let mut result: [f64; 3] = [0.0; 3];
    no_derivative_eval_polynomial(p_min, min, 3, (min << 1) + min, &local, &mut result)?;
    if let Some(weights) = weights {
        let dimension: i32 = min + 1;
        no_derivative_eval_polynomial(
            p_max,
            max,
            dimension,
            max * dimension,
            &weight_values(weights)?,
            &mut local,
        )?;
        let mut inverse: [f64; 1] = [0.0];
        no_derivative_eval_polynomial(p_min, min, 1, min, &local, &mut inverse)?;
        let inverse: f64 = 1.0e0 / inverse[0];
        for value in &mut result {
            *value *= inverse;
        }
    }
    Ok(point(&result))
}

// Normalized parameters and inverse span lengths: ([max, min] params, [min, max] inverses).
fn normalized(u: f64, v: f64, span: &SurfaceCacheSpan) -> ([f64; 2], [f64; 2]) {
    if span.u_min() {
        let (inverse_min, inverse_max) = (1.0e0 / span.u_span_length, 1.0e0 / span.v_span_length);
        let params = [
            (v - span.v_cache_parameter) * inverse_max,
            (u - span.u_cache_parameter) * inverse_min,
        ];
        return (params, [inverse_min, inverse_max]);
    }
    let (inverse_min, inverse_max) = (1.0e0 / span.v_span_length, 1.0e0 / span.u_span_length);
    let params = [
        (u - span.u_cache_parameter) * inverse_max,
        (v - span.v_cache_parameter) * inverse_min,
    ];
    (params, [inverse_min, inverse_max])
}

/// Point and first derivatives of a cached patch (`BSplSLib::CacheD1`); returns
/// `(Point, VecU, VecV)`; `bsplslib::cache_d1(0.5, 0.5, &span, &cache, None)?`.
pub fn cache_d1(
    u: f64,
    v: f64,
    span: &SurfaceCacheSpan,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
) -> Result<(Point3, Vector3, Vector3), Failure> {
    let (min, max) = span.degrees();
    let ([p_max, p_min], [inverse_min, inverse_max]) = normalized(u, v, span);
    let dimension: i32 = 3 * (min + 1);
    let dim: usize = dimension.max(0) as usize;
    let mut local: Vec<f64> = vec![0.0; 2 * dim];
    // [0..6): value and min-derivative of row 0; [6..9): max-derivative; [9..12): mixed.
    let mut lpa: [f64; 12] = [0.0; 12];
    eval_polynomial(p_max, 1, max, dimension, &pole_values(poles)?, &mut local)?;
    eval_polynomial(p_min, 1, min, 3, &local, &mut lpa[..6])?;
    no_derivative_eval_polynomial(
        p_min,
        min,
        3,
        (min << 1) + min,
        &local[dim..],
        &mut lpa[6..9],
    )?;
    if let Some(weights) = weights {
        let wdim: i32 = min + 1;
        let mut lw: [f64; 4] = [0.0; 4];
        eval_polynomial(p_max, 1, max, wdim, &weight_values(weights)?, &mut local)?;
        eval_polynomial(p_min, 1, min, 1, &local, &mut lw[..2])?;
        no_derivative_eval_polynomial(p_min, min, 1, min, &local[wdim as usize..], &mut lw[2..3])?;
        let homogeneous: Vec<f64> = interleave(&lpa, &lw);
        rational_derivative(1, 1, 1, 1, &homogeneous, &mut lpa, true)?;
    }
    let vec_min: Vector3 = scaled(inverse_min, &lpa[3..6]);
    let vec_max: Vector3 = scaled(inverse_max, &lpa[6..9]);
    let (vu, vv) = if span.u_min() {
        (vec_min, vec_max)
    } else {
        (vec_max, vec_min)
    };
    Ok((point(&lpa), vu, vv))
}

// [x, y, z] rows and their weights as [x, y, z, w] rows.
fn interleave(poles: &[f64], weights: &[f64]) -> Vec<f64> {
    let mut values: Vec<f64> = Vec::with_capacity(weights.len() * 4);
    for (k, w) in weights.iter().enumerate() {
        values.extend_from_slice(&poles[3 * k..3 * k + 3]);
        values.push(*w);
    }
    values
}

fn scaled(factor: f64, values: &[f64]) -> Vector3 {
    vector(&[factor * values[0], factor * values[1], factor * values[2]])
}

/// Point and derivatives up to the second order of a cached patch
/// (`BSplSLib::CacheD2`).
///
/// Quirk kept: for a max degree below 2 the rational path reads the weights' missing
/// max-derivative rows from the evaluation buffer, where the pole rows were left;
/// `bsplslib::cache_d2(0.5, 0.5, &span, &cache, None)?`.
pub fn cache_d2(
    u: f64,
    v: f64,
    span: &SurfaceCacheSpan,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
) -> Result<SurfaceD2, Failure> {
    let (min, max) = span.degrees();
    let ([p_max, p_min], inverses) = normalized(u, v, span);
    let dimension: i32 = 3 * (min + 1);
    let dim: usize = dimension.max(0) as usize;
    let (min_ind_max, min_ind_min): (i32, i32) = (max.min(2), min.min(2));
    let mut local: Vec<f64> = vec![0.0; 3 * dim];
    // Rows beyond MinIndMax are cleared, with the pole dimension, before any evaluation.
    local[min_ind_max.max(0) as usize * dim..].fill(0.0);
    // lpa[i][j][k] at 9*i + 3*j + k.
    let mut lpa: [f64; 27] = [0.0; 27];
    eval_polynomial(
        p_max,
        min_ind_max,
        max,
        dimension,
        &pole_values(poles)?,
        &mut local,
    )?;
    eval_polynomial(p_min, min_ind_min, min, 3, &local, &mut lpa[..9])?;
    eval_polynomial(p_min, 1, min, 3, &local[dim..], &mut lpa[9..15])?;
    no_derivative_eval_polynomial(
        p_min,
        min,
        3,
        (min << 1) + min,
        &local[2 * dim..],
        &mut lpa[18..21],
    )?;
    if let Some(weights) = weights {
        let wdim: i32 = min + 1;
        let w: usize = wdim.max(0) as usize;
        let mut lw: [f64; 9] = [0.0; 9];
        eval_polynomial(
            p_max,
            min_ind_max,
            max,
            wdim,
            &weight_values(weights)?,
            &mut local,
        )?;
        eval_polynomial(p_min, min_ind_min, min, 1, &local, &mut lw[..3])?;
        eval_polynomial(p_min, 1, min, 1, &local[w..], &mut lw[3..5])?;
        no_derivative_eval_polynomial(p_min, min, 1, min, &local[2 * w..], &mut lw[6..7])?;
        let homogeneous: Vec<f64> = interleave(&lpa, &lw);
        rational_derivative(2, 2, 2, 2, &homogeneous, &mut lpa, true)?;
    }
    Ok(second_order(&lpa, inverses, span.u_min()))
}

fn second_order(lpa: &[f64; 27], [inverse_min, inverse_max]: [f64; 2], u_min: bool) -> SurfaceD2 {
    let minmin: f64 = inverse_min * inverse_min;
    let minmax: f64 = inverse_min * inverse_max;
    let maxmax: f64 = inverse_max * inverse_max;
    let vec_min: Vector3 = scaled(inverse_min, &lpa[3..6]);
    let vec_max: Vector3 = scaled(inverse_max, &lpa[9..12]);
    let min_min: Vector3 = scaled(minmin, &lpa[6..9]);
    let min_max: Vector3 = scaled(minmax, &lpa[12..15]);
    let max_max: Vector3 = scaled(maxmax, &lpa[18..21]);
    let p: Point3 = point(&lpa[..]);
    if u_min {
        return SurfaceD2 {
            p,
            vu: vec_min,
            vv: vec_max,
            vuu: min_min,
            vvv: max_max,
            vuv: min_max,
        };
    }
    SurfaceD2 {
        p,
        vu: vec_max,
        vv: vec_min,
        vuu: max_max,
        vvv: min_min,
        vuv: min_max,
    }
}

/// [`cache_d0`] on Bezier coefficients (`BSplSLib::CoefsD0`);
/// `bsplslib::coefs_d0(0.5, 0.5, &coefficients, None)?`.
pub fn coefs_d0(
    u: f64,
    v: f64,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
) -> Result<Point3, Failure> {
    cache_d0(u, v, &SurfaceCacheSpan::bezier(poles), poles, weights)
}

/// [`cache_d1`] on Bezier coefficients (`BSplSLib::CoefsD1`);
/// `bsplslib::coefs_d1(0.5, 0.5, &coefficients, None)?`.
pub fn coefs_d1(
    u: f64,
    v: f64,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
) -> Result<(Point3, Vector3, Vector3), Failure> {
    cache_d1(u, v, &SurfaceCacheSpan::bezier(poles), poles, weights)
}

/// [`cache_d2`] on Bezier coefficients (`BSplSLib::CoefsD2`);
/// `bsplslib::coefs_d2(0.5, 0.5, &coefficients, None)?`.
pub fn coefs_d2(
    u: f64,
    v: f64,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
) -> Result<SurfaceD2, Failure> {
    cache_d2(u, v, &SurfaceCacheSpan::bezier(poles), poles, weights)
}
