// Rust translation of ModelingData/TKG2d/Geom2dGridEval/Geom2dGridEval_OtherCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGridEval_OtherCurve`: the fallback grid evaluator through an adaptor.
use super::grid::{column, empty, try_map_grid};
use super::{Geom2dGridEvalCurveD1, Geom2dGridEvalCurveD2, Geom2dGridEvalCurveD3};
use crate::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Vector2};

/// Grid evaluation through the `EvalD0`..`EvalDN` members of any adaptor
/// (`Geom2dGridEval_OtherCurve`).
///
/// The evaluator borrows the adaptor, as the native `std::reference_wrapper`, so the
/// adaptor must outlive it; an adaptor failure aborts the grid;
/// `Geom2dGridEvalOtherCurve::new(&adaptor).evaluate_grid(&params)?`.
pub struct Geom2dGridEvalOtherCurve<'a> {
    curve: &'a dyn Adaptor2dCurve2d,
}

impl<'a> Geom2dGridEvalOtherCurve<'a> {
    /// Evaluator over `curve` (`Geom2dGridEval_OtherCurve(theCurve)`);
    /// `Geom2dGridEvalOtherCurve::new(&adaptor)`.
    pub fn new(curve: &'a dyn Adaptor2dCurve2d) -> Self {
        Self { curve }
    }
    /// The evaluated adaptor (`Curve`); `evaluator.curve().get_type()?`.
    pub fn curve(&self) -> &'a dyn Adaptor2dCurve2d {
        self.curve
    }
    /// `EvalD0` at every parameter (`EvaluateGrid`); `evaluator.evaluate_grid(&params)?`.
    pub fn evaluate_grid(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Point2>, Failure> {
        if params.is_empty() {
            return Ok(empty());
        }
        try_map_grid(params, |u| self.curve.eval_d0(u))
    }
    /// `EvalD1` at every parameter (`EvaluateGridD1`); `evaluator.evaluate_grid_d1(&params)?`.
    pub fn evaluate_grid_d1(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Geom2dGridEvalCurveD1>, Failure> {
        if params.is_empty() {
            return Ok(empty());
        }
        try_map_grid(params, |u| self.curve.eval_d1(u))
    }
    /// `EvalD2` at every parameter (`EvaluateGridD2`); `evaluator.evaluate_grid_d2(&params)?`.
    pub fn evaluate_grid_d2(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Geom2dGridEvalCurveD2>, Failure> {
        if params.is_empty() {
            return Ok(empty());
        }
        try_map_grid(params, |u| self.curve.eval_d2(u))
    }
    /// `EvalD3` at every parameter (`EvaluateGridD3`); `evaluator.evaluate_grid_d3(&params)?`.
    pub fn evaluate_grid_d3(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Geom2dGridEvalCurveD3>, Failure> {
        if params.is_empty() {
            return Ok(empty());
        }
        try_map_grid(params, |u| self.curve.eval_d3(u))
    }
    /// Derivative of order `n`: the D1..D3 grids for `n <= 3`, `EvalDN` above; empty
    /// for `n < 1` (`EvaluateGridDN`); `evaluator.evaluate_grid_dn(&params, 4)?`.
    pub fn evaluate_grid_dn(
        &self,
        params: &Array1<'_, f64>,
        n: i32,
    ) -> Result<Array1<'static, Vector2>, Failure> {
        if params.is_empty() || n < 1 {
            return Ok(empty());
        }
        match n {
            1 => Ok(column(&self.evaluate_grid_d1(params)?, |r| r.d1)),
            2 => Ok(column(&self.evaluate_grid_d2(params)?, |r| r.d2)),
            3 => Ok(column(&self.evaluate_grid_d3(params)?, |r| r.d3)),
            _ => try_map_grid(params, |u| self.curve.eval_dn(u, n)),
        }
    }
}
