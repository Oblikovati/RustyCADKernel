// Rust translation of FoundationClasses/TKMath/BSplSLib/BSplSLib_Cache.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Polynomial cache of one patch of a B-spline surface (`BSplSLib_Cache`).
//!
//! BSplSLib parametrizes a cached span by its middle and half length, unlike
//! BSplCLib; the cache is rebuilt around that convention.
use super::build_cache::{CacheSource, build_cache_matrix};
use super::eval::{SurfaceD2, point, vector};
use super::rational::rational_derivative;
use crate::bsplclib::CacheParams;
use crate::gp::{Point3, Vector3};
use crate::plib::{eval_polynomial, no_derivative_eval_polynomial};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;

/// Cached Taylor coefficients of the patch last located (`BSplSLib_Cache`).
pub struct Cache {
    is_rational: bool,
    params_u: CacheParams,
    params_v: CacheParams,
    poles_weights: Array2<'static, f64>,
}

impl Cache {
    /// Prepares an empty cache of `(max degree + 1) x (3 or 4)*(min degree + 1)` values;
    /// `Cache::new(3, false, &u_flat, 2, false, &v_flat, None)?`.
    pub fn new(
        degree_u: i32,
        periodic_u: bool,
        flat_knots_u: &Array1<'_, f64>,
        degree_v: i32,
        periodic_v: bool,
        flat_knots_v: &Array1<'_, f64>,
        weights: Option<&Array2<'_, f64>>,
    ) -> Result<Self, Failure> {
        let is_rational: bool = weights.is_some();
        let (min_degree, max_degree) = (degree_u.min(degree_v), degree_u.max(degree_v));
        let columns: i32 = (if is_rational { 4 } else { 3 }) * (min_degree + 1);
        let poles_weights: Array2<'static, f64> =
            Array2::with_bounds(1, max_degree + 1, 1, columns)
                .map_err(|error| Failure::out_of_range(&format!("BSplSLib_Cache: {error}")))?;
        Ok(Self {
            is_rational,
            params_u: CacheParams::new(degree_u, periodic_u, flat_knots_u)?,
            params_v: CacheParams::new(degree_v, periodic_v, flat_knots_v)?,
            poles_weights,
        })
    }

    /// Tests whether (U, V) lies in the cached patch (`IsCacheValid`);
    /// `cache.is_cache_valid(0.5, 0.5)`.
    pub fn is_cache_valid(&self, parameter_u: f64, parameter_v: f64) -> bool {
        self.params_u.is_cache_valid(parameter_u) && self.params_v.is_cache_valid(parameter_v)
    }

    // Span middle and half length, the BSplSLib convention.
    fn half_span(params: &CacheParams) -> (f64, f64) {
        let length: f64 = 0.5 * params.span_length;
        (params.span_start + length, length)
    }

    /// Recomputes the cache on the patch of (U, V) (`BuildCache`);
    /// `cache.build_cache(0.5, 0.5, &u_flat, &v_flat, &poles, None)?`.
    pub fn build_cache(
        &mut self,
        parameter_u: f64,
        parameter_v: f64,
        flat_knots_u: &Array1<'_, f64>,
        flat_knots_v: &Array1<'_, f64>,
        poles: &Array2<'_, Point3>,
        weights: Option<&Array2<'_, f64>>,
    ) -> Result<(), Failure> {
        let mut new_u: f64 = self.params_u.periodic_normalization(parameter_u);
        let mut new_v: f64 = self.params_v.periodic_normalization(parameter_v);
        self.params_u.locate_parameter(&mut new_u, flat_knots_u)?;
        self.params_v.locate_parameter(&mut new_v, flat_knots_v)?;
        let (start_u, length_u) = Self::half_span(&self.params_u);
        let (start_v, length_v) = Self::half_span(&self.params_v);
        let source = CacheSource {
            u: start_u,
            v: start_v,
            u_span_domain: length_u,
            v_span_domain: length_v,
            u_periodic: self.params_u.is_periodic,
            v_periodic: self.params_v.is_periodic,
            u_degree: self.params_u.degree,
            v_degree: self.params_v.degree,
            u_index: self.params_u.span_index,
            v_index: self.params_v.span_index,
            u_flat_knots: flat_knots_u,
            v_flat_knots: flat_knots_v,
        };
        build_cache_matrix(&source, poles, weights, &mut self.poles_weights)
    }

    fn u_is_max(&self) -> bool {
        self.params_u.degree > self.params_v.degree
    }

    /// Point at (U, V) (`D0`); `cache.d0(0.5, 0.5)?`.
    pub fn d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        let (start_u, length_u) = Self::half_span(&self.params_u);
        let (start_v, length_v) = Self::half_span(&self.params_v);
        let local_u: f64 = (self.params_u.periodic_normalization(u) - start_u) / length_u;
        let local_v: f64 = (self.params_v.periodic_normalization(v) - start_v) / length_v;
        self.d0_local(local_u, local_v)
    }

    /// Point at normalized patch parameters (`D0Local`); `cache.d0_local(0.0, 0.0)?`.
    pub fn d0_local(&self, local_u: f64, local_v: f64) -> Result<Point3, Failure> {
        let dimension: i32 = if self.is_rational { 4 } else { 3 };
        let cols: i32 = self.poles_weights.nb_columns() as i32;
        let (min_degree, max_degree) = self.degrees();
        let (min_param, max_param) = if self.u_is_max() {
            (local_v, local_u)
        } else {
            (local_u, local_v)
        };
        let mut transient: Vec<f64> = vec![0.0; cols.max(0) as usize];
        no_derivative_eval_polynomial(
            max_param,
            max_degree,
            cols,
            max_degree * cols,
            self.poles_weights.as_slice(),
            &mut transient,
        )?;
        let mut values: [f64; 4] = [0.0; 4];
        no_derivative_eval_polynomial(
            min_param,
            min_degree,
            dimension,
            dimension * min_degree,
            &transient,
            &mut values,
        )?;
        if self.is_rational {
            return Ok(Point3::new(
                values[0] / values[3],
                values[1] / values[3],
                values[2] / values[3],
            ));
        }
        Ok(point(&values))
    }

    fn degrees(&self) -> (i32, i32) {
        let (u, v) = (self.params_u.degree, self.params_v.degree);
        (u.min(v), u.max(v))
    }

    // Local parameters and inverse half spans, reused for the derivative scaling.
    fn local_with_inverses(&self, u: f64, v: f64) -> ([f64; 2], [f64; 2]) {
        let (start_u, length_u) = Self::half_span(&self.params_u);
        let (start_v, length_v) = Self::half_span(&self.params_v);
        let (inverse_u, inverse_v) = (1.0 / length_u, 1.0 / length_v);
        let local_u: f64 = (self.params_u.periodic_normalization(u) - start_u) * inverse_u;
        let local_v: f64 = (self.params_v.periodic_normalization(v) - start_v) * inverse_v;
        ([local_u, local_v], [inverse_u, inverse_v])
    }

    /// Point and tangents at (U, V) (`D1`); `cache.d1(0.5, 0.5)?`.
    pub fn d1(&self, u: f64, v: f64) -> Result<(Point3, Vector3, Vector3), Failure> {
        let ([local_u, local_v], [inverse_u, inverse_v]) = self.local_with_inverses(u, v);
        let (p, du, dv) = self.first_order(local_u, local_v)?;
        Ok((p, du * inverse_u, dv * inverse_v))
    }

    /// Point and tangents at normalized parameters (`D1Local`), divided by the half
    /// spans; `cache.d1_local(0.0, 0.0)?`.
    pub fn d1_local(
        &self,
        local_u: f64,
        local_v: f64,
    ) -> Result<(Point3, Vector3, Vector3), Failure> {
        let (p, du, dv) = self.first_order(local_u, local_v)?;
        let (length_u, length_v) = (
            0.5 * self.params_u.span_length,
            0.5 * self.params_v.span_length,
        );
        Ok((p, divided(du, length_u), divided(dv, length_v)))
    }

    fn first_order(
        &self,
        local_u: f64,
        local_v: f64,
    ) -> Result<(Point3, Vector3, Vector3), Failure> {
        let mut values: [f64; 36] = [0.0; 36];
        self.evaluate_polynomials(local_u, local_v, [1, 1], &mut values)?;
        let (first, second) = (vector(&values[3..]), vector(&values[6..]));
        let (du, dv) = if self.u_is_max() {
            (second, first)
        } else {
            (first, second)
        };
        Ok((point(&values), du, dv))
    }

    /// Point, tangents and second derivatives at (U, V) (`D2`); `cache.d2(0.5, 0.5)?`.
    pub fn d2(&self, u: f64, v: f64) -> Result<SurfaceD2, Failure> {
        let ([local_u, local_v], [iu, iv]) = self.local_with_inverses(u, v);
        let d = self.second_order(local_u, local_v)?;
        Ok(SurfaceD2 {
            p: d.p,
            vu: d.vu * iu,
            vv: d.vv * iv,
            vuu: d.vuu * (iu * iu),
            vvv: d.vvv * (iv * iv),
            vuv: d.vuv * (iu * iv),
        })
    }

    /// Second-order evaluation at normalized parameters (`D2Local`), divided by the
    /// half spans and their products; `cache.d2_local(0.0, 0.0)?`.
    pub fn d2_local(&self, local_u: f64, local_v: f64) -> Result<SurfaceD2, Failure> {
        let d = self.second_order(local_u, local_v)?;
        let (lu, lv) = (
            0.5 * self.params_u.span_length,
            0.5 * self.params_v.span_length,
        );
        Ok(SurfaceD2 {
            p: d.p,
            vu: divided(d.vu, lu),
            vv: divided(d.vv, lv),
            vuu: divided(d.vuu, lu * lu),
            vvv: divided(d.vvv, lv * lv),
            vuv: divided(d.vuv, lu * lv),
        })
    }

    fn second_order(&self, local_u: f64, local_v: f64) -> Result<SurfaceD2, Failure> {
        let mut values: [f64; 36] = [0.0; 36];
        self.evaluate_polynomials(local_u, local_v, [2, 2], &mut values)?;
        // Rows of the max-degree direction: [P, Dmin, Dminmin], [Dmax, Dmaxmin], [Dmaxmax].
        let at = |offset: usize| vector(&values[offset..]);
        let (d_min, d_min_min, d_max, d_mixed, d_max_max) = (at(3), at(6), at(9), at(12), at(18));
        if self.u_is_max() {
            return Ok(SurfaceD2 {
                p: point(&values),
                vu: d_max,
                vv: d_min,
                vuu: d_max_max,
                vvv: d_min_min,
                vuv: d_mixed,
            });
        }
        Ok(SurfaceD2 {
            p: point(&values),
            vu: d_min,
            vv: d_max,
            vuu: d_min_min,
            vvv: d_max_max,
            vuv: d_mixed,
        })
    }
}

// gp_Vec::Divide.
fn divided(vector: Vector3, divisor: f64) -> Vector3 {
    Vector3::new(
        vector.x() / divisor,
        vector.y() / divisor,
        vector.z() / divisor,
    )
}

mod polynomials;
