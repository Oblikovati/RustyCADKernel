// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_OffsetCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_OffsetCurve`: an offset curve on a grid from the basis grid.
use super::curve::GeomGridEvalCurve;
use super::{GeomGridEvalCurveD1, GeomGridEvalCurveD2, GeomGridEvalCurveD3};
use crate::geom::offset_curve_utils::{
    adjust_derivative, calculate_d0, calculate_d1, calculate_d2, calculate_d3,
};
use crate::geom::{GeomCurve, GeomOffsetCurve};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d_grid_eval::grid_loops::{empty, result_array};
use rustycad_tkmath::gp::{Direction3, Point3, Vector3, Xyz, resolution};

/// Batch evaluator of a `Geom_OffsetCurve` (`GeomGridEval_OffsetCurve`).
///
/// The basis, offset and reference direction are read at construction. Each order
/// evaluates the basis grid one order higher through a `GeomGridEval_Curve` and
/// applies `Geom_OffsetCurveUtils`; `EvaluateGridD3` also calls the basis `DN(U, 4)` at
/// every parameter. A degenerate normal anywhere, or a basis without a specialized
/// evaluator (an eval curve), gives an empty grid; orders above 3 are the basis
/// derivatives. Quirk kept: `EvaluateGridDN` of orders 1 to 3 reads the D1..D3 grid
/// without testing it, so an empty one raises Standard_OutOfRange. The offset curve's
/// own evaluation representation is ignored;
/// `GeomGridEvalOffsetCurve::new(offset).evaluate_grid(&params)?`.
pub struct GeomGridEvalOffsetCurve {
    geom: Handle<GeomOffsetCurve>,
    basis: Handle<dyn GeomCurve>,
    offset: f64,
    direction: Direction3,
}

impl GeomGridEvalOffsetCurve {
    /// Evaluator of `offset_curve`, reading its basis, distance and direction now
    /// (`GeomGridEval_OffsetCurve(theOffset)`); a null handle gives empty grids;
    /// `GeomGridEvalOffsetCurve::new(offset_curve)`.
    pub fn new(offset_curve: Handle<GeomOffsetCurve>) -> Self {
        let (basis, offset, direction) = match offset_curve.get() {
            Some(curve) => (curve.basis_curve(), curve.offset(), curve.direction()),
            None => (Handle::default(), 0.0, Direction3::default()),
        };
        Self {
            geom: offset_curve,
            basis,
            offset,
            direction,
        }
    }
    /// The evaluated curve (`Geometry`); `evaluator.geometry().is_null()`.
    pub fn geometry(&self) -> &Handle<GeomOffsetCurve> {
        &self.geom
    }
    // The basis evaluator when there is something to evaluate.
    fn basis_eval(&self, params: &Array1<'_, f64>) -> Option<GeomGridEvalCurve<'static>> {
        if self.basis.is_null() || params.is_empty() {
            return None;
        }
        Some(GeomGridEvalCurve::from_curve(&self.basis))
    }
    fn dir(&self) -> Xyz {
        self.direction.xyz()
    }
    /// Offset points from the basis D1 grid (`EvaluateGrid`); `evaluator.evaluate_grid(&params)?`.
    pub fn evaluate_grid(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Point3>, Failure> {
        let Some(basis) = self.basis_eval(params) else {
            return Ok(empty());
        };
        let grid: Array1<'static, GeomGridEvalCurveD1> = basis.evaluate_grid_d1(params)?;
        let mut result: Array1<'static, Point3> = result_array(grid.len());
        for (slot, b) in result.iter_mut().zip(grid.iter()) {
            let mut p: Point3 = b.point;
            if !calculate_d0(&mut p, b.d1, self.dir(), self.offset, resolution()) {
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
    ) -> Result<Array1<'static, GeomGridEvalCurveD1>, Failure> {
        let Some(basis) = self.basis_eval(params) else {
            return Ok(empty());
        };
        let grid: Array1<'static, GeomGridEvalCurveD2> = basis.evaluate_grid_d2(params)?;
        let mut result: Array1<'static, GeomGridEvalCurveD1> = result_array(grid.len());
        for (slot, b) in result.iter_mut().zip(grid.iter()) {
            let (mut point, mut d1) = (b.point, b.d1);
            if !calculate_d1(
                &mut point,
                &mut d1,
                b.d2,
                self.dir(),
                self.offset,
                resolution(),
            ) {
                return Ok(empty());
            }
            *slot = GeomGridEvalCurveD1 { point, d1 };
        }
        Ok(result)
    }
    /// Offset points, D1 and D2 from the basis D3 grid, with `AdjustDerivative` at
    /// singular basis points (`EvaluateGridD2`); `evaluator.evaluate_grid_d2(&params)?`.
    pub fn evaluate_grid_d2(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, GeomGridEvalCurveD2>, Failure> {
        let Some(basis) = self.basis_eval(params) else {
            return Ok(empty());
        };
        let grid: Array1<'static, GeomGridEvalCurveD3> = basis.evaluate_grid_d3(params)?;
        let mut result: Array1<'static, GeomGridEvalCurveD2> = result_array(grid.len());
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
        b: GeomGridEvalCurveD3,
        u: f64,
    ) -> Result<Option<GeomGridEvalCurveD2>, Failure> {
        let (mut point, mut d1, mut d2, mut d3) = (b.point, b.d1, b.d2, b.d3);
        let mut is_direction_change: bool = false;
        if d1.square_magnitude() <= resolution() {
            let mut dummy_d4: Vector3 = Vector3::default();
            let derivatives = [&mut d1, &mut d2, &mut d3, &mut dummy_d4];
            is_direction_change = adjust_derivative(&*self.basis, 3, u, derivatives)?;
        }
        let derivatives = [&mut d1, &mut d2];
        let (dir, offset) = (self.dir(), self.offset);
        if !calculate_d2(
            &mut point,
            derivatives,
            d3,
            dir,
            offset,
            is_direction_change,
            resolution(),
        ) {
            return Ok(None);
        }
        Ok(Some(GeomGridEvalCurveD2 { point, d1, d2 }))
    }
    /// Offset points, D1, D2 and D3 from the basis D3 grid and the basis `DN(U, 4)`
    /// (`EvaluateGridD3`); `evaluator.evaluate_grid_d3(&params)?`.
    pub fn evaluate_grid_d3(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, GeomGridEvalCurveD3>, Failure> {
        let Some(basis) = self.basis_eval(params) else {
            return Ok(empty());
        };
        let grid: Array1<'static, GeomGridEvalCurveD3> = basis.evaluate_grid_d3(params)?;
        let mut result: Array1<'static, GeomGridEvalCurveD3> = result_array(grid.len());
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
        b: GeomGridEvalCurveD3,
        u: f64,
    ) -> Result<Option<GeomGridEvalCurveD3>, Failure> {
        let (mut point, mut d1, mut d2, mut d3) = (b.point, b.d1, b.d2, b.d3);
        let mut d4: Vector3 = self.basis.dn(u, 4)?;
        let mut is_direction_change: bool = false;
        if d1.square_magnitude() <= resolution() {
            let derivatives = [&mut d1, &mut d2, &mut d3, &mut d4];
            is_direction_change = adjust_derivative(&*self.basis, 4, u, derivatives)?;
        }
        let derivatives = [&mut d1, &mut d2, &mut d3];
        let (dir, offset) = (self.dir(), self.offset);
        if !calculate_d3(
            &mut point,
            derivatives,
            d4,
            dir,
            offset,
            is_direction_change,
            resolution(),
        ) {
            return Ok(None);
        }
        Ok(Some(GeomGridEvalCurveD3 { point, d1, d2, d3 }))
    }
    /// Derivative of order `n`: the D1..D3 grids for `n <= 3` (an empty grid raises
    /// Standard_OutOfRange), the basis grid DN above; empty for `n < 1`
    /// (`EvaluateGridDN`); `evaluator.evaluate_grid_dn(&params, 2)?`.
    pub fn evaluate_grid_dn(
        &self,
        params: &Array1<'_, f64>,
        n: i32,
    ) -> Result<Array1<'static, Vector3>, Failure> {
        if self.basis.is_null() || params.is_empty() || n < 1 {
            return Ok(empty());
        }
        let len: usize = params.len();
        match n {
            1 => checked_column(&self.evaluate_grid_d1(params)?, len, |r| r.d1),
            2 => checked_column(&self.evaluate_grid_d2(params)?, len, |r| r.d2),
            3 => checked_column(&self.evaluate_grid_d3(params)?, len, |r| r.d3),
            _ => GeomGridEvalCurve::from_curve(&self.basis).evaluate_grid_dn(params, n),
        }
    }
}

/// `aResult.SetValue(i, aGrid.Value(i).DK)` for `i` in `1..=len`: one derivative of
/// every record, where reading a record the grid does not hold raises
/// Standard_OutOfRange as the range-checked `NCollection_Array1::Value`;
/// `checked_column(&grid, 3, |r| r.d1)?`.
pub(super) fn checked_column<R>(
    grid: &Array1<'_, R>,
    len: usize,
    pick: impl Fn(&R) -> Vector3,
) -> Result<Array1<'static, Vector3>, Failure> {
    let mut result: Array1<'static, Vector3> = result_array(len);
    for (index, slot) in (1_i32..).zip(result.iter_mut()) {
        let record: &R = grid.value(index).map_err(|error| {
            Failure::out_of_range(&format!(
                "NCollection_Array1::Value({index}): {error}; expected an index in 1..={}",
                grid.len()
            ))
        })?;
        *slot = pick(record);
    }
    Ok(result)
}
