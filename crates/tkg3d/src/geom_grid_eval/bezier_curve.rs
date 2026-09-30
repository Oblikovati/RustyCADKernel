// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_BezierCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_BezierCurve`: a Bezier curve on a grid through one span cache.
use super::{GeomGridEvalCurveD1, GeomGridEvalCurveD2, GeomGridEvalCurveD3};
use crate::geom::GeomBezierCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d_grid_eval::bezier_arrays::BezierCurveArrays;
use rustycad_tkg2d::geom2d_grid_eval::grid_loops::{constant_grid, empty, try_map_grid};
use rustycad_tkmath::bsplclib::Cache;
use rustycad_tkmath::gp::{Point3, Vector3};

/// Batch evaluator of a `Geom_BezierCurve` (`GeomGridEval_BezierCurve`).
///
/// `EvaluateGrid` to `EvaluateGridD3` build a `BSplCLib_Cache` of the single span at
/// `0.5` and evaluate every parameter through it; `EvaluateGridDN` calls
/// `BSplCLib::DN` on the flat knots. Quirks kept: the evaluation representation of the
/// curve is ignored, and orders above the degree give zero vectors even for a rational
/// curve, whose derivatives never vanish there;
/// `GeomGridEvalBezierCurve::new(bezier).evaluate_grid(&params)?`.
pub struct GeomGridEvalBezierCurve {
    geom: Handle<GeomBezierCurve>,
}

// The arrays of the curve, read once per call.
fn read_arrays(bezier: &GeomBezierCurve) -> Result<BezierCurveArrays<Point3>, Failure> {
    let flat_knots: Array1<'static, f64> = bezier.knot_sequence()?;
    Ok(BezierCurveArrays {
        degree: bezier.degree(),
        poles: bezier.poles(),
        weights: bezier.weights(),
        flat_knots,
    })
}

impl GeomGridEvalBezierCurve {
    /// Evaluator of `bezier` (`GeomGridEval_BezierCurve(theBezier)`); a null handle is
    /// accepted; `GeomGridEvalBezierCurve::new(bezier)`.
    pub fn new(bezier: Handle<GeomBezierCurve>) -> Self {
        Self { geom: bezier }
    }
    /// The evaluated curve (`Geometry`); `evaluator.geometry().is_null()`.
    pub fn geometry(&self) -> &Handle<GeomBezierCurve> {
        &self.geom
    }
    // The span cache when there is something to evaluate.
    fn cache(&self, params: &Array1<'_, f64>) -> Result<Option<Cache>, Failure> {
        let Some(bezier) = self.geom.get() else {
            return Ok(None);
        };
        if params.is_empty() {
            return Ok(None);
        }
        Ok(Some(read_arrays(bezier)?.cache()?))
    }
    rustycad_tkg2d::bezier_grid_methods!(
        Point3,
        GeomGridEvalCurveD1,
        GeomGridEvalCurveD2,
        GeomGridEvalCurveD3
    );
    /// Derivative of order `n` through `BSplCLib::DN`, zero above the degree; empty for
    /// `n < 1` (`EvaluateGridDN`); `evaluator.evaluate_grid_dn(&params, 2)?`.
    pub fn evaluate_grid_dn(
        &self,
        params: &Array1<'_, f64>,
        n: i32,
    ) -> Result<Array1<'static, Vector3>, Failure> {
        let Some(bezier) = self.geom.get() else {
            return Ok(empty());
        };
        if params.is_empty() || n < 1 {
            return Ok(empty());
        }
        if n > bezier.degree() {
            return Ok(constant_grid(params.len(), Vector3::new(0.0, 0.0, 0.0)));
        }
        let arrays: BezierCurveArrays<Point3> = read_arrays(bezier)?;
        try_map_grid(params, |u| arrays.dn(u, n))
    }
}
