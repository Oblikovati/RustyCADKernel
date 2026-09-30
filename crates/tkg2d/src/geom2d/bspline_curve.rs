// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BSplineCurve.hxx/.cxx, Geom2d_BSplineCurve_1.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_BSplineCurve`: rational or non-rational, periodic or not, B-spline curves.
mod access;
mod poles;
mod traits;
use super::arrays::at;
use super::bounded_curve::Geom2dBoundedCurve;
use super::bspline_curve_core::GivenWeights;
use super::curve::Geom2dCurve;
use super::errors;
use super::geometry::Geom2dGeometry;
use super::spline_data::check_curve_data;
use super::spline_kind::BSplineState;
use crate::geom2d_eval::{RepCurveDesc, rep_utils};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{Point2, Vector2, resolution};

/// A B-spline curve; `Handle::new(Geom2dBSplineCurve::new(&poles, &knots, &mults, 3, false)?)`.
pub struct Geom2dBSplineCurve {
    parent: StandardTransient,
    state: SharedState<BSplineState>,
}
impl_transient!(
    Geom2dBSplineCurve,
    "Geom2d_BSplineCurve",
    dyn Geom2dBoundedCurve,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve, dyn Geom2dBoundedCurve]
);

// The weight checks of the rational constructor.
fn check_weights(weights: &Array1<'_, f64>, nbpoles: usize) -> Result<(), Failure> {
    if weights.len() != nbpoles {
        return Err(errors::construction(&format!(
            "Geom2d_BSplineCurve: {} weights for {nbpoles} poles; Weights and Poles array size mismatch",
            weights.len()
        )));
    }
    for i in weights.lower()..=weights.upper() {
        let weight: f64 = at(weights, i)?;
        if weight <= resolution() {
            return Err(errors::construction(&format!(
                "Geom2d_BSplineCurve: Weights({i}) = {weight}; Weights values too small"
            )));
        }
    }
    Ok(())
}

impl Geom2dBSplineCurve {
    crate::bspline_curve_edits!(Point2, Vector2);
    fn from_state(state: BSplineState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// Non-rational curve (`Geom2d_BSplineCurve(Poles, Knots, Multiplicities, Degree, Periodic)`);
    /// Standard_ConstructionError for inconsistent data;
    /// `Geom2dBSplineCurve::new(&poles, &knots, &mults, 3, false)?`.
    pub fn new(
        poles: &Array1<'_, Point2>,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        degree: i32,
        periodic: bool,
    ) -> Result<Self, Failure> {
        check_curve_data(poles, knots, mults, degree, periodic)?;
        let state = BSplineState::from_arrays(poles, None, knots, mults, degree, periodic)?;
        Ok(Self::from_state(state))
    }
    /// Rational curve (`Geom2d_BSplineCurve(Poles, Weights, Knots, Multiplicities, Degree, Periodic)`);
    /// equal weights give a non-rational curve;
    /// `Geom2dBSplineCurve::with_weights(&poles, &weights, &knots, &mults, 2, false)?`.
    pub fn with_weights(
        poles: &Array1<'_, Point2>,
        weights: &Array1<'_, f64>,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        degree: i32,
        periodic: bool,
    ) -> Result<Self, Failure> {
        check_curve_data(poles, knots, mults, degree, periodic)?;
        check_weights(weights, poles.len())?;
        let given = GivenWeights {
            weights,
            check_rational: true,
        };
        let state = BSplineState::from_arrays(poles, Some(given), knots, mults, degree, periodic)?;
        Ok(Self::from_state(state))
    }
    /// The native copy constructor: the evaluation representation is deep-copied, the
    /// cached resolution value is kept but marked stale
    /// (`Geom2d_BSplineCurve(const Geom2d_BSplineCurve&)`); `Geom2dBSplineCurve::copy_of(&curve)?`.
    pub fn copy_of(other: &Geom2dBSplineCurve) -> Result<Self, Failure> {
        let mut copy: BSplineState = other.state.snapshot();
        copy.eval_rep = rep_utils::clone_curve_desc(&copy.eval_rep)?;
        if !copy.rational {
            copy.reset_unit_weights()?;
        }
        copy.max_deriv_inv_ok = false;
        Ok(Self::from_state(copy))
    }
    crate::eval_representation_members!(dyn RepCurveDesc, rep_utils::validate_curve_desc, "curve");
    /// Insert knot `u` with multiplicity `mult` (`InsertKnot(U, M, ParametricTolerance)`);
    /// `curve.insert_knot(0.5, 1, 0.0)?`.
    pub fn insert_knot(&self, u: f64, mult: i32, parametric_tolerance: f64) -> Result<(), Failure> {
        let mut knots: Array1<'static, f64> = super::arrays::one_based(1)?;
        super::arrays::put(&mut knots, 1, u)?;
        let mut mults: Array1<'static, i32> = super::arrays::one_based(1)?;
        super::arrays::put(&mut mults, 1, mult)?;
        self.insert_knots(&knots, &mults, parametric_tolerance, false)
    }
    /// Insert a pole after `index` on a uniform or quasi-uniform curve (`InsertPoleAfter`);
    /// `curve.insert_pole_after(2, p, 1.0)?`.
    pub fn insert_pole_after(&self, index: i32, pole: Point2, weight: f64) -> Result<(), Failure> {
        self.state.write().insert_pole_after(index, pole, weight)
    }
    /// Insert a pole before `index` (`InsertPoleBefore`); `curve.insert_pole_before(1, p, 1.0)?`.
    pub fn insert_pole_before(&self, index: i32, pole: Point2, weight: f64) -> Result<(), Failure> {
        self.insert_pole_after(index - 1, pole, weight)
    }
    /// Remove the pole at `index` on a uniform or quasi-uniform curve (`RemovePole`);
    /// `curve.remove_pole(2)?`.
    pub fn remove_pole(&self, index: i32) -> Result<(), Failure> {
        self.state.write().remove_pole(index)
    }
}
