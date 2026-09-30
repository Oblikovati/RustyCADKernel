// Rust translation of ModelingData/TKG2d/Geom2dGridEval/Geom2dGridEval_BSplineCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGridEval_BSplineCurve`: a B-spline curve on a grid, with a span cache for
//! runs of parameters in the same span.
use super::bspline_spans::{SplineArrays, evaluate_cached};
use super::grid::{constant_grid, empty, try_map_grid};
use super::{Geom2dGridEvalCurveD1, Geom2dGridEvalCurveD2, Geom2dGridEvalCurveD3};
use crate::geom2d::Geom2dBSplineCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{Point2, Vector2};

/// Batch evaluator of a `Geom2d_BSplineCurve` (`Geom2dGridEval_BSplineCurve`).
///
/// Each parameter is located in the flat knots; a run of at least four consecutive
/// parameters in one span goes through a `BSplCLib_Cache` built for that span, shorter
/// runs through `BSplCLib::D0`..`D3` directly. `EvaluateGridDN` calls `BSplCLib::DN`
/// with the knots and multiplicities but passes the span index found in the flat
/// knots: `PrepareEval` keeps an index that lies in the knot index range, so for such
/// indices the derivative is the polynomial of another span (quirk kept). When that
/// index is the last knot index of a non-periodic curve, the native `BuildKnots` reads
/// past the knot and multiplicity arrays (undefined); the port raises
/// Standard_OutOfRange there. The evaluation representation of the curve is ignored;
/// `Geom2dGridEvalBSplineCurve::new(bspline).evaluate_grid(&params)?`.
pub struct Geom2dGridEvalBSplineCurve {
    geom: Handle<Geom2dBSplineCurve>,
}

impl Geom2dGridEvalBSplineCurve {
    /// Evaluator of `bspline` (`Geom2dGridEval_BSplineCurve(theCurve)`); a null handle
    /// is accepted; `Geom2dGridEvalBSplineCurve::new(bspline)`.
    pub fn new(bspline: Handle<Geom2dBSplineCurve>) -> Self {
        Self { geom: bspline }
    }
    /// The evaluated curve (`Geometry`); `evaluator.geometry().is_null()`.
    pub fn geometry(&self) -> &Handle<Geom2dBSplineCurve> {
        &self.geom
    }
    // `extractCurveData` after the empty-parameter test.
    fn arrays(&self, params: &Array1<'_, f64>) -> Option<SplineArrays> {
        if params.is_empty() {
            return None;
        }
        Some(SplineArrays::read(self.geom.get()?))
    }
    /// Points at every parameter (`EvaluateGrid`); `evaluator.evaluate_grid(&params)?`.
    pub fn evaluate_grid(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Point2>, Failure> {
        let Some(data) = self.arrays(params) else {
            return Ok(empty());
        };
        evaluate_cached(
            &data,
            params,
            |cache, local| cache.d0_local::<Point2>(local),
            |data, u, span| bsplclib::d0(&data.flat_span(u, span), &data.poles, data.weights()),
        )
    }
    /// Points and D1 (`EvaluateGridD1`); `evaluator.evaluate_grid_d1(&params)?`.
    pub fn evaluate_grid_d1(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Geom2dGridEvalCurveD1>, Failure> {
        let Some(data) = self.arrays(params) else {
            return Ok(empty());
        };
        let pack = |(point, d1)| Geom2dGridEvalCurveD1 { point, d1 };
        evaluate_cached(
            &data,
            params,
            |cache, local| cache.d1_local::<Point2>(local).map(pack),
            |data, u, span| {
                bsplclib::d1(&data.flat_span(u, span), &data.poles, data.weights()).map(pack)
            },
        )
    }
    /// Points, D1 and D2 (`EvaluateGridD2`); `evaluator.evaluate_grid_d2(&params)?`.
    pub fn evaluate_grid_d2(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Geom2dGridEvalCurveD2>, Failure> {
        let Some(data) = self.arrays(params) else {
            return Ok(empty());
        };
        let pack = |(point, d1, d2)| Geom2dGridEvalCurveD2 { point, d1, d2 };
        evaluate_cached(
            &data,
            params,
            |cache, local| cache.d2_local::<Point2>(local).map(pack),
            |data, u, span| {
                bsplclib::d2(&data.flat_span(u, span), &data.poles, data.weights()).map(pack)
            },
        )
    }
    /// Points, D1, D2 and D3 (`EvaluateGridD3`); `evaluator.evaluate_grid_d3(&params)?`.
    pub fn evaluate_grid_d3(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Geom2dGridEvalCurveD3>, Failure> {
        let Some(data) = self.arrays(params) else {
            return Ok(empty());
        };
        let pack = |(point, d1, d2, d3)| Geom2dGridEvalCurveD3 { point, d1, d2, d3 };
        evaluate_cached(
            &data,
            params,
            |cache, local| cache.d3_local::<Point2>(local).map(pack),
            |data, u, span| {
                bsplclib::d3(&data.flat_span(u, span), &data.poles, data.weights()).map(pack)
            },
        )
    }
    /// Derivative of order `n` through `BSplCLib::DN`, zero above the degree; empty for
    /// `n < 1` (`EvaluateGridDN`); `evaluator.evaluate_grid_dn(&params, 2)?`.
    pub fn evaluate_grid_dn(
        &self,
        params: &Array1<'_, f64>,
        n: i32,
    ) -> Result<Array1<'static, Vector2>, Failure> {
        if params.is_empty() || n < 1 {
            return Ok(empty());
        }
        let Some(data) = self.arrays(params) else {
            return Ok(empty());
        };
        if n > data.degree {
            return Ok(constant_grid(params.len(), Vector2::new(0.0, 0.0)));
        }
        try_map_grid(params, |u| {
            let (adjusted, span) = data.locate_span(u)?;
            bsplclib::dn(
                &data.knot_span(adjusted, span),
                n,
                &data.poles,
                data.weights(),
            )
        })
    }
}
