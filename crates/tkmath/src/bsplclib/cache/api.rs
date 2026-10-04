use super::Cache;
use crate::bsplclib::point::{CurveD3, CurvePoint};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

impl Cache {
    /// Copies current span bookkeeping; `cache.params().span_index`.
    pub fn params(&self) -> super::CacheParams {
        self.lock_state().params()
    }

    /// Tests whether `parameter` lies in the cached span; `cache.is_cache_valid(0.5)`.
    pub fn is_cache_valid(&self, parameter: f64) -> bool {
        self.lock_state().is_cache_valid(parameter)
    }

    /// Recomputes the cache span; `cache.build_cache(0.5, &flat, &poles, None)?`.
    pub fn build_cache<P: CurvePoint>(
        &self,
        parameter: f64,
        flat: &Array1<'_, f64>,
        poles: &Array1<'_, P>,
        weights: Option<&Array1<'_, f64>>,
    ) -> Result<(), Failure> {
        self.lock_state()
            .build_cache(parameter, flat, poles, weights)
    }

    /// Evaluates a point; `cache.d0::<Point3>(0.5)?`.
    pub fn d0<P: CurvePoint>(&self, parameter: f64) -> Result<P, Failure> {
        self.lock_state().d0(parameter)
    }

    /// Evaluates a normalized point; `cache.d0_local::<Point3>(0.5)?`.
    pub fn d0_local<P: CurvePoint>(&self, parameter: f64) -> Result<P, Failure> {
        self.lock_state().d0_local(parameter)
    }

    /// Evaluates point and tangent; `cache.d1::<Point3>(0.5)?`.
    pub fn d1<P: CurvePoint>(&self, parameter: f64) -> Result<(P, P::Vector), Failure> {
        self.lock_state().d1(parameter)
    }

    /// Evaluates normalized point and tangent; `cache.d1_local::<Point3>(0.5)?`.
    pub fn d1_local<P: CurvePoint>(&self, parameter: f64) -> Result<(P, P::Vector), Failure> {
        self.lock_state().d1_local(parameter)
    }

    /// Evaluates point and two derivatives; `cache.d2::<Point3>(0.5)?`.
    pub fn d2<P: CurvePoint>(&self, parameter: f64) -> Result<(P, P::Vector, P::Vector), Failure> {
        self.lock_state().d2(parameter)
    }

    /// Evaluates normalized point and two derivatives; `cache.d2_local::<Point3>(0.5)?`.
    pub fn d2_local<P: CurvePoint>(
        &self,
        parameter: f64,
    ) -> Result<(P, P::Vector, P::Vector), Failure> {
        self.lock_state().d2_local(parameter)
    }

    /// Evaluates point through third derivative; `cache.d3::<Point3>(0.5)?`.
    pub fn d3<P: CurvePoint>(&self, parameter: f64) -> Result<CurveD3<P>, Failure> {
        self.lock_state().d3(parameter)
    }

    /// Evaluates normalized point through third derivative; `cache.d3_local::<Point3>(0.5)?`.
    pub fn d3_local<P: CurvePoint>(&self, parameter: f64) -> Result<CurveD3<P>, Failure> {
        self.lock_state().d3_local(parameter)
    }

    /// Evaluates packed derivatives; `cache.derivatives(0.5, 2)?`.
    pub fn derivatives(&self, parameter: f64, derivative: i32) -> Result<[f64; 16], Failure> {
        self.lock_state().derivatives(parameter, derivative)
    }

    /// Writes packed derivatives; `cache.derivatives_into(0.5, 1, &mut rows)?`.
    pub fn derivatives_into(
        &self,
        parameter: f64,
        derivative: i32,
        output: &mut [f64],
    ) -> Result<(), Failure> {
        self.lock_state()
            .derivatives_into(parameter, derivative, output)
    }

    /// Writes local packed derivatives; `cache.derivatives_local_into(0.25, 1, &mut rows)?`.
    pub fn derivatives_local_into(
        &self,
        parameter: f64,
        derivative: i32,
        output: &mut [f64],
    ) -> Result<(), Failure> {
        self.lock_state()
            .derivatives_local_into(parameter, derivative, output)
    }
}
