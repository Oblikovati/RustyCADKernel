// Rust translation of ModelingData/TKG2d/Geom2dGridEval/Geom2dGridEval_Line.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGridEval_Line`: `P(t) = Location + t * Direction` on a parameter grid.
use super::grid::{constant_grid, empty, map_grid};
use super::{Geom2dGridEvalCurveD1, Geom2dGridEvalCurveD2, Geom2dGridEvalCurveD3};
use crate::geom2d::Geom2dLine;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Point2, Vector2};

/// Batch evaluator of a `Geom2d_Line` (`Geom2dGridEval_Line`).
///
/// A null line or an empty parameter array gives an empty result; results are indexed
/// from 1 whatever the bounds of the parameters;
/// `Geom2dGridEvalLine::new(line).evaluate_grid(&params)`.
pub struct Geom2dGridEvalLine {
    geom: Handle<Geom2dLine>,
}

// The pre-extracted location and direction coordinates (`aLocX`...`aDirY`).
struct LineCoords {
    loc_x: f64,
    loc_y: f64,
    dir_x: f64,
    dir_y: f64,
}

impl LineCoords {
    fn point(&self, t: f64) -> Point2 {
        Point2::new(self.loc_x + t * self.dir_x, self.loc_y + t * self.dir_y)
    }
    fn d1(&self) -> Vector2 {
        Vector2::new(self.dir_x, self.dir_y)
    }
}

impl Geom2dGridEvalLine {
    /// Evaluator of `line` (`Geom2dGridEval_Line(theLine)`); a null handle is accepted;
    /// `Geom2dGridEvalLine::new(line)`.
    pub fn new(line: Handle<Geom2dLine>) -> Self {
        Self { geom: line }
    }
    /// The evaluated line (`Geometry`); `evaluator.geometry().is_null()`.
    pub fn geometry(&self) -> &Handle<Geom2dLine> {
        &self.geom
    }
    // The coordinates of a loaded line when there is something to evaluate.
    fn coords(&self, params: &Array1<'_, f64>) -> Option<LineCoords> {
        let line: &Geom2dLine = self.geom.get()?;
        if params.is_empty() {
            return None;
        }
        let position = line.lin2d().position();
        let (location, direction) = (position.location(), position.direction());
        Some(LineCoords {
            loc_x: location.x(),
            loc_y: location.y(),
            dir_x: direction.x(),
            dir_y: direction.y(),
        })
    }
    /// Points at every parameter (`EvaluateGrid`); `evaluator.evaluate_grid(&params)`.
    pub fn evaluate_grid(&self, params: &Array1<'_, f64>) -> Array1<'static, Point2> {
        let Some(c) = self.coords(params) else {
            return empty();
        };
        map_grid(params, |t| c.point(t))
    }
    /// Points and the constant first derivative (`EvaluateGridD1`);
    /// `evaluator.evaluate_grid_d1(&params)`.
    pub fn evaluate_grid_d1(
        &self,
        params: &Array1<'_, f64>,
    ) -> Array1<'static, Geom2dGridEvalCurveD1> {
        let Some(c) = self.coords(params) else {
            return empty();
        };
        map_grid(params, |t| Geom2dGridEvalCurveD1 {
            point: c.point(t),
            d1: c.d1(),
        })
    }
    /// Points, D1 and a zero D2 (`EvaluateGridD2`); `evaluator.evaluate_grid_d2(&params)`.
    pub fn evaluate_grid_d2(
        &self,
        params: &Array1<'_, f64>,
    ) -> Array1<'static, Geom2dGridEvalCurveD2> {
        let Some(c) = self.coords(params) else {
            return empty();
        };
        map_grid(params, |t| Geom2dGridEvalCurveD2 {
            point: c.point(t),
            d1: c.d1(),
            d2: Vector2::new(0.0, 0.0),
        })
    }
    /// Points, D1 and zero D2 and D3 (`EvaluateGridD3`); `evaluator.evaluate_grid_d3(&params)`.
    pub fn evaluate_grid_d3(
        &self,
        params: &Array1<'_, f64>,
    ) -> Array1<'static, Geom2dGridEvalCurveD3> {
        let Some(c) = self.coords(params) else {
            return empty();
        };
        let zero = Vector2::new(0.0, 0.0);
        map_grid(params, |t| Geom2dGridEvalCurveD3 {
            point: c.point(t),
            d1: c.d1(),
            d2: zero,
            d3: zero,
        })
    }
    /// Derivative of order `n` at every parameter: the direction for `n == 1`, zero
    /// above; empty for `n < 1` (`EvaluateGridDN`); `evaluator.evaluate_grid_dn(&params, 2)`.
    pub fn evaluate_grid_dn(&self, params: &Array1<'_, f64>, n: i32) -> Array1<'static, Vector2> {
        if n < 1 {
            return empty();
        }
        let Some(c) = self.coords(params) else {
            return empty();
        };
        if n == 1 {
            return constant_grid(params.len(), c.d1());
        }
        constant_grid(params.len(), Vector2::new(0.0, 0.0))
    }
}
