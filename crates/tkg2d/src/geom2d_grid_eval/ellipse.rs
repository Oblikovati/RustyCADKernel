// Rust translation of ModelingData/TKG2d/Geom2dGridEval/Geom2dGridEval_Ellipse.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGridEval_Ellipse`: `P(u) = C + MajR * cos(u) * X + MinR * sin(u) * Y` on a grid.
use super::conic_grid::{
    ConicTerms, conic_grid, conic_grid_d1, conic_grid_d2, conic_grid_d3, conic_grid_methods,
};
use super::grid::{GridFrame, RadiiFrame, cyclic_coefficients, empty, map_grid};
use super::{Geom2dGridEvalCurveD1, Geom2dGridEvalCurveD2, Geom2dGridEvalCurveD3};
use crate::geom2d::Geom2dEllipse;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Ellipse2, Point2, Vector2};

/// Batch evaluator of a `Geom2d_Ellipse` (`Geom2dGridEval_Ellipse`).
///
/// The ellipse is read once per call (`Elips2d()`); a null ellipse or empty parameters
/// give an empty result; `Geom2dGridEvalEllipse::new(ellipse).evaluate_grid(&params)?`.
pub struct Geom2dGridEvalEllipse {
    geom: Handle<Geom2dEllipse>,
}

// The trigonometric terms of one parameter.
struct EllipseTerms(RadiiFrame);
impl ConicTerms for EllipseTerms {
    fn pair(u: f64) -> (f64, f64) {
        (u.cos(), u.sin())
    }
    fn point(&self, cos_u: f64, sin_u: f64) -> Point2 {
        self.0.point(cos_u, sin_u)
    }
    fn d1(&self, cos_u: f64, sin_u: f64) -> Vector2 {
        self.0.vector(-sin_u, cos_u)
    }
    fn d2(&self, cos_u: f64, sin_u: f64) -> Vector2 {
        self.0.vector(-cos_u, -sin_u)
    }
    fn d3(&self, cos_u: f64, sin_u: f64) -> Vector2 {
        self.0.vector(sin_u, -cos_u)
    }
}

impl Geom2dGridEvalEllipse {
    /// Evaluator of `ellipse` (`Geom2dGridEval_Ellipse(theEllipse)`); a null handle is
    /// accepted; `Geom2dGridEvalEllipse::new(ellipse)`.
    pub fn new(ellipse: Handle<Geom2dEllipse>) -> Self {
        Self { geom: ellipse }
    }
    /// The evaluated ellipse (`Geometry`); `evaluator.geometry().is_null()`.
    pub fn geometry(&self) -> &Handle<Geom2dEllipse> {
        &self.geom
    }
    fn coords(&self, params: &Array1<'_, f64>) -> Result<Option<EllipseTerms>, Failure> {
        let Some(ellipse) = self.geom.get() else {
            return Ok(None);
        };
        if params.is_empty() {
            return Ok(None);
        }
        let elips: Ellipse2 = ellipse.elips2d()?;
        let (x_dir, y_dir) = (elips.x_axis().direction(), elips.y_axis().direction());
        Ok(Some(EllipseTerms(RadiiFrame {
            frame: GridFrame::new(elips.location(), x_dir, y_dir),
            major: elips.major_radius(),
            minor: elips.minor_radius(),
        })))
    }
    conic_grid_methods!();
    /// Derivative of order `n >= 1`, cyclic with period 4; empty for `n < 1`
    /// (`EvaluateGridDN`); `evaluator.evaluate_grid_dn(&params, 6)?`.
    pub fn evaluate_grid_dn(
        &self,
        params: &Array1<'_, f64>,
        n: i32,
    ) -> Result<Array1<'static, Vector2>, Failure> {
        if n < 1 {
            return Ok(empty());
        }
        let Some(e) = self.coords(params)? else {
            return Ok(empty());
        };
        Ok(map_grid(params, |u| {
            let (a, b) = cyclic_coefficients(n, u.cos(), u.sin());
            e.0.vector(a, b)
        }))
    }
}
