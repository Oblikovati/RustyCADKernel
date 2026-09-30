// Rust translation of ModelingData/TKG2d/Geom2dGridEval/Geom2dGridEval_Circle.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGridEval_Circle`: `P(u) = C + R * (cos(u) * X + sin(u) * Y)` on a grid.
use super::conic_grid::{
    ConicTerms, conic_grid, conic_grid_d1, conic_grid_d2, conic_grid_d3, conic_grid_methods,
};
use super::grid::{GridFrame, cyclic_coefficients, empty, map_grid};
use super::{Geom2dGridEvalCurveD1, Geom2dGridEvalCurveD2, Geom2dGridEvalCurveD3};
use crate::geom2d::Geom2dCircle;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Circle2, Point2, Vector2};

/// Batch evaluator of a `Geom2d_Circle` (`Geom2dGridEval_Circle`).
///
/// The circle is read once per call (`Circ2d()`, whose gp constructor is the only
/// failure source); a null circle or empty parameters give an empty result;
/// `Geom2dGridEvalCircle::new(circle).evaluate_grid(&params)?`.
pub struct Geom2dGridEvalCircle {
    geom: Handle<Geom2dCircle>,
}

// The frame and radius of the circle, with the native formulas.
struct CircleCoords {
    frame: GridFrame,
    radius: f64,
}

impl CircleCoords {
    // `R * (a * X + b * Y)`, as `aRadius * (a * aXX + b * aYX)`.
    fn vector(&self, a: f64, b: f64) -> Vector2 {
        let v: Vector2 = self.frame.combined(a, b);
        Vector2::new(self.radius * v.x(), self.radius * v.y())
    }
}

impl ConicTerms for CircleCoords {
    fn pair(u: f64) -> (f64, f64) {
        (u.cos(), u.sin())
    }
    fn point(&self, cos_u: f64, sin_u: f64) -> Point2 {
        self.frame.translated(self.vector(cos_u, sin_u))
    }
    fn d1(&self, cos_u: f64, sin_u: f64) -> Vector2 {
        self.vector(-sin_u, cos_u)
    }
    fn d2(&self, cos_u: f64, sin_u: f64) -> Vector2 {
        self.vector(-cos_u, -sin_u)
    }
    fn d3(&self, cos_u: f64, sin_u: f64) -> Vector2 {
        self.vector(sin_u, -cos_u)
    }
}

impl Geom2dGridEvalCircle {
    /// Evaluator of `circle` (`Geom2dGridEval_Circle(theCircle)`); a null handle is
    /// accepted; `Geom2dGridEvalCircle::new(circle)`.
    pub fn new(circle: Handle<Geom2dCircle>) -> Self {
        Self { geom: circle }
    }
    /// The evaluated circle (`Geometry`); `evaluator.geometry().is_null()`.
    pub fn geometry(&self) -> &Handle<Geom2dCircle> {
        &self.geom
    }
    fn coords(&self, params: &Array1<'_, f64>) -> Result<Option<CircleCoords>, Failure> {
        let Some(circle) = self.geom.get() else {
            return Ok(None);
        };
        if params.is_empty() {
            return Ok(None);
        }
        let circ: Circle2 = circle.circ2d()?;
        let axis = circ.axis();
        Ok(Some(CircleCoords {
            frame: GridFrame::new(circ.location(), axis.x_direction(), axis.y_direction()),
            radius: circ.radius(),
        }))
    }
    conic_grid_methods!();
    /// Derivative of order `n >= 1`, cyclic with period 4; empty for `n < 1`
    /// (`EvaluateGridDN`); `evaluator.evaluate_grid_dn(&params, 5)?`.
    pub fn evaluate_grid_dn(
        &self,
        params: &Array1<'_, f64>,
        n: i32,
    ) -> Result<Array1<'static, Vector2>, Failure> {
        if n < 1 {
            return Ok(empty());
        }
        let Some(c) = self.coords(params)? else {
            return Ok(empty());
        };
        Ok(map_grid(params, |u| {
            let (a, b) = cyclic_coefficients(n, u.cos(), u.sin());
            c.vector(a, b)
        }))
    }
}
