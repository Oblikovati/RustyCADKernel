// Copyright (c) 2014 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib_Cache.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Polynomial cache of one span of a 2D or 3D B-spline curve (`BSplCLib_Cache`).
use super::build_cache::build_cache_matrix;
use super::cache_params::CacheParams;
use super::point::{CurveD3, CurvePoint};
use crate::plib::{eval_polynomial, no_derivative_eval_polynomial, rational_derivative};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;

/// Cached Taylor coefficients of the span last located (`BSplCLib_Cache`).
///
/// Rows of the cache hold the point coordinates, then the weight when the curve is
/// rational; the row length is fixed by the pole type given at construction.
#[derive(Debug)]
pub struct Cache {
    is_rational: bool,
    params: CacheParams,
    row_length: usize,
    poles_weights: [f64; 128],
}

impl Cache {
    /// Prepares an empty cache for a curve of `degree` on flat knots; `poles` only
    /// select the 2D or 3D layout; `Cache::new(3, false, &flat, &poles, None)?`.
    pub fn new<P: CurvePoint>(
        degree: i32,
        periodic: bool,
        flat_knots: &Array1<'_, f64>,
        _poles: &Array1<'_, P>,
        weights: Option<&Array1<'_, f64>>,
    ) -> Result<Self, Failure> {
        let is_rational: bool = weights.is_some();
        Ok(Self {
            is_rational,
            params: CacheParams::new(degree, periodic, flat_knots)?,
            row_length: P::DIMENSION + usize::from(is_rational),
            poles_weights: [0.0; 128],
        })
    }

    /// The span bookkeeping; `cache.params().span_index`.
    pub fn params(&self) -> &CacheParams {
        &self.params
    }

    /// Tests whether `parameter` lies in the cached span (`IsCacheValid`);
    /// `cache.is_cache_valid(0.5)`.
    pub fn is_cache_valid(&self, parameter: f64) -> bool {
        self.params.is_cache_valid(parameter)
    }

    /// Recomputes the cache on the span of `parameter` (`BuildCache`);
    /// `cache.build_cache(0.5, &flat, &poles, None)?`.
    pub fn build_cache<P: CurvePoint>(
        &mut self,
        parameter: f64,
        flat_knots: &Array1<'_, f64>,
        poles: &Array1<'_, P>,
        weights: Option<&Array1<'_, f64>>,
    ) -> Result<(), Failure> {
        let mut new_param: f64 = self.params.periodic_normalization(parameter);
        self.params.locate_parameter(&mut new_param, flat_knots)?;
        let rows: i32 = self.params.degree + 1;
        let used: usize = (rows.max(0) as usize * self.row_length).min(self.poles_weights.len());
        let mut cache_array: Array2<'_, f64> = Array2::from_buffer(
            &mut self.poles_weights[..used],
            rows.max(0) as usize,
            self.row_length,
        )
        .map_err(|error| Failure::out_of_range(&format!("BSplCLib_Cache: {error}")))?;
        cache_array.update_lower_row(1);
        cache_array.update_lower_col(1);
        build_cache_matrix(
            self.params.span_start,
            self.params.span_length,
            self.params.is_periodic,
            self.params.degree,
            self.params.span_index,
            flat_knots,
            poles,
            weights,
            &mut cache_array,
        )
    }

    fn local_parameter(&self, parameter: f64) -> f64 {
        let normalized: f64 = self.params.periodic_normalization(parameter);
        (normalized - self.params.span_start) / self.params.span_length
    }

    /// Point at `parameter` (`D0`); `cache.d0::<Point3>(0.5)?`.
    pub fn d0<P: CurvePoint>(&self, parameter: f64) -> Result<P, Failure> {
        self.d0_local(self.local_parameter(parameter))
    }

    /// Point at a normalized span parameter (`D0Local`); `cache.d0_local::<Point3>(0.5)?`.
    pub fn d0_local<P: CurvePoint>(&self, local_param: f64) -> Result<P, Failure> {
        let mut point: [f64; 4] = [0.0; 4];
        let degree: i32 = self.params.degree;
        let width: i32 = self.row_length as i32;
        no_derivative_eval_polynomial(
            local_param,
            degree,
            width,
            degree * width,
            &self.poles_weights,
            &mut point,
        )?;
        let mut result: P = P::point_from(&point);
        if self.is_rational {
            // gp_XYZ::Divide by the weight stored after the coordinates.
            let weight: f64 = point[P::DIMENSION];
            for (k, coordinate) in point.iter().enumerate().take(P::DIMENSION) {
                result.set_pole_coordinate(k, coordinate / weight);
            }
        }
        Ok(result)
    }

    /// Point and tangent (`D1`); `cache.d1::<Point3>(0.5)?`.
    pub fn d1<P: CurvePoint>(&self, parameter: f64) -> Result<(P, P::Vector), Failure> {
        self.d1_local(self.local_parameter(parameter))
    }

    /// Point and tangent at a normalized parameter (`D1Local`); `cache.d1_local::<Point3>(0.5)?`.
    pub fn d1_local<P: CurvePoint>(&self, local_param: f64) -> Result<(P, P::Vector), Failure> {
        let derivatives: [f64; 16] = self.derivatives_local(local_param, 1)?;
        let dim: usize = self.derivative_stride();
        Ok((
            P::point_from(&derivatives),
            P::vector_from(&derivatives[dim..]),
        ))
    }

    /// Point, tangent and curvature vector (`D2`); `cache.d2::<Point3>(0.5)?`.
    pub fn d2<P: CurvePoint>(&self, parameter: f64) -> Result<(P, P::Vector, P::Vector), Failure> {
        self.d2_local(self.local_parameter(parameter))
    }

    /// Second-order evaluation at a normalized parameter (`D2Local`);
    /// `cache.d2_local::<Point3>(0.5)?`.
    pub fn d2_local<P: CurvePoint>(
        &self,
        local_param: f64,
    ) -> Result<(P, P::Vector, P::Vector), Failure> {
        let derivatives: [f64; 16] = self.derivatives_local(local_param, 2)?;
        let dim: usize = self.derivative_stride();
        Ok((
            P::point_from(&derivatives),
            P::vector_from(&derivatives[dim..]),
            P::vector_from(&derivatives[2 * dim..]),
        ))
    }

    /// Point and three derivatives (`D3`); `cache.d3::<Point3>(0.5)?`.
    pub fn d3<P: CurvePoint>(&self, parameter: f64) -> Result<CurveD3<P>, Failure> {
        self.d3_local(self.local_parameter(parameter))
    }

    /// Third-order evaluation at a normalized parameter (`D3Local`);
    /// `cache.d3_local::<Point3>(0.5)?`.
    pub fn d3_local<P: CurvePoint>(&self, local_param: f64) -> Result<CurveD3<P>, Failure> {
        let derivatives: [f64; 16] = self.derivatives_local(local_param, 3)?;
        let dim: usize = self.derivative_stride();
        Ok((
            P::point_from(&derivatives),
            P::vector_from(&derivatives[dim..]),
            P::vector_from(&derivatives[2 * dim..]),
            P::vector_from(&derivatives[3 * dim..]),
        ))
    }

    fn derivative_stride(&self) -> usize {
        if self.is_rational {
            self.row_length - 1
        } else {
            self.row_length
        }
    }

    /// Derivatives up to `derivative` at a normalized parameter (`calculateDerivative`
    /// after normalization); `cache.derivatives(0.5, 2)?`.
    pub fn derivatives(&self, parameter: f64, derivative: i32) -> Result<[f64; 16], Failure> {
        self.derivatives_local(self.local_parameter(parameter), derivative)
    }

    // calculateDerivativeLocal: rows of `derivative_stride()` values.
    fn derivatives_local(&self, local_param: f64, derivative: i32) -> Result<[f64; 16], Failure> {
        let dim: usize = self.row_length;
        let degree: i32 = self.params.degree;
        let mut result: [f64; 16] = [0.0; 16];
        let mut polynomial: [f64; 16] = [0.0; 16];
        let mut order: i32 = derivative;
        // When the degree is below the request, the higher derivatives are nullified.
        if !self.is_rational && degree < derivative {
            order = degree;
            let from: usize = (degree.max(0) as usize) * dim;
            polynomial[from..(derivative as usize + 1) * dim].fill(0.0);
        }
        eval_polynomial(
            local_param,
            order,
            degree,
            dim as i32,
            &self.poles_weights,
            &mut polynomial,
        )?;
        // Unnormalized by successive divisions, stabler than a precomputed inverse.
        let mut factor: f64 = 1.0;
        for row in 1..=order.max(0) as usize {
            factor /= self.params.span_length;
            for value in &mut polynomial[dim * row..dim * (row + 1)] {
                *value *= factor;
            }
        }
        if !self.is_rational {
            return Ok(polynomial);
        }
        rational_derivative(order, order, dim as i32 - 1, &polynomial, &mut result, true)?;
        Ok(result)
    }
}
