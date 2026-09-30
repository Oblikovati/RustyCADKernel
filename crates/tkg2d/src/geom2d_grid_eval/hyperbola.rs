// Rust translation of ModelingData/TKG2d/Geom2dGridEval/Geom2dGridEval_Hyperbola.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGridEval_Hyperbola`: `P(u) = C + MajR * cosh(u) * X + MinR * sinh(u) * Y` on a grid.
use super::conic_grid::{
    ConicTerms, conic_grid, conic_grid_d1, conic_grid_d2, conic_grid_d3, conic_grid_methods,
};
use super::grid::{GridFrame, RadiiFrame, empty, map_grid};
use super::{Geom2dGridEvalCurveD1, Geom2dGridEvalCurveD2, Geom2dGridEvalCurveD3};
use crate::geom2d::Geom2dHyperbola;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Hyperbola2, Point2, Vector2};

/// Batch evaluator of a `Geom2d_Hyperbola` (`Geom2dGridEval_Hyperbola`).
///
/// The hyperbola is read once per call (`Hypr2d()`); a null hyperbola or empty parameters
/// give an empty result; `Geom2dGridEvalHyperbola::new(hyperbola).evaluate_grid(&params)?`.
pub struct Geom2dGridEvalHyperbola {
    geom: Handle<Geom2dHyperbola>,
}

// The hyperbolic terms of one parameter: the derivatives alternate between the
// (sinh, cosh) and (cosh, sinh) combinations.
struct HyperbolaTerms(RadiiFrame);
impl ConicTerms for HyperbolaTerms {
    fn pair(u: f64) -> (f64, f64) {
        (u.cosh(), u.sinh())
    }
    fn point(&self, cosh_u: f64, sinh_u: f64) -> Point2 {
        self.0.point(cosh_u, sinh_u)
    }
    fn d1(&self, cosh_u: f64, sinh_u: f64) -> Vector2 {
        odd(&self.0, cosh_u, sinh_u)
    }
    fn d2(&self, cosh_u: f64, sinh_u: f64) -> Vector2 {
        even(&self.0, cosh_u, sinh_u)
    }
    fn d3(&self, cosh_u: f64, sinh_u: f64) -> Vector2 {
        odd(&self.0, cosh_u, sinh_u)
    }
}
fn odd(h: &RadiiFrame, cosh_u: f64, sinh_u: f64) -> Vector2 {
    h.vector(sinh_u, cosh_u)
}
fn even(h: &RadiiFrame, cosh_u: f64, sinh_u: f64) -> Vector2 {
    h.vector(cosh_u, sinh_u)
}

impl Geom2dGridEvalHyperbola {
    /// Evaluator of `hyperbola` (`Geom2dGridEval_Hyperbola(theEllipse)`); a null handle is
    /// accepted; `Geom2dGridEvalHyperbola::new(hyperbola)`.
    pub fn new(hyperbola: Handle<Geom2dHyperbola>) -> Self {
        Self { geom: hyperbola }
    }
    /// The evaluated hyperbola (`Geometry`); `evaluator.geometry().is_null()`.
    pub fn geometry(&self) -> &Handle<Geom2dHyperbola> {
        &self.geom
    }
    fn coords(&self, params: &Array1<'_, f64>) -> Result<Option<HyperbolaTerms>, Failure> {
        let Some(hyperbola) = self.geom.get() else {
            return Ok(None);
        };
        if params.is_empty() {
            return Ok(None);
        }
        let hypr: Hyperbola2 = hyperbola.hypr2d()?;
        let (x_dir, y_dir) = (hypr.x_axis().direction(), hypr.y_axis().direction());
        Ok(Some(HyperbolaTerms(RadiiFrame {
            frame: GridFrame::new(hypr.location(), x_dir, y_dir),
            major: hypr.major_radius(),
            minor: hypr.minor_radius(),
        })))
    }
    conic_grid_methods!();
    /// Derivative of order `n >= 1`: `(sinh, cosh)` terms for odd `n`, `(cosh, sinh)`
    /// for even `n`; empty for `n < 1` (`EvaluateGridDN`); `evaluator.evaluate_grid_dn(&params, 3)?`.
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
        let is_odd: bool = n % 2 == 1;
        Ok(map_grid(params, |u| {
            let (c, s) = (u.cosh(), u.sinh());
            if is_odd {
                odd(&e.0, c, s)
            } else {
                even(&e.0, c, s)
            }
        }))
    }
}
