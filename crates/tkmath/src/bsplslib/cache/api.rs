use super::Cache;
use crate::bsplslib::SurfaceD2;
use crate::gp::{Point3, Vector3};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;

impl Cache {
    /// Tests whether the parameters lie in the cached patch; `cache.is_cache_valid(0.5, 0.5)`.
    pub fn is_cache_valid(&self, u: f64, v: f64) -> bool {
        self.lock_state().is_cache_valid(u, v)
    }

    /// Recomputes the cached patch; `cache.build_cache(0.5, 0.5, &u, &v, &poles, None)?`.
    pub fn build_cache(
        &self,
        u: f64,
        v: f64,
        ku: &Array1<'_, f64>,
        kv: &Array1<'_, f64>,
        poles: &Array2<'_, Point3>,
        weights: Option<&Array2<'_, f64>>,
    ) -> Result<(), Failure> {
        self.lock_state().build_cache(u, v, ku, kv, poles, weights)
    }

    /// Evaluates a point; `cache.d0(0.5, 0.5)?`.
    pub fn d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        self.lock_state().d0(u, v)
    }

    /// Evaluates normalized patch parameters; `cache.d0_local(0.0, 0.0)?`.
    pub fn d0_local(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        self.lock_state().d0_local(u, v)
    }

    /// Evaluates a point and tangents; `cache.d1(0.5, 0.5)?`.
    pub fn d1(&self, u: f64, v: f64) -> Result<(Point3, Vector3, Vector3), Failure> {
        self.lock_state().d1(u, v)
    }

    /// Evaluates normalized parameters and tangents; `cache.d1_local(0.0, 0.0)?`.
    pub fn d1_local(&self, u: f64, v: f64) -> Result<(Point3, Vector3, Vector3), Failure> {
        self.lock_state().d1_local(u, v)
    }

    /// Evaluates point and second derivatives; `cache.d2(0.5, 0.5)?`.
    pub fn d2(&self, u: f64, v: f64) -> Result<SurfaceD2, Failure> {
        self.lock_state().d2(u, v)
    }

    /// Evaluates normalized patch parameters through second order; `cache.d2_local(0.0, 0.0)?`.
    pub fn d2_local(&self, u: f64, v: f64) -> Result<SurfaceD2, Failure> {
        self.lock_state().d2_local(u, v)
    }
}
