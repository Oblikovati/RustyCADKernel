// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BezierCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_BezierCurve`: a rational or non-rational Bezier curve on `[0, 1]`.
use super::arrays::at;
use super::bounded_curve::{Geom2dBoundedCurve, dump_bounded_curve};
use super::curve::{Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3};
use super::geometry::Geom2dGeometry;
use super::spline_kind::BezierState;
use crate::geom2d_eval::{RepCurveDesc, rep_utils};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{Point2, Transform2, Vector2};

/// A Bezier curve; `Handle::new(Geom2dBezierCurve::new(&poles)?)`.
pub struct Geom2dBezierCurve {
    parent: StandardTransient,
    state: SharedState<BezierState>,
}
impl_transient!(
    Geom2dBezierCurve,
    "Geom2d_BezierCurve",
    dyn Geom2dBoundedCurve,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve, dyn Geom2dBoundedCurve]
);

impl Geom2dBezierCurve {
    crate::bezier_curve_members!(Point2);
    fn from_state(state: BezierState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// Non-rational curve (`Geom2d_BezierCurve(CurvePoles)`); 2 to MaxDegree()+1 poles;
    /// `Geom2dBezierCurve::new(&poles)?`.
    pub fn new(poles: &Array1<'_, Point2>) -> Result<Self, Failure> {
        Ok(Self::from_state(BezierState::checked(poles, None)?))
    }
    /// Rational curve (`Geom2d_BezierCurve(CurvePoles, PoleWeights)`); equal weights give a
    /// non-rational curve; `Geom2dBezierCurve::with_weights(&poles, &weights)?`.
    pub fn with_weights(
        poles: &Array1<'_, Point2>,
        weights: &Array1<'_, f64>,
    ) -> Result<Self, Failure> {
        Ok(Self::from_state(BezierState::checked(
            poles,
            Some(weights),
        )?))
    }
    /// The native copy constructor with its failures (`Geom2d_BezierCurve(const Geom2d_BezierCurve&)`);
    /// `Geom2dBezierCurve::copy_of(&curve)?`.
    pub fn copy_of(other: &Geom2dBezierCurve) -> Result<Self, Failure> {
        let mut copy: BezierState = other.state.snapshot();
        copy.eval_rep = rep_utils::clone_curve_desc(&copy.eval_rep)?;
        copy.reset_copied()?;
        Ok(Self::from_state(copy))
    }
    crate::eval_representation_members!(dyn RepCurveDesc, rep_utils::validate_curve_desc, "curve");
    /// Insert a pole after position `index` in 0..=NbPoles (`InsertPoleAfter`);
    /// `curve.insert_pole_after(2, p, 1.0)?`.
    pub fn insert_pole_after(&self, index: i32, pole: Point2, weight: f64) -> Result<(), Failure> {
        self.state.write().insert_pole_after(index, pole, weight)
    }
    /// Insert a pole before position `index` (`InsertPoleBefore`); `curve.insert_pole_before(1, p, 1.0)?`.
    pub fn insert_pole_before(&self, index: i32, pole: Point2, weight: f64) -> Result<(), Failure> {
        self.insert_pole_after(index - 1, pole, weight)
    }
}
impl Geom2dGeometry for Geom2dBezierCurve {
    crate::spline_geometry_members!(
        Transform2,
        dyn Geom2dGeometry,
        "Geom2d_BezierCurve",
        dump_bounded_curve
    );
}
impl Geom2dCurve for Geom2dBezierCurve {
    fn reverse(&self) -> Result<(), Failure> {
        self.state.write().reverse();
        Ok(())
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        Ok(1. - u)
    }
    fn first_parameter(&self) -> f64 {
        0.0
    }
    fn last_parameter(&self) -> f64 {
        1.0
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        Ok(self.state.read().closed)
    }
    crate::non_periodic_cn_curve!();
    crate::spline_state_evaluators!(Point2, Vector2, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3);
}
impl Geom2dBoundedCurve for Geom2dBezierCurve {
    fn end_point(&self) -> Result<Point2, Failure> {
        let state = self.state.read();
        at(&state.poles, state.poles.upper())
    }
    fn start_point(&self) -> Result<Point2, Failure> {
        at(&self.state.read().poles, 1)
    }
}
