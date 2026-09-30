// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_Parabola.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_Parabola`: `P(u) = C + (u^2 / 4F) * X + u * Y` on a grid.
use super::conic::{
    ConicTerms, conic_grid, conic_grid_d1, conic_grid_d2, conic_grid_d3, conic_grid_dn,
    conic_grid_members,
};
use super::frame::GridFrame3;
use super::{GeomGridEvalCurveD1, GeomGridEvalCurveD2, GeomGridEvalCurveD3};
use crate::geom::GeomParabola;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d_grid_eval::grid_loops::empty;
use rustycad_tkmath::gp::{Parabola3, Point3, Vector3};

/// Batch evaluator of a `Geom_Parabola` (`GeomGridEval_Parabola`).
///
/// X and Y come from `XAxis()`/`YAxis()` of the parabola, so, unlike the 2D evaluator,
/// the stored frame is followed. The coefficients are `1 / (4F)` and `1 / (2F)`, so the
/// bits can differ from `ElCLib`'s `u * u / (4F)`; a zero focal gives infinite or NaN
/// coefficients, as natively; `GeomGridEvalParabola::new(parabola).evaluate_grid(&params)?`.
pub struct GeomGridEvalParabola {
    geom: Handle<GeomParabola>,
}

// The frame and the two focal coefficients (`aCoeff`, `aCoeff2`).
struct ParabolaCoords {
    frame: GridFrame3,
    coeff: f64,
    coeff2: f64,
}

impl ConicTerms for ParabolaCoords {
    type Local = f64;
    fn local(&self, u: f64) -> f64 {
        u
    }
    // `aCX + u2Term * aXX + u * aYX` with `u2Term = u * u * aCoeff`.
    fn point(&self, u: f64) -> Point3 {
        let u2_term: f64 = u * u * self.coeff;
        self.frame.offset_point(u2_term, u)
    }
    // `d1Term * aXX + aYX` with `d1Term = u * aCoeff2`.
    fn d1(&self, u: f64) -> Vector3 {
        self.frame.combined(u * self.coeff2, 1.0)
    }
    // The constant `(1 / 2F) * X`.
    fn d2(&self, _u: f64) -> Vector3 {
        self.frame.scaled_x(self.coeff2)
    }
    fn d3(&self, _u: f64) -> Vector3 {
        Vector3::new(0.0, 0.0, 0.0)
    }
    fn dn(&self, u: f64, n: i32) -> Vector3 {
        match n {
            1 => self.d1(u),
            2 => self.d2(u),
            _ => self.d3(u),
        }
    }
}

impl GeomGridEvalParabola {
    /// Evaluator of `parabola` (`GeomGridEval_Parabola(theParabola)`); a null handle is
    /// accepted; `GeomGridEvalParabola::new(parabola)`.
    pub fn new(parabola: Handle<GeomParabola>) -> Self {
        Self { geom: parabola }
    }
    fn terms(&self, params: &Array1<'_, f64>) -> Result<Option<ParabolaCoords>, Failure> {
        let Some(parabola) = self.geom.get() else {
            return Ok(None);
        };
        if params.is_empty() {
            return Ok(None);
        }
        let parab: Parabola3 = parabola.parab()?;
        let (x_dir, y_dir) = (parab.x_axis().direction(), parab.y_axis().direction());
        let focal: f64 = parab.focal();
        Ok(Some(ParabolaCoords {
            frame: GridFrame3::new(parab.location(), x_dir, y_dir),
            coeff: 1.0 / (4.0 * focal),
            coeff2: 1.0 / (2.0 * focal),
        }))
    }
    conic_grid_members!(GeomParabola);
}
