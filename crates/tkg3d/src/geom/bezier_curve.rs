// Rust translation of ModelingData/TKG3d/Geom/Geom_BezierCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_BezierCurve`: a rational or non-rational Bezier curve on `[0, 1]`.
use super::bounded_curve::{GeomBoundedCurve, dump_bounded_curve};
use super::curve::{GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3};
use super::geometry::GeomGeometry;
use super::spline_kind::BezierState;
use crate::geom_eval::{RepCurveDesc, rep_utils};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::{Failure, StandardTransient};
use rustycad_tkg2d::geom2d::arrays::at;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};

/// A Bezier curve; `Handle::new(GeomBezierCurve::new(&poles)?)`.
pub struct GeomBezierCurve {
    parent: StandardTransient,
    state: SharedState<BezierState>,
}
impl_transient!(
    GeomBezierCurve,
    "Geom_BezierCurve",
    dyn GeomBoundedCurve,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomCurve, dyn GeomBoundedCurve]
);

impl GeomBezierCurve {
    rustycad_tkg2d::bezier_curve_members!(Point3);
    fn from_state(state: BezierState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// Non-rational curve (`Geom_BezierCurve(CurvePoles)`); 2 to MaxDegree()+1 poles;
    /// `GeomBezierCurve::new(&poles)?`.
    pub fn new(poles: &Array1<'_, Point3>) -> Result<Self, Failure> {
        Ok(Self::from_state(BezierState::checked(poles, None)?))
    }
    /// Rational curve (`Geom_BezierCurve(CurvePoles, PoleWeights)`); equal weights give a
    /// non-rational curve; `GeomBezierCurve::with_weights(&poles, &weights)?`.
    pub fn with_weights(
        poles: &Array1<'_, Point3>,
        weights: &Array1<'_, f64>,
    ) -> Result<Self, Failure> {
        Ok(Self::from_state(BezierState::checked(
            poles,
            Some(weights),
        )?))
    }
    /// The native copy constructor (`Geom_BezierCurve(const Geom_BezierCurve&)`): the
    /// evaluation representation is deep-copied; `GeomBezierCurve::copy_of(&curve)?`.
    pub fn copy_of(other: &GeomBezierCurve) -> Result<Self, Failure> {
        let mut copy: BezierState = other.state.snapshot();
        copy.eval_rep = rep_utils::clone_curve_desc(&copy.eval_rep)?;
        copy.reset_copied()?;
        Ok(Self::from_state(copy))
    }
    rustycad_tkg2d::eval_representation_members!(
        dyn RepCurveDesc,
        rep_utils::validate_curve_desc,
        "curve"
    );
    /// Insert a pole of weight 1 after position `index` in 0..=NbPoles
    /// (`InsertPoleAfter(Index, P)`); `curve.insert_pole_after(2, p)?`.
    pub fn insert_pole_after(&self, index: i32, pole: Point3) -> Result<(), Failure> {
        self.insert_pole_after_weighted(index, pole, 1.)
    }
    /// Insert a weighted pole after position `index` (`InsertPoleAfter(Index, P, Weight)`);
    /// `curve.insert_pole_after_weighted(2, p, 2.0)?`.
    pub fn insert_pole_after_weighted(
        &self,
        index: i32,
        pole: Point3,
        weight: f64,
    ) -> Result<(), Failure> {
        self.state.write().insert_pole_after(index, pole, weight)
    }
    /// Insert a pole of weight 1 before position `index` (`InsertPoleBefore(Index, P)`);
    /// `curve.insert_pole_before(1, p)?`.
    pub fn insert_pole_before(&self, index: i32, pole: Point3) -> Result<(), Failure> {
        self.insert_pole_after(index - 1, pole)
    }
    /// Insert a weighted pole before position `index` (`InsertPoleBefore(Index, P, Weight)`);
    /// `curve.insert_pole_before_weighted(1, p, 2.0)?`.
    pub fn insert_pole_before_weighted(
        &self,
        index: i32,
        pole: Point3,
        weight: f64,
    ) -> Result<(), Failure> {
        self.insert_pole_after_weighted(index - 1, pole, weight)
    }
}
impl GeomGeometry for GeomBezierCurve {
    rustycad_tkg2d::spline_geometry_members!(
        Transform3,
        dyn GeomGeometry,
        "Geom_BezierCurve",
        dump_bounded_curve
    );
}
impl GeomCurve for GeomBezierCurve {
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
    rustycad_tkg2d::non_periodic_cn_curve!();
    rustycad_tkg2d::spline_state_evaluators!(
        Point3,
        Vector3,
        GeomCurveD1,
        GeomCurveD2,
        GeomCurveD3
    );
}
impl GeomBoundedCurve for GeomBezierCurve {
    fn end_point(&self) -> Result<Point3, Failure> {
        let state = self.state.read();
        at(&state.poles, state.poles.upper())
    }
    fn start_point(&self) -> Result<Point3, Failure> {
        at(&self.state.read().poles, 1)
    }
}
