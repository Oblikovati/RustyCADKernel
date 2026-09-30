// Rust translation of the loop structure shared by ModelingData/TKG3d/GeomGridEval/GeomGridEval_{Circle,Ellipse,Hyperbola,Parabola}.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The conic evaluators share one loop shape: read the conic once, then per parameter
//! compute the trigonometric (or hyperbolic, or polynomial) terms once and combine
//! them with the frame. `ConicTerms` holds the per-conic formulas; the functions below
//! are the five native loops written once.
use super::{GeomGridEvalCurveD1, GeomGridEvalCurveD2, GeomGridEvalCurveD3};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkg2d::geom2d_grid_eval::grid_loops::map_grid;
use rustycad_tkmath::gp::{Point3, Vector3};

/// The formulas of one conic evaluator, on the terms of one parameter.
pub(super) trait ConicTerms {
    /// The terms computed once per parameter (`cosU, sinU`, `coshU, sinhU` or `u`).
    type Local: Copy;
    /// The terms of parameter `u`.
    fn local(&self, u: f64) -> Self::Local;
    /// The point.
    fn point(&self, local: Self::Local) -> Point3;
    /// The first derivative.
    fn d1(&self, local: Self::Local) -> Vector3;
    /// The second derivative.
    fn d2(&self, local: Self::Local) -> Vector3;
    /// The third derivative.
    fn d3(&self, local: Self::Local) -> Vector3;
    /// The derivative of order `n >= 1` at `u` (the `EvaluateGridDN` loop body).
    fn dn(&self, u: f64, n: i32) -> Vector3;
}

/// The `EvaluateGrid` loop.
pub(super) fn conic_grid<K: ConicTerms>(
    k: &K,
    params: &Array1<'_, f64>,
) -> Array1<'static, Point3> {
    map_grid(params, |u| k.point(k.local(u)))
}

/// The `EvaluateGridD1` loop.
pub(super) fn conic_grid_d1<K: ConicTerms>(
    k: &K,
    params: &Array1<'_, f64>,
) -> Array1<'static, GeomGridEvalCurveD1> {
    map_grid(params, |u| {
        let l: K::Local = k.local(u);
        GeomGridEvalCurveD1 {
            point: k.point(l),
            d1: k.d1(l),
        }
    })
}

/// The `EvaluateGridD2` loop.
pub(super) fn conic_grid_d2<K: ConicTerms>(
    k: &K,
    params: &Array1<'_, f64>,
) -> Array1<'static, GeomGridEvalCurveD2> {
    map_grid(params, |u| {
        let l: K::Local = k.local(u);
        GeomGridEvalCurveD2 {
            point: k.point(l),
            d1: k.d1(l),
            d2: k.d2(l),
        }
    })
}

/// The `EvaluateGridD3` loop.
pub(super) fn conic_grid_d3<K: ConicTerms>(
    k: &K,
    params: &Array1<'_, f64>,
) -> Array1<'static, GeomGridEvalCurveD3> {
    map_grid(params, |u| {
        let l: K::Local = k.local(u);
        GeomGridEvalCurveD3 {
            point: k.point(l),
            d1: k.d1(l),
            d2: k.d2(l),
            d3: k.d3(l),
        }
    })
}

/// The `EvaluateGridDN` loop, for `n >= 1`.
pub(super) fn conic_grid_dn<K: ConicTerms>(
    k: &K,
    params: &Array1<'_, f64>,
    n: i32,
) -> Array1<'static, Vector3> {
    map_grid(params, |u| k.dn(u, n))
}

/// The public members of a conic evaluator whose `terms(params)` reads the conic
/// (`Result<Option<K>, Failure>`, `None` for a null conic or empty parameters): the
/// native `Geometry` and the five `EvaluateGrid*` members.
macro_rules! conic_grid_members {
    ($geom:ty) => {
        /// The evaluated conic (`Geometry`); `evaluator.geometry().is_null()`.
        pub fn geometry(&self) -> &Handle<$geom> {
            &self.geom
        }
        /// Points at every parameter (`EvaluateGrid`); `evaluator.evaluate_grid(&params)?`.
        pub fn evaluate_grid(
            &self,
            params: &Array1<'_, f64>,
        ) -> Result<Array1<'static, Point3>, Failure> {
            Ok(match self.terms(params)? {
                Some(t) => conic_grid(&t, params),
                None => empty(),
            })
        }
        /// Points and D1 (`EvaluateGridD1`); `evaluator.evaluate_grid_d1(&params)?`.
        pub fn evaluate_grid_d1(
            &self,
            params: &Array1<'_, f64>,
        ) -> Result<Array1<'static, GeomGridEvalCurveD1>, Failure> {
            Ok(match self.terms(params)? {
                Some(t) => conic_grid_d1(&t, params),
                None => empty(),
            })
        }
        /// Points, D1 and D2 (`EvaluateGridD2`); `evaluator.evaluate_grid_d2(&params)?`.
        pub fn evaluate_grid_d2(
            &self,
            params: &Array1<'_, f64>,
        ) -> Result<Array1<'static, GeomGridEvalCurveD2>, Failure> {
            Ok(match self.terms(params)? {
                Some(t) => conic_grid_d2(&t, params),
                None => empty(),
            })
        }
        /// Points, D1, D2 and D3 (`EvaluateGridD3`); `evaluator.evaluate_grid_d3(&params)?`.
        pub fn evaluate_grid_d3(
            &self,
            params: &Array1<'_, f64>,
        ) -> Result<Array1<'static, GeomGridEvalCurveD3>, Failure> {
            Ok(match self.terms(params)? {
                Some(t) => conic_grid_d3(&t, params),
                None => empty(),
            })
        }
        /// Derivative of order `n` at every parameter; empty for `n < 1`, which is tested
        /// before the conic is read (`EvaluateGridDN`); `evaluator.evaluate_grid_dn(&params, 2)?`.
        pub fn evaluate_grid_dn(
            &self,
            params: &Array1<'_, f64>,
            n: i32,
        ) -> Result<Array1<'static, Vector3>, Failure> {
            if n < 1 {
                return Ok(empty());
            }
            Ok(match self.terms(params)? {
                Some(t) => conic_grid_dn(&t, params, n),
                None => empty(),
            })
        }
    };
}
pub(super) use conic_grid_members;
