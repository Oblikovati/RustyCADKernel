// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_OtherCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_OtherCurve`: the fallback grid evaluator through an adaptor.
use super::{GeomGridEvalCurveD1, GeomGridEvalCurveD2, GeomGridEvalCurveD3};
use crate::adaptor3d::Adaptor3dCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d_grid_eval::grid_loops::{column, empty, try_map_grid};
use rustycad_tkmath::gp::{Point3, Vector3};

/// Grid evaluation through the non-virtual `Value`, `D1`..`D3` and `DN` members of any
/// adaptor, which call its `EvalD0`..`EvalDN` (`GeomGridEval_OtherCurve`).
///
/// The evaluator borrows the adaptor, as the native `std::reference_wrapper`, so the
/// adaptor must outlive it; an adaptor failure aborts the grid;
/// `GeomGridEvalOtherCurve::new(&adaptor).evaluate_grid(&params)?`.
pub struct GeomGridEvalOtherCurve<'a> {
    curve: &'a dyn Adaptor3dCurve,
}

impl<'a> GeomGridEvalOtherCurve<'a> {
    /// Evaluator over `curve` (`GeomGridEval_OtherCurve(theCurve)`);
    /// `GeomGridEvalOtherCurve::new(&adaptor)`.
    pub fn new(curve: &'a dyn Adaptor3dCurve) -> Self {
        Self { curve }
    }
    /// The evaluated adaptor (`Curve`); `evaluator.curve().get_type()?`.
    pub fn curve(&self) -> &'a dyn Adaptor3dCurve {
        self.curve
    }
    /// `Value` at every parameter (`EvaluateGrid`); `evaluator.evaluate_grid(&params)?`.
    pub fn evaluate_grid(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Point3>, Failure> {
        if params.is_empty() {
            return Ok(empty());
        }
        try_map_grid(params, |u| self.curve.value(u))
    }
    /// `D1` at every parameter (`EvaluateGridD1`); `evaluator.evaluate_grid_d1(&params)?`.
    pub fn evaluate_grid_d1(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, GeomGridEvalCurveD1>, Failure> {
        if params.is_empty() {
            return Ok(empty());
        }
        try_map_grid(params, |u| {
            let (point, d1) = self.curve.d1(u)?;
            Ok(GeomGridEvalCurveD1 { point, d1 })
        })
    }
    /// `D2` at every parameter (`EvaluateGridD2`); `evaluator.evaluate_grid_d2(&params)?`.
    pub fn evaluate_grid_d2(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, GeomGridEvalCurveD2>, Failure> {
        if params.is_empty() {
            return Ok(empty());
        }
        try_map_grid(params, |u| {
            let (point, d1, d2) = self.curve.d2(u)?;
            Ok(GeomGridEvalCurveD2 { point, d1, d2 })
        })
    }
    /// `D3` at every parameter (`EvaluateGridD3`); `evaluator.evaluate_grid_d3(&params)?`.
    pub fn evaluate_grid_d3(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, GeomGridEvalCurveD3>, Failure> {
        if params.is_empty() {
            return Ok(empty());
        }
        try_map_grid(params, |u| {
            let (point, d1, d2, d3) = self.curve.d3(u)?;
            Ok(GeomGridEvalCurveD3 { point, d1, d2, d3 })
        })
    }
    /// Derivative of order `n`: the D1..D3 grids for `n <= 3`, `DN` above; empty for
    /// `n < 1` (`EvaluateGridDN`); `evaluator.evaluate_grid_dn(&params, 4)?`.
    pub fn evaluate_grid_dn(
        &self,
        params: &Array1<'_, f64>,
        n: i32,
    ) -> Result<Array1<'static, Vector3>, Failure> {
        if params.is_empty() || n < 1 {
            return Ok(empty());
        }
        match n {
            1 => Ok(column(&self.evaluate_grid_d1(params)?, |r| r.d1)),
            2 => Ok(column(&self.evaluate_grid_d2(params)?, |r| r.d2)),
            3 => Ok(column(&self.evaluate_grid_d3(params)?, |r| r.d3)),
            _ => try_map_grid(params, |u| self.curve.dn(u, n)),
        }
    }
}
