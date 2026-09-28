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
// Rust translation of FoundationClasses/TKMath/BSplSLib/BSplSLib.cxx (BuildCache) and BSplSLib_BzSyntaxes.cxx (PolesCoefficients) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Taylor coefficients of a surface patch, rows along the direction of greater
//! degree and columns along the other ([MaxCoeff, MinCoeff]).
use super::eval::point;
use super::prepare::{Patch, SurfaceSpan, prepare_eval, validate_degrees};
use crate::bsplclib::bohm;
use crate::gp::Point3;
use crate::plib::access::grid_cell;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;

/// Flat knots, degrees, span hints and span domains of a cache construction.
#[derive(Clone, Copy)]
pub struct CacheSource<'r, 'a> {
    /// U parameter of the span.
    pub u: f64,
    /// V parameter of the span.
    pub v: f64,
    /// Span length multiplied into the U coefficients.
    pub u_span_domain: f64,
    /// Span length multiplied into the V coefficients.
    pub v_span_domain: f64,
    /// Periodicity in U.
    pub u_periodic: bool,
    /// Periodicity in V.
    pub v_periodic: bool,
    /// U degree.
    pub u_degree: i32,
    /// V degree.
    pub v_degree: i32,
    /// U span hint (flat knot index).
    pub u_index: i32,
    /// V span hint (flat knot index).
    pub v_index: i32,
    /// U flat knots.
    pub u_flat_knots: &'r Array1<'a, f64>,
    /// V flat knots.
    pub v_flat_knots: &'r Array1<'a, f64>,
}

// Taylor expansion of the patch: Bohm to all orders in both directions.
fn taylor_patch(
    source: &CacheSource<'_, '_>,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
) -> Result<Patch, Failure> {
    validate_degrees(source.u_degree, source.v_degree)?;
    let weighted: bool = weights.is_some();
    let span = SurfaceSpan {
        u: source.u,
        v: source.v,
        u_index: source.u_index,
        v_index: source.v_index,
        u_knots: source.u_flat_knots,
        v_knots: source.v_flat_knots,
        u_mults: None,
        v_mults: None,
        u_degree: source.u_degree,
        v_degree: source.v_degree,
        u_rat: weighted,
        v_rat: weighted,
        u_per: source.u_periodic,
        v_per: source.v_periodic,
    };
    let mut patch: Patch = prepare_eval(&span, poles, weights)?;
    let dim: usize = patch.dim();
    let row: usize = dim * (patch.d2 as usize + 1);
    bohm(
        patch.u1,
        patch.d1,
        patch.d1,
        &patch.knots1,
        row as i32,
        &mut patch.poles,
    )?;
    for kk in 0..=patch.d1.max(-1) as usize {
        let offset: usize = kk * row;
        bohm(
            patch.u2,
            patch.d2,
            patch.d2,
            &patch.knots2,
            dim as i32,
            &mut patch.poles[offset..],
        )?;
    }
    Ok(patch)
}

// [min-degree domain, max-degree domain].
fn domains(source: &CacheSource<'_, '_>, u_first: bool) -> [f64; 2] {
    if u_first {
        [source.u_span_domain, source.v_span_domain]
    } else {
        [source.v_span_domain, source.u_span_domain]
    }
}

/// Stores the Taylor expansion into point and weight grids indexed from (1, 1),
/// `(max-degree order + 1, min-degree order + 1)` (`BSplSLib::BuildCache`).
///
/// A weighted but locally polynomial patch gets cache weights 1 at (1, 1), else 0;
/// `bsplslib::build_cache(&source, &poles, None, &mut cache_poles, None)?`.
pub fn build_cache(
    source: &CacheSource<'_, '_>,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
    cache_poles: &mut Array2<'_, Point3>,
    mut cache_weights: Option<&mut Array2<'_, f64>>,
) -> Result<(), Failure> {
    let patch: Patch = taylor_patch(source, poles, weights)?;
    let dim: usize = patch.dim();
    let [min_domain, max_domain] = domains(source, patch.u_first);
    let (d1p1, d2p1): (usize, usize) = (patch.d1 as usize + 1, patch.d2 as usize + 1);
    let mut factor0: f64 = 1.0e0;
    for ii in 0..d2p1 {
        let iii: i32 = ii as i32 + 1;
        let mut factor1: f64 = 1.0e0;
        for jj in 0..d1p1 {
            let jjj: i32 = jj as i32 + 1;
            let index: usize = (jj * d2p1 + ii) * dim;
            let f: f64 = factor0 * factor1;
            let values: [f64; 3] = [
                f * patch.poles[index],
                f * patch.poles[index + 1],
                f * patch.poles[index + 2],
            ];
            *grid_cell(cache_poles, iii, jjj)? = point(&values);
            if patch.rational {
                *grid_cell(weights_of(&mut cache_weights)?, iii, jjj)? = f * patch.poles[index + 3];
            }
            factor1 *= min_domain / f64::from(jjj);
        }
        factor0 *= max_domain / f64::from(iii);
    }
    if !patch.rational && weights.is_some() {
        // Locally polynomial with weights: a constant weight polynomial.
        let cache_weights = weights_of(&mut cache_weights)?;
        for ii in 1..=d2p1 as i32 {
            for jj in 1..=d1p1 as i32 {
                *grid_cell(cache_weights, ii, jj)? = 0.0e0;
            }
        }
        *grid_cell(cache_weights, 1, 1)? = 1.0e0;
    }
    Ok(())
}

fn weights_of<'r, 'a>(
    weights: &'r mut Option<&mut Array2<'a, f64>>,
) -> Result<&'r mut Array2<'a, f64>, Failure> {
    weights.as_deref_mut().ok_or_else(|| {
        Failure::out_of_range(
            "BSplSLib::BuildCache: rational patch needs CacheWeights; received None",
        )
    })
}

/// Stores the Taylor expansion into one real matrix read from its first cell: rows
/// along the max-degree direction, each holding (min degree + 1) poles of 3 values
/// (4 when weighted) (`BSplSLib::BuildCache` into `NCollection_Array2<double>`);
/// `bsplslib::build_cache_matrix(&source, &poles, None, &mut cache)?`.
pub fn build_cache_matrix(
    source: &CacheSource<'_, '_>,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
    cache_array: &mut Array2<'_, f64>,
) -> Result<(), Failure> {
    let patch: Patch = taylor_patch(source, poles, weights)?;
    let dim: usize = patch.dim();
    let shift: usize = if weights.is_some() && !patch.rational {
        dim + 1
    } else {
        dim
    };
    let [domain_min, domain_max] = domains(source, patch.u_first);
    let (d1p1, d2p1): (usize, usize) = (patch.d1 as usize + 1, patch.d2 as usize + 1);
    let cache: &mut [f64] = cache_array.as_mut_slice();
    if cache.len() < d1p1 * d2p1 * shift {
        return Err(Failure::out_of_range(&format!(
            "BSplSLib::BuildCache: cache of {} values; expected at least {}",
            cache.len(),
            d1p1 * d2p1 * shift
        )));
    }
    let mut factor_max: f64 = 1.0;
    let mut at: usize = 0;
    for row in 0..d2p1 {
        let mut factor_min: f64 = 1.0;
        for col in 0..d1p1 {
            let coefficients: usize = (col * d2p1 + row) * dim;
            let coeff: f64 = factor_min * factor_max;
            for i in 0..dim {
                cache[at + i] = patch.poles[coefficients + i] * coeff;
            }
            at += shift;
            factor_min *= domain_min / (col + 1) as f64;
        }
        factor_max *= domain_max / (row + 1) as f64;
    }
    if shift > dim {
        // A weighted surface found locally polynomial: a constant weight polynomial.
        for k in 0..d1p1 * d2p1 {
            cache[k * shift + shift - 1] = 0.0;
        }
        let (row, col) = (cache_array.lower_row(), cache_array.lower_col());
        *grid_cell(cache_array, row, col + shift as i32 - 1)? = 1.0;
    }
    Ok(())
}

/// Polynomial coefficients of a Bezier surface (`BSplSLib::PolesCoefficients`; the
/// weightless overload passes None twice). Results are indexed like the poles;
/// `bsplslib::poles_coefficients(&poles, None, &mut coefficients, None)?`.
pub fn poles_coefficients(
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
    cache_poles: &mut Array2<'_, Point3>,
    cache_weights: Option<&mut Array2<'_, f64>>,
) -> Result<(), Failure> {
    let (uclas, vclas): (i32, i32) = (poles.nb_rows() as i32, poles.nb_columns() as i32);
    let u_flat: Array1<'static, f64> = bezier_flat(uclas)?;
    let v_flat: Array1<'static, f64> = bezier_flat(vclas)?;
    let source = CacheSource {
        u: 0.,
        v: 0.,
        u_span_domain: 1.,
        v_span_domain: 1.,
        u_periodic: false,
        v_periodic: false,
        u_degree: uclas - 1,
        v_degree: vclas - 1,
        u_index: 0,
        v_index: 0,
        u_flat_knots: &u_flat,
        v_flat_knots: &v_flat,
    };
    if uclas > vclas {
        return build_cache(&source, poles, weights, cache_poles, cache_weights);
    }
    // BuildCache formats its results as [MaxCoeff, MinCoeff]: transpose them.
    let mut c_poles: Array2<'static, Point3> = grid(vclas, uclas)?;
    let mut c_weights: Array2<'static, f64> = grid(vclas, uclas)?;
    build_cache(&source, poles, weights, &mut c_poles, Some(&mut c_weights))?;
    transpose_into(&c_poles, cache_poles)?;
    if weights.is_some() {
        let cache_weights = cache_weights.ok_or_else(|| {
            Failure::out_of_range(
                "BSplSLib::PolesCoefficients: weights need CacheWeights; received None",
            )
        })?;
        transpose_into(&c_weights, cache_weights)?;
    }
    Ok(())
}

fn bezier_flat(count: i32) -> Result<Array1<'static, f64>, Failure> {
    let mut flat: Array1<'static, f64> = Array1::with_bounds(1, count << 1)
        .map_err(|error| Failure::out_of_range(&format!("BSplSLib::PolesCoefficients: {error}")))?;
    for i in 1..=count {
        flat[i] = 0.;
        flat[i + count] = 1.;
    }
    Ok(flat)
}

fn grid<T: Default>(rows: i32, cols: i32) -> Result<Array2<'static, T>, Failure> {
    Array2::with_bounds(1, rows, 1, cols)
        .map_err(|error| Failure::out_of_range(&format!("BSplSLib: {error}")))
}

fn transpose_into<T: Copy>(
    source: &Array2<'_, T>,
    target: &mut Array2<'_, T>,
) -> Result<(), Failure> {
    for ii in 1..=source.nb_columns() as i32 {
        for jj in 1..=source.nb_rows() as i32 {
            let value: T = *source
                .value(jj, ii)
                .map_err(|error| Failure::out_of_range(&format!("NCollection_Array2: {error}")))?;
            *grid_cell(target, ii, jj)? = value;
        }
    }
    Ok(())
}
