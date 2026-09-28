// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib_CurveComputation.pxx (BSplCLib_CacheD0..BSplCLib_CacheD3) and BSplCLib.lxx (CoefsD0..CoefsD3) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Evaluation of a span cached as polynomial coefficients in the normalized
//! parameter `(U - CacheParameter) / SpanLength`.
use super::flat_points;
use super::point::{CurveD3, CurvePoint};
use crate::plib::{eval_polynomial, no_derivative_eval_polynomial, rational_derivatives};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

/// Degree of a cached span and its parametrization.
#[derive(Clone, Copy, Debug)]
pub struct CacheSpan {
    /// Degree of the cached polynomial.
    pub degree: i32,
    /// Parameter where the cache was built.
    pub cache_parameter: f64,
    /// Span length used to normalize the parameter.
    pub span_length: f64,
}

impl CacheSpan {
    /// The span of a Bezier coefficient array: degree `count-1` on `[0, 1]`
    /// (the CoefsD* shortcuts); `CacheSpan::bezier(4).degree == 3`.
    pub fn bezier(count: usize) -> Self {
        Self {
            degree: count as i32 - 1,
            cache_parameter: 0.,
            span_length: 1.,
        }
    }

    fn normalized(&self, u: f64) -> f64 {
        (u - self.cache_parameter) / self.span_length
    }
}

/// Point of a cached 2D/3D span (`BSplCLib::CacheD0`); `CoefsD0` is
/// `cache_d0(u, CacheSpan::bezier(poles.len()), ..)`;
/// `bsplclib::cache_d0(0.5, span, &cache_poles, None)?`.
pub fn cache_d0<P: CurvePoint>(
    u: f64,
    span: CacheSpan,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<P, Failure> {
    let d: usize = P::DIMENSION;
    let new_parameter: f64 = span.normalized(u);
    let flat: Vec<f64> = flat_points(poles);
    let mut point: Vec<f64> = vec![0.0; d];
    let degree: i32 = span.degree;
    no_derivative_eval_polynomial(
        new_parameter,
        degree,
        d as i32,
        degree * d as i32,
        &flat,
        &mut point,
    )?;
    if let Some(weights) = weights {
        let mut inverse: [f64; 1] = [0.0];
        no_derivative_eval_polynomial(
            new_parameter,
            degree,
            1,
            degree,
            weights.as_slice(),
            &mut inverse,
        )?;
        let inverse: f64 = 1.0e0 / inverse[0];
        for value in &mut point {
            *value *= inverse;
        }
    }
    Ok(P::point_from(&point))
}

/// Point and first derivative of a cached span (`BSplCLib::CacheD1`);
/// `bsplclib::cache_d1(0.5, span, &cache_poles, None)?`.
pub fn cache_d1<P: CurvePoint>(
    u: f64,
    span: CacheSpan,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<(P, P::Vector), Failure> {
    let d: usize = P::DIMENSION;
    let new_parameter: f64 = span.normalized(u);
    let mut pd: Vec<f64> = vec![0.0; 2 * d];
    eval_polynomial(
        new_parameter,
        1,
        span.degree,
        d as i32,
        &flat_points(poles),
        &mut pd,
    )?;
    for value in &mut pd[d..] {
        *value /= span.span_length;
    }
    if let Some(weights) = weights {
        let mut wd: [f64; 2] = [0.0; 2];
        eval_polynomial(
            new_parameter,
            1,
            span.degree,
            1,
            weights.as_slice(),
            &mut wd,
        )?;
        wd[1] /= span.span_length;
        rational_in_place(1, d, &mut pd, &wd)?;
    }
    Ok((P::point_from(&pd), P::vector_from(&pd[d..])))
}

// PLib::RationalDerivatives called with the same array as input and output: each
// row is read before it is written, so a copy of the input is equivalent.
fn rational_in_place(request: i32, d: usize, pd: &mut [f64], wd: &[f64]) -> Result<(), Failure> {
    let input: Vec<f64> = pd.to_vec();
    rational_derivatives(request, d as i32, &input, wd, pd)
}

/// Point and two derivatives of a cached span (`BSplCLib::CacheD2`);
/// `bsplclib::cache_d2(0.5, span, &cache_poles, None)?`.
pub fn cache_d2<P: CurvePoint>(
    u: f64,
    span: CacheSpan,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<(P, P::Vector, P::Vector), Failure> {
    let d: usize = P::DIMENSION;
    let new_parameter: f64 = span.normalized(u);
    let mut pd: Vec<f64> = vec![0.0; 3 * d];
    eval_polynomial(
        new_parameter,
        2,
        span.degree,
        d as i32,
        &flat_points(poles),
        &mut pd,
    )?;
    let end_index: usize = span.degree.clamp(0, 2) as usize;
    scale_rows(&mut pd, d, end_index, span.span_length);
    nullify_rows(&mut pd, d, span.degree, 2);
    if let Some(weights) = weights {
        let mut wd: [f64; 3] = [0.0; 3];
        eval_polynomial(
            new_parameter,
            2,
            span.degree,
            1,
            weights.as_slice(),
            &mut wd,
        )?;
        nullify_rows(&mut wd, 1, span.degree, 2);
        scale_rows(&mut wd, 1, end_index, span.span_length);
        rational_in_place(2, d, &mut pd, &wd)?;
    }
    Ok((
        P::point_from(&pd),
        P::vector_from(&pd[d..]),
        P::vector_from(&pd[2 * d..]),
    ))
}

/// Point and three derivatives of a cached span (`BSplCLib::CacheD3`);
/// `bsplclib::cache_d3(0.5, span, &cache_poles, None)?`.
pub fn cache_d3<P: CurvePoint>(
    u: f64,
    span: CacheSpan,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<CurveD3<P>, Failure> {
    let d: usize = P::DIMENSION;
    let new_parameter: f64 = span.normalized(u);
    let mut pd: Vec<f64> = vec![0.0; 4 * d];
    eval_polynomial(
        new_parameter,
        3,
        span.degree,
        d as i32,
        &flat_points(poles),
        &mut pd,
    )?;
    // D3 clears the rows above the degree before scaling, D2 after.
    nullify_rows(&mut pd, d, span.degree, 3);
    let end_index: usize = span.degree.clamp(0, 3) as usize;
    scale_rows(&mut pd, d, end_index, span.span_length);
    if let Some(weights) = weights {
        let mut wd: [f64; 4] = [0.0; 4];
        eval_polynomial(
            new_parameter,
            3,
            span.degree,
            1,
            weights.as_slice(),
            &mut wd,
        )?;
        scale_rows(&mut wd, 1, end_index, span.span_length);
        nullify_rows(&mut wd, 1, span.degree, 3);
        rational_in_place(3, d, &mut pd, &wd)?;
    }
    Ok((
        P::point_from(&pd),
        P::vector_from(&pd[d..]),
        P::vector_from(&pd[2 * d..]),
        P::vector_from(&pd[3 * d..]),
    ))
}

// Row r (1..=end_index) *= 1/SpanLength^r, the factor divided by SpanLength each row.
fn scale_rows(values: &mut [f64], d: usize, end_index: usize, span_length: f64) {
    let mut factor: f64 = 1.0e0 / span_length;
    for row in 1..=end_index {
        for value in &mut values[row * d..(row + 1) * d] {
            *value *= factor;
        }
        factor /= span_length;
    }
}

// Clears the derivative rows Degree+1..=last.
fn nullify_rows(values: &mut [f64], d: usize, degree: i32, last: i32) {
    for row in degree + 1..=last {
        if row >= 0 {
            let row: usize = row as usize;
            values[row * d..(row + 1) * d].fill(0.);
        }
    }
}

/// [`cache_d0`] on Bezier coefficients from `poles_coefficients` (`BSplCLib::CoefsD0`);
/// `bsplclib::coefs_d0(0.5, &coefficients, None)?`.
pub fn coefs_d0<P: CurvePoint>(
    u: f64,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<P, Failure> {
    cache_d0(u, CacheSpan::bezier(poles.len()), poles, weights)
}

/// [`cache_d1`] on Bezier coefficients (`BSplCLib::CoefsD1`);
/// `bsplclib::coefs_d1(0.5, &coefficients, None)?`.
pub fn coefs_d1<P: CurvePoint>(
    u: f64,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<(P, P::Vector), Failure> {
    cache_d1(u, CacheSpan::bezier(poles.len()), poles, weights)
}

/// [`cache_d2`] on Bezier coefficients (`BSplCLib::CoefsD2`);
/// `bsplclib::coefs_d2(0.5, &coefficients, None)?`.
pub fn coefs_d2<P: CurvePoint>(
    u: f64,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<(P, P::Vector, P::Vector), Failure> {
    cache_d2(u, CacheSpan::bezier(poles.len()), poles, weights)
}

/// [`cache_d3`] on Bezier coefficients (`BSplCLib::CoefsD3`);
/// `bsplclib::coefs_d3(0.5, &coefficients, None)?`.
pub fn coefs_d3<P: CurvePoint>(
    u: f64,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<CurveD3<P>, Failure> {
    cache_d3(u, CacheSpan::bezier(poles.len()), poles, weights)
}
