// Rust translation of ModelingData/TKG3d/Geom/Geom_BSplineCurve.cxx, Geom_BSplineCurve_1.cxx and Geom_BezierCurve.cxx (PeriodicNormalization, RemoveKnot index check, closure test, evaluation representation) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Geom members of the spline curve states that Geom2d shares: B-spline edits
//! clear the GeomEval evaluation representation, `PeriodicNormalization` is a closed
//! form, `RemoveKnot` keeps the end knots of a non-periodic curve and the Bezier
//! closure test uses Precision::Confusion (Geom2d: gp::Resolution).
use crate::geom::errors;
use crate::geom_eval::{RepCurveDesc, rep_utils};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, real};
use rustycad_tkg2d::geom2d::arrays::at;
use rustycad_tkg2d::geom2d::bezier_curve_core::{BezierCurveCore, BezierCurveKind};
use rustycad_tkg2d::geom2d::bspline_curve_core::{BSplineCurveCore, BSplineCurveKind};
use rustycad_tkg2d::geom2d::spline_curve_kind::{KindD1, KindD2, KindD3, SplineCurveKind};
use rustycad_tkmath::gp::{Point3, Vector3, resolution};

/// The Geom variant of the spline curve kinds.
#[derive(Clone, Copy, Debug)]
pub(super) struct GeomSplineKind;

/// The fields of Geom_BSplineCurve.
pub(super) type BSplineState = BSplineCurveCore<GeomSplineKind>;
/// The fields of Geom_BezierCurve.
pub(super) type BezierState = BezierCurveCore<GeomSplineKind>;

type RepHandle = Handle<dyn RepCurveDesc>;

impl SplineCurveKind for GeomSplineKind {
    type Point = Point3;
    type EvalRep = RepHandle;
    fn undefined_derivative(message: &str) -> Failure {
        errors::undefined_derivative(message)
    }
    fn rep_d0(eval_rep: &RepHandle, u: f64) -> Result<Option<Point3>, Failure> {
        rep_utils::try_eval_curve_d0(eval_rep, u)
    }
    fn rep_d1(eval_rep: &RepHandle, u: f64) -> Result<Option<KindD1<Self>>, Failure> {
        Ok(rep_utils::try_eval_curve_d1(eval_rep, u)?.map(|r| (r.point, r.d1)))
    }
    fn rep_d2(eval_rep: &RepHandle, u: f64) -> Result<Option<KindD2<Self>>, Failure> {
        Ok(rep_utils::try_eval_curve_d2(eval_rep, u)?.map(|r| (r.point, r.d1, r.d2)))
    }
    fn rep_d3(eval_rep: &RepHandle, u: f64) -> Result<Option<KindD3<Self>>, Failure> {
        Ok(rep_utils::try_eval_curve_d3(eval_rep, u)?.map(|r| (r.point, r.d1, r.d2, r.d3)))
    }
    fn rep_dn(eval_rep: &RepHandle, u: f64, n: i32) -> Result<Option<Vector3>, Failure> {
        rep_utils::try_eval_curve_dn(eval_rep, u, n)
    }
}

impl BezierCurveKind for GeomSplineKind {
    const BEZIER_CLASS: &'static str = "Geom_BezierCurve";
    fn point_distance(first: Point3, last: Point3) -> f64 {
        first.distance(&last)
    }
    // Unlike Geom2d_BezierCurve (gp::Resolution), the 3D class tests closure against
    // Precision::Confusion in init, Segment and SetPole.
    fn closure_tolerance() -> f64 {
        precision::confusion()
    }
}

impl BSplineCurveKind for GeomSplineKind {
    const BSPLINE_CLASS: &'static str = "Geom_BSplineCurve";
    // `ClearEvalRepresentation`, which most 3D edits call (unlike Geom2d).
    fn clear_eval_rep_on_edit(eval_rep: &mut RepHandle) {
        *eval_rep = Handle::default();
    }
    // Shift a periodic parameter into the curve range by a whole number of periods,
    // then by one period, stepping to the neighbouring double when the shift is
    // absorbed by rounding. Geom_BSplineCurve replaces the loops of Geom2d_BSplineCurve
    // by this closed form, so an infinite parameter gives NaN instead of looping.
    fn periodic_normalization(state: &BSplineState, parameter: f64) -> Result<f64, Failure> {
        if !state.periodic {
            return Ok(parameter);
        }
        let first: f64 = at(&state.flat_knots, state.deg + 1)?;
        let last: f64 = at(&state.flat_knots, state.flat_knots.upper() - state.deg)?;
        if parameter >= first && parameter <= last {
            return Ok(parameter);
        }
        let period: f64 = last - first;
        let nb_periods: f64 = ((parameter - first) / period).floor();
        let mut value: f64 = parameter;
        if nb_periods != 0.0 {
            value -= nb_periods * period;
        }
        if value > last {
            return Ok(shifted(value, value - period, first));
        }
        if value < first {
            return Ok(shifted(value, value + period, last));
        }
        Ok(value)
    }
    // Strictly interior for a non-periodic curve.
    fn check_removable_index(state: &BSplineState, index: i32) -> Result<(), Failure> {
        let (i1, i2) = (state.first_u_knot_index()?, state.last_u_knot_index()?);
        let outside: bool = if state.periodic {
            index < i1 || index > i2
        } else {
            index <= i1 || index >= i2
        };
        if outside {
            return Err(errors::out_of_range(&format!(
                "BSpline curve: RemoveKnot: index {index} out of range; expected in \
                 ({i1}, {i2}) or [{i1}, {i2}] when periodic"
            )));
        }
        Ok(())
    }
    fn tangents_agree(v1: Vector3, v2: Vector3, ang_tol: f64) -> Result<bool, Failure> {
        if v1.square_magnitude() <= resolution() || v2.square_magnitude() <= resolution() {
            return Ok(false);
        }
        Ok(v1.angle(v2)?.abs() <= ang_tol)
    }
}

// `aNewParam == theParameter ? std::nextafter(aNewParam, toward) : aNewParam`; the
// candidate equals a finite value there, so C `nextafter` never sees a NaN.
fn shifted(value: f64, candidate: f64, toward: f64) -> f64 {
    if candidate == value {
        return real::next_after(candidate, toward);
    }
    candidate
}
