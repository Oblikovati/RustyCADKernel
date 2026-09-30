// Rust translation of CreateBezierCache2d/CreateBezierCache and the DN loops of ModelingData/TKG2d/Geom2dGridEval/Geom2dGridEval_BezierCurve.cxx and ModelingData/TKG3d/GeomGridEval/GeomGridEval_BezierCurve.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Bezier-curve data that the 2D and 3D grid evaluators read (degree, poles,
//! weights, flat knots), its single-span `BSplCLib_Cache` and the `BSplCLib::DN` call;
//! the two native files differ only in the point type.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib::{self, Cache, CurvePoint, CurveSpan};

/// The arrays that the native evaluators read by reference (`Degree()`, `Poles()`,
/// `Weights()`, `KnotSequence()`); here they are copies taken once per call;
/// `BezierCurveArrays { degree: 2, poles, weights: None, flat_knots }.cache()?`.
pub struct BezierCurveArrays<P: 'static> {
    /// The degree.
    pub degree: i32,
    /// The poles.
    pub poles: Array1<'static, P>,
    /// The weights of a rational curve.
    pub weights: Option<Array1<'static, f64>>,
    /// The flat knots `0...0 1...1`.
    pub flat_knots: Array1<'static, f64>,
}

impl<P: CurvePoint + 'static> BezierCurveArrays<P> {
    /// `CreateBezierCache`: the cache of the single span, built at 0.5;
    /// `arrays.cache()?.d0::<Point3>(0.25)?`.
    pub fn cache(&self) -> Result<Cache, Failure> {
        let weights: Option<&Array1<'static, f64>> = self.weights.as_ref();
        let mut cache: Cache =
            Cache::new(self.degree, false, &self.flat_knots, &self.poles, weights)?;
        cache.build_cache(0.5, &self.flat_knots, &self.poles, weights)?;
        Ok(cache)
    }
    /// `BSplCLib::DN(U, N, 0, Degree, false, Poles, Weights, KnotSequence, nullptr)`;
    /// `arrays.dn(0.25, 2)?`.
    pub fn dn(&self, u: f64, n: i32) -> Result<P::Vector, Failure> {
        let span = CurveSpan {
            u,
            index: 0,
            degree: self.degree,
            periodic: false,
            knots: &self.flat_knots,
            mults: None,
        };
        bsplclib::dn(&span, n, &self.poles, self.weights.as_ref())
    }
}

/// `EvaluateGrid`..`EvaluateGridD3` of the Bezier grid evaluators, through the span
/// cache of `self.cache(params)` (`None` gives an empty result); `empty` and
/// `try_map_grid` must be in scope;
/// `rustycad_tkg2d::bezier_grid_methods!(Point3, GeomGridEvalCurveD1, GeomGridEvalCurveD2, GeomGridEvalCurveD3);`.
#[macro_export]
macro_rules! bezier_grid_methods {
    ($point:ty, $d1:ident, $d2:ident, $d3:ident) => {
        /// Points at every parameter (`EvaluateGrid`); `evaluator.evaluate_grid(&params)?`.
        pub fn evaluate_grid(
            &self,
            params: &Array1<'_, f64>,
        ) -> Result<Array1<'static, $point>, Failure> {
            let Some(cache) = self.cache(params)? else {
                return Ok(empty());
            };
            try_map_grid(params, |u| cache.d0::<$point>(u))
        }
        /// Points and D1 (`EvaluateGridD1`); `evaluator.evaluate_grid_d1(&params)?`.
        pub fn evaluate_grid_d1(
            &self,
            params: &Array1<'_, f64>,
        ) -> Result<Array1<'static, $d1>, Failure> {
            let Some(cache) = self.cache(params)? else {
                return Ok(empty());
            };
            try_map_grid(params, |u| {
                let (point, d1) = cache.d1::<$point>(u)?;
                Ok($d1 { point, d1 })
            })
        }
        /// Points, D1 and D2 (`EvaluateGridD2`); `evaluator.evaluate_grid_d2(&params)?`.
        pub fn evaluate_grid_d2(
            &self,
            params: &Array1<'_, f64>,
        ) -> Result<Array1<'static, $d2>, Failure> {
            let Some(cache) = self.cache(params)? else {
                return Ok(empty());
            };
            try_map_grid(params, |u| {
                let (point, d1, d2) = cache.d2::<$point>(u)?;
                Ok($d2 { point, d1, d2 })
            })
        }
        /// Points, D1, D2 and D3 (`EvaluateGridD3`); `evaluator.evaluate_grid_d3(&params)?`.
        pub fn evaluate_grid_d3(
            &self,
            params: &Array1<'_, f64>,
        ) -> Result<Array1<'static, $d3>, Failure> {
            let Some(cache) = self.cache(params)? else {
                return Ok(empty());
            };
            try_map_grid(params, |u| {
                let (point, d1, d2, d3) = cache.d3::<$point>(u)?;
                Ok($d3 { point, d1, d2, d3 })
            })
        }
    };
}
