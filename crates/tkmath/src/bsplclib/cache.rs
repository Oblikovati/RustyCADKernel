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

    /// Derivatives up to `derivative` at the curve parameter
    /// (`calculateDerivative`); `cache.derivatives(0.5, 2)?`.
    pub fn derivatives(&self, parameter: f64, derivative: i32) -> Result<[f64; 16], Failure> {
        validate_derivative_order(self.is_rational, false, derivative)?;
        self.derivatives_local(self.local_parameter(parameter), derivative)
    }

    /// Writes packed curve-parameter rows (`calculateDerivative`); non-rational caches
    /// support any order that fits `output`;
    /// extra capacity stays unchanged;
    /// `cache.derivatives_into(0.5, 1, &mut rows)?`.
    pub fn derivatives_into(
        &self,
        parameter: f64,
        derivative: i32,
        output: &mut [f64],
    ) -> Result<(), Failure> {
        let local_param: f64 = self.local_parameter(parameter);
        if !self.is_rational && derivative > 3 {
            let needed: usize =
                ensure_derivative_output_capacity(self, derivative, output.len(), true)?;
            return self.derivatives_non_rational_local_into(
                local_param,
                derivative,
                &mut output[..needed],
            );
        }
        self.derivatives_local_into(local_param, derivative, output)
    }

    /// Writes packed rows for a normalized span parameter, scaled to curve-parameter
    /// derivatives (`calculateDerivativeLocal`); extra capacity stays unchanged;
    /// `cache.derivatives_local_into(0.25, 1, &mut rows)?`.
    pub fn derivatives_local_into(
        &self,
        local_param: f64,
        derivative: i32,
        output: &mut [f64],
    ) -> Result<(), Failure> {
        let needed: usize =
            ensure_derivative_output_capacity(self, derivative, output.len(), false)?;
        let values: [f64; 16] = self.derivatives_local(local_param, derivative)?;
        output[..needed].copy_from_slice(&values[..needed]);
        Ok(())
    }

    fn derivatives_non_rational_local_into(
        &self,
        local_param: f64,
        derivative: i32,
        output: &mut [f64],
    ) -> Result<(), Failure> {
        let (degree, order) = non_rational_orders(self, derivative)?;
        clear_uncomputed_derivative_rows(output, degree, derivative, self.row_length);
        eval_polynomial(
            local_param,
            order,
            degree,
            self.row_length as i32,
            &self.poles_weights,
            output,
        )?;
        scale_derivative_rows(output, order, self.row_length, self.params.span_length);
        Ok(())
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
        scale_derivative_rows(&mut polynomial, order.max(0), dim, self.params.span_length);
        if !self.is_rational {
            return Ok(polynomial);
        }
        rational_derivative(order, order, dim as i32 - 1, &polynomial, &mut result, true)?;
        Ok(result)
    }
}

fn ensure_derivative_output_capacity(
    cache: &Cache,
    derivative: i32,
    actual: usize,
    allow_high_order: bool,
) -> Result<usize, Failure> {
    let expected: usize = derivative_output_length(cache, derivative, allow_high_order)?;
    if actual < expected {
        return Err(Failure::with_type(
            "Standard_DimensionMismatch",
            &format!("BSplCLib_Cache output length {actual}; expected at least {expected}"),
            "",
        ));
    }
    Ok(expected)
}

fn derivative_output_length(
    cache: &Cache,
    derivative: i32,
    allow_high_order: bool,
) -> Result<usize, Failure> {
    validate_derivative_order(cache.is_rational, allow_high_order, derivative)?;
    let rows: usize = usize::try_from(derivative)
        .ok()
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| Failure::out_of_range("BSplCLib_Cache derivative row count overflow"))?;
    rows.checked_mul(cache.derivative_stride())
        .ok_or_else(|| Failure::out_of_range("BSplCLib_Cache derivative output size overflow"))
}

fn validate_derivative_order(
    is_rational: bool,
    allow_high_order: bool,
    derivative: i32,
) -> Result<(), Failure> {
    if derivative < 0 || (derivative > 3 && (is_rational || !allow_high_order)) {
        return Err(Failure::out_of_range(&format!(
            "BSplCLib_Cache derivative order {derivative}; expected {}",
            if is_rational || !allow_high_order {
                "0..=3"
            } else {
                ">= 0"
            }
        )));
    }
    Ok(())
}

fn clear_uncomputed_derivative_rows(
    output: &mut [f64],
    degree: i32,
    request: i32,
    dimension: usize,
) {
    if degree < request {
        let start: usize = degree as usize * dimension;
        output[start..].fill(0.0);
    }
}

fn non_rational_orders(cache: &Cache, derivative: i32) -> Result<(i32, i32), Failure> {
    let degree: i32 = cache.params.degree;
    if degree < 0 {
        return Err(Failure::out_of_range(&format!(
            "BSplCLib_Cache degree {degree}; expected >= 0"
        )));
    }
    Ok((degree, degree.min(derivative)))
}

fn scale_derivative_rows(values: &mut [f64], order: i32, dimension: usize, span_length: f64) {
    let mut factor: f64 = 1.0;
    for row in 1..=order as usize {
        factor /= span_length;
        for value in &mut values[dimension * row..dimension * (row + 1)] {
            *value *= factor;
        }
    }
}
