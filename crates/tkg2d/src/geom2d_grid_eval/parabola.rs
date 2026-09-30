// Rust translation of ModelingData/TKG2d/Geom2dGridEval/Geom2dGridEval_Parabola.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGridEval_Parabola`: `P(u) = C + (u^2 / 4F) * X + u * Y` on a grid.
use super::grid::{GridFrame, constant_grid, empty, map_grid};
use super::{Geom2dGridEvalCurveD1, Geom2dGridEvalCurveD2, Geom2dGridEvalCurveD3};
use crate::geom2d::Geom2dParabola;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Direction2, Parabola2, Point2, Vector2};

/// Batch evaluator of a `Geom2d_Parabola` (`Geom2dGridEval_Parabola`).
///
/// Quirk kept: Y is rebuilt as the direct normal `(-X.Y, X.X)` of the mirror axis, so
/// the grid of a parabola on an indirect frame (`Sense = false`) is mirrored against
/// `Geom2d_Parabola::D0`, which follows the stored Y direction. The coefficients are
/// `1 / (4F)` and `1 / (2F)`, so the bits can differ from `ElCLib`'s `u * u / (4F)`;
/// `Geom2dGridEvalParabola::new(parabola).evaluate_grid(&params)?`.
pub struct Geom2dGridEvalParabola {
    geom: Handle<Geom2dParabola>,
}

// The frame and the two focal coefficients (`aCoeff`, `aCoeff2`).
struct ParabolaCoords {
    frame: GridFrame,
    coeff: f64,
    coeff2: f64,
}

impl ParabolaCoords {
    fn point(&self, u: f64) -> Point2 {
        let u2_term: f64 = u * u * self.coeff;
        self.frame.offset_point(u2_term, u)
    }
    // `(u / 2F) * X + Y`, as `d1Term * aXX + aYX`.
    fn d1(&self, u: f64) -> Vector2 {
        self.frame.combined(u * self.coeff2, 1.0)
    }
    // The constant `(1 / 2F) * X`.
    fn d2(&self) -> Vector2 {
        self.frame.scaled_x(self.coeff2)
    }
}

impl Geom2dGridEvalParabola {
    /// Evaluator of `parabola` (`Geom2dGridEval_Parabola(theParabola)`); a null handle
    /// is accepted; `Geom2dGridEvalParabola::new(parabola)`.
    pub fn new(parabola: Handle<Geom2dParabola>) -> Self {
        Self { geom: parabola }
    }
    /// The evaluated parabola (`Geometry`); `evaluator.geometry().is_null()`.
    pub fn geometry(&self) -> &Handle<Geom2dParabola> {
        &self.geom
    }
    fn coords(&self, params: &Array1<'_, f64>) -> Result<Option<ParabolaCoords>, Failure> {
        let Some(parabola) = self.geom.get() else {
            return Ok(None);
        };
        if params.is_empty() {
            return Ok(None);
        }
        let parab: Parabola2 = parabola.parab2d()?;
        let x_dir: Direction2 = parab.axis_of_symmetry().direction();
        // gp_Dir2d(-X.Y, X.X) normalizes again, as natively.
        let y_dir: Direction2 = Direction2::new(-x_dir.y(), x_dir.x())?;
        let focal: f64 = parab.focal();
        Ok(Some(ParabolaCoords {
            frame: GridFrame::new(parab.location(), x_dir, y_dir),
            coeff: 1.0 / (4.0 * focal),
            coeff2: 1.0 / (2.0 * focal),
        }))
    }
    /// Points at every parameter (`EvaluateGrid`); `evaluator.evaluate_grid(&params)?`.
    pub fn evaluate_grid(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Point2>, Failure> {
        let Some(p) = self.coords(params)? else {
            return Ok(empty());
        };
        Ok(map_grid(params, |u| p.point(u)))
    }
    /// Points and D1 (`EvaluateGridD1`); `evaluator.evaluate_grid_d1(&params)?`.
    pub fn evaluate_grid_d1(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Geom2dGridEvalCurveD1>, Failure> {
        let Some(p) = self.coords(params)? else {
            return Ok(empty());
        };
        Ok(map_grid(params, |u| Geom2dGridEvalCurveD1 {
            point: p.point(u),
            d1: p.d1(u),
        }))
    }
    /// Points, D1 and the constant D2 (`EvaluateGridD2`); `evaluator.evaluate_grid_d2(&params)?`.
    pub fn evaluate_grid_d2(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Geom2dGridEvalCurveD2>, Failure> {
        let Some(p) = self.coords(params)? else {
            return Ok(empty());
        };
        let d2: Vector2 = p.d2();
        Ok(map_grid(params, |u| Geom2dGridEvalCurveD2 {
            point: p.point(u),
            d1: p.d1(u),
            d2,
        }))
    }
    /// Points, D1, the constant D2 and a zero D3 (`EvaluateGridD3`);
    /// `evaluator.evaluate_grid_d3(&params)?`.
    pub fn evaluate_grid_d3(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Geom2dGridEvalCurveD3>, Failure> {
        let Some(p) = self.coords(params)? else {
            return Ok(empty());
        };
        let d2: Vector2 = p.d2();
        Ok(map_grid(params, |u| Geom2dGridEvalCurveD3 {
            point: p.point(u),
            d1: p.d1(u),
            d2,
            d3: Vector2::new(0.0, 0.0),
        }))
    }
    /// Derivative of order `n`: D1 depends on `u`, D2 is constant, higher ones are
    /// zero; empty for `n < 1` (`EvaluateGridDN`); `evaluator.evaluate_grid_dn(&params, 2)?`.
    pub fn evaluate_grid_dn(
        &self,
        params: &Array1<'_, f64>,
        n: i32,
    ) -> Result<Array1<'static, Vector2>, Failure> {
        if n < 1 {
            return Ok(empty());
        }
        let Some(p) = self.coords(params)? else {
            return Ok(empty());
        };
        Ok(match n {
            1 => map_grid(params, |u| p.d1(u)),
            2 => constant_grid(params.len(), p.d2()),
            _ => constant_grid(params.len(), Vector2::new(0.0, 0.0)),
        })
    }
}
