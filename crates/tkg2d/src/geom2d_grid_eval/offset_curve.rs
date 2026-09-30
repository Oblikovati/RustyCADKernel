// Rust translation of ModelingData/TKG2d/Geom2dGridEval/Geom2dGridEval_OffsetCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGridEval_OffsetCurve`: an offset curve on a grid from the basis grid.
use super::curve::Geom2dGridEvalCurve;
use super::grid::{column, empty, result_array};
use super::{Geom2dGridEvalCurveD1, Geom2dGridEvalCurveD2, Geom2dGridEvalCurveD3};
use crate::geom2d::offset_curve_utils::{
    adjust_derivative, calculate_d0, calculate_d1, calculate_d2, calculate_d3,
};
use crate::geom2d::{Geom2dCurve, Geom2dOffsetCurve};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Point2, Vector2, resolution};

/// Batch evaluator of a `Geom2d_OffsetCurve` (`Geom2dGridEval_OffsetCurve`).
///
/// The basis and offset are read at construction. Each order evaluates the basis grid
/// one order higher through a `Geom2dGridEval_Curve` and applies
/// `Geom2d_OffsetCurveUtils`; `EvaluateGridD3` also calls the basis `EvalDN(U, 4)` at
/// every parameter. A degenerate normal anywhere, or a basis without a specialized
/// evaluator (an eval curve), gives an empty grid; orders above 3 are the basis
/// derivatives. The offset curve's own evaluation representation is ignored;
/// `Geom2dGridEvalOffsetCurve::new(offset).evaluate_grid(&params)?`.
pub struct Geom2dGridEvalOffsetCurve {
    geom: Handle<Geom2dOffsetCurve>,
    basis: Handle<dyn Geom2dCurve>,
    offset: f64,
}

impl Geom2dGridEvalOffsetCurve {
    /// Evaluator of `offset_curve`, reading its basis and distance now
    /// (`Geom2dGridEval_OffsetCurve(theOffset)`); a null handle gives empty grids;
    /// `Geom2dGridEvalOffsetCurve::new(offset_curve)`.
    pub fn new(offset_curve: Handle<Geom2dOffsetCurve>) -> Self {
        let (basis, offset) = match offset_curve.get() {
            Some(curve) => (curve.basis_curve(), curve.offset()),
            None => (Handle::default(), 0.0),
        };
        Self {
            geom: offset_curve,
            basis,
            offset,
        }
    }
    /// The evaluated curve (`Geometry`); `evaluator.geometry().is_null()`.
    pub fn geometry(&self) -> &Handle<Geom2dOffsetCurve> {
        &self.geom
    }
    // The basis evaluator when there is something to evaluate.
    fn basis_eval(&self, params: &Array1<'_, f64>) -> Option<Geom2dGridEvalCurve<'static>> {
        if self.basis.is_null() || params.is_empty() {
            return None;
        }
        Some(Geom2dGridEvalCurve::from_curve(&self.basis))
    }
    /// Offset points from the basis D1 grid (`EvaluateGrid`); `evaluator.evaluate_grid(&params)?`.
    pub fn evaluate_grid(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Point2>, Failure> {
        let Some(basis) = self.basis_eval(params) else {
            return Ok(empty());
        };
        let grid: Array1<'static, Geom2dGridEvalCurveD1> = basis.evaluate_grid_d1(params)?;
        let mut result: Array1<'static, Point2> = result_array(grid.len());
        for (slot, b) in result.iter_mut().zip(grid.iter()) {
            let mut p: Point2 = b.point;
            if !calculate_d0(&mut p, b.d1, self.offset)? {
                return Ok(empty());
            }
            *slot = p;
        }
        Ok(result)
    }
    /// Offset points and D1 from the basis D2 grid (`EvaluateGridD1`);
    /// `evaluator.evaluate_grid_d1(&params)?`.
    pub fn evaluate_grid_d1(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Geom2dGridEvalCurveD1>, Failure> {
        let Some(basis) = self.basis_eval(params) else {
            return Ok(empty());
        };
        let grid: Array1<'static, Geom2dGridEvalCurveD2> = basis.evaluate_grid_d2(params)?;
        let mut result: Array1<'static, Geom2dGridEvalCurveD1> = result_array(grid.len());
        for (slot, b) in result.iter_mut().zip(grid.iter()) {
            let (mut point, mut d1) = (b.point, b.d1);
            if !calculate_d1(&mut point, &mut d1, b.d2, self.offset) {
                return Ok(empty());
            }
            *slot = Geom2dGridEvalCurveD1 { point, d1 };
        }
        Ok(result)
    }
    /// Offset points, D1 and D2 from the basis D3 grid, with `AdjustDerivative` at
    /// singular basis points (`EvaluateGridD2`); `evaluator.evaluate_grid_d2(&params)?`.
    pub fn evaluate_grid_d2(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Geom2dGridEvalCurveD2>, Failure> {
        let Some(basis) = self.basis_eval(params) else {
            return Ok(empty());
        };
        let grid: Array1<'static, Geom2dGridEvalCurveD3> = basis.evaluate_grid_d3(params)?;
        let mut result: Array1<'static, Geom2dGridEvalCurveD2> = result_array(grid.len());
        for ((slot, b), &u) in result.iter_mut().zip(grid.iter()).zip(params.iter()) {
            let Some(value) = self.offset_d2(*b, u)? else {
                return Ok(empty());
            };
            *slot = value;
        }
        Ok(result)
    }
    fn offset_d2(
        &self,
        b: Geom2dGridEvalCurveD3,
        u: f64,
    ) -> Result<Option<Geom2dGridEvalCurveD2>, Failure> {
        let (mut point, mut d1, mut d2, mut d3) = (b.point, b.d1, b.d2, b.d3);
        let mut is_direction_change: bool = false;
        if d1.square_magnitude() <= resolution() {
            let mut dummy_d4: Vector2 = Vector2::default();
            let derivatives = [&mut d1, &mut d2, &mut d3, &mut dummy_d4];
            is_direction_change = adjust_derivative(&*self.basis, 3, u, derivatives)?;
        }
        let offset = self.offset;
        if !calculate_d2(
            &mut point,
            &mut d1,
            &mut d2,
            d3,
            is_direction_change,
            offset,
        ) {
            return Ok(None);
        }
        Ok(Some(Geom2dGridEvalCurveD2 { point, d1, d2 }))
    }
    /// Offset points, D1, D2 and D3 from the basis D3 grid and the basis `EvalDN(U, 4)`
    /// (`EvaluateGridD3`); `evaluator.evaluate_grid_d3(&params)?`.
    pub fn evaluate_grid_d3(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Geom2dGridEvalCurveD3>, Failure> {
        let Some(basis) = self.basis_eval(params) else {
            return Ok(empty());
        };
        let grid: Array1<'static, Geom2dGridEvalCurveD3> = basis.evaluate_grid_d3(params)?;
        let mut result: Array1<'static, Geom2dGridEvalCurveD3> = result_array(grid.len());
        for ((slot, b), &u) in result.iter_mut().zip(grid.iter()).zip(params.iter()) {
            let Some(value) = self.offset_d3(*b, u)? else {
                return Ok(empty());
            };
            *slot = value;
        }
        Ok(result)
    }
    fn offset_d3(
        &self,
        b: Geom2dGridEvalCurveD3,
        u: f64,
    ) -> Result<Option<Geom2dGridEvalCurveD3>, Failure> {
        let (mut point, mut d1, mut d2, mut d3) = (b.point, b.d1, b.d2, b.d3);
        let mut d4: Vector2 = self.basis.eval_dn(u, 4)?;
        let mut is_direction_change: bool = false;
        if d1.square_magnitude() <= resolution() {
            let derivatives = [&mut d1, &mut d2, &mut d3, &mut d4];
            is_direction_change = adjust_derivative(&*self.basis, 4, u, derivatives)?;
        }
        let derivatives = [&mut d1, &mut d2, &mut d3];
        if !calculate_d3(
            &mut point,
            derivatives,
            d4,
            is_direction_change,
            self.offset,
        ) {
            return Ok(None);
        }
        Ok(Some(Geom2dGridEvalCurveD3 { point, d1, d2, d3 }))
    }
    /// Derivative of order `n`: the D1..D3 grids for `n <= 3`, the basis grid DN above;
    /// empty for `n < 1` (`EvaluateGridDN`); `evaluator.evaluate_grid_dn(&params, 2)?`.
    pub fn evaluate_grid_dn(
        &self,
        params: &Array1<'_, f64>,
        n: i32,
    ) -> Result<Array1<'static, Vector2>, Failure> {
        if self.basis.is_null() || params.is_empty() || n < 1 {
            return Ok(empty());
        }
        match n {
            1 => Ok(column(&self.evaluate_grid_d1(params)?, |r| r.d1)),
            2 => Ok(column(&self.evaluate_grid_d2(params)?, |r| r.d2)),
            3 => Ok(column(&self.evaluate_grid_d3(params)?, |r| r.d3)),
            _ => Geom2dGridEvalCurve::from_curve(&self.basis).evaluate_grid_dn(params, n),
        }
    }
}
