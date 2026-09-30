// Rust translation of ProjLib_OnPlane and PerformApprox (ProjLib_ProjectOnPlane.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The B-spline approximation of a projection on a plane: the continuous function
//! `ProjLib_OnPlane` fitted by `Approx_FitAndDivide`, joined into one B-spline and
//! smoothed by knot removal.
use super::eval::Oblique;
use crate::app_cont::{Function, FunctionBase};
use crate::app_par_curves::{MultiCurve, MultiCurveEval};
use crate::approx::{ComputeCLineOptions, FitAndDivide};
use crate::failure_capture::FailureCapture;
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use std::cell::RefCell;

/// `ProjLib_OnPlane`: the projected curve as one 3D point function.
pub(super) struct OnPlaneFunction<'a> {
    base: FunctionBase,
    curve: &'a dyn Adaptor3dCurve,
    oblique: Oblique,
    failure: RefCell<FailureCapture>,
}

impl<'a> OnPlaneFunction<'a> {
    pub(super) fn new(curve: &'a dyn Adaptor3dCurve, oblique: Oblique) -> Self {
        let mut base: FunctionBase = FunctionBase::new();
        base.set_number_of_points(1, 0);
        Self {
            base,
            curve,
            oblique,
            failure: RefCell::new(FailureCapture::default()),
        }
    }
    fn keep<T>(&self, value: Result<T, Failure>) -> Option<T> {
        self.failure.borrow_mut().keep(value)
    }
}

impl Function for OnPlaneFunction<'_> {
    fn base(&self) -> &FunctionBase {
        &self.base
    }
    // The native bounds cannot fail; a failing adaptor yields NaN, which the
    // failure check after the fit reports.
    fn first_parameter(&self) -> f64 {
        self.keep(self.curve.first_parameter()).unwrap_or(f64::NAN)
    }
    fn last_parameter(&self) -> f64 {
        self.keep(self.curve.last_parameter()).unwrap_or(f64::NAN)
    }
    fn value(&self, u: f64, _pnt2d: &mut Array1<'_, Point2>, pnt: &mut Array1<'_, Point3>) -> bool {
        let Some(point) = self.keep(self.oblique.value(self.curve, u)) else {
            return false;
        };
        pnt[1] = point;
        true
    }
    fn d1(&self, u: f64, _vec2d: &mut Array1<'_, Vector2>, vec: &mut Array1<'_, Vector3>) -> bool {
        let Some((_, vector)) = self.keep(self.oblique.d1(self.curve, u)) else {
            return false;
        };
        vec[1] = vector;
        true
    }
}

// The degrees and the segment count of the fit for a curve kind.
fn fit_setup(curve_type: CurveType) -> (i32, i32, i32) {
    match curve_type {
        CurveType::Parabola => (2, 2, 100),
        CurveType::Hyperbola => (14, 14, 1000),
        _ => (8, 8, 100),
    }
}

/// `PerformApprox(C, Pl, D, BSplineCurvePtr)`: the fitted B-spline, or a null
/// handle when the fit does not approximate the whole range (the native output
/// handle is then left untouched); `perform_approx(curve, oblique)?`.
pub(super) fn perform_approx(
    curve: &dyn Adaptor3dCurve,
    oblique: Oblique,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let function: OnPlaneFunction<'_> = OnPlaneFunction::new(curve, oblique);
    let (deg1, deg2, nb_segm) = fit_setup(curve.get_type()?);
    let options = ComputeCLineOptions {
        degree_min: deg1,
        degree_max: deg2,
        tolerance_3d: precision::approximation(),
        tolerance_2d: precision::p_approximation(),
        cutting: true,
        ..ComputeCLineOptions::default()
    };
    let mut fit: FitAndDivide = FitAndDivide::new(options);
    fit.set_max_segments(nb_segm);
    let fitted = fit.perform(&function);
    function.failure.borrow_mut().take()?;
    fitted?;
    if !fit.is_all_approximated() {
        return Ok(Handle::default());
    }
    join_pieces(&fit)
}

// The Bezier pieces raised to the highest degree, joined with interior
// multiplicity MaxDeg, then smoothed by knot removal within the largest error.
fn join_pieces(fit: &FitAndDivide) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let nb_curves: i32 = fit.nb_multi_curves();
    let mut max_deg: i32 = 0;
    for i in 1..=nb_curves {
        max_deg = max_deg.max(fit.value(i)?.degree()?);
    }
    let nb_poles: i32 = max_deg * nb_curves + 1;
    let mut poles: Array1<'static, Point3> = new_array(nb_poles)?;
    let mut knots: Array1<'static, f64> = new_array(nb_curves + 1)?;
    let mut err_max: f64 = 0.;
    let mut compt: i32 = 1;
    for i in 1..=nb_curves {
        let (first, last) = fit.parameters(i)?;
        (knots[i], knots[i + 1]) = (first, last);
        err_max = err_max.max(fit.error(i)?.0);
        let piece: Array1<'static, Point3> = piece_poles(&fit.value(i)?, max_deg)?;
        for j in 1..=max_deg + 1 {
            poles[compt] = piece[j];
            compt += 1;
        }
        compt -= 1;
    }
    let mults: Array1<'static, i32> = interior_mults(nb_curves + 1, max_deg)?;
    let curve = GeomBSplineCurve::new(&poles, &knots, &mults, max_deg, false)?;
    smooth(&curve, nb_curves + 1, max_deg, err_max)?;
    Ok(Handle::new(curve))
}

// The poles of one Bezier piece, raised to `max_deg` when its degree is lower.
fn piece_poles(mc: &MultiCurve, max_deg: i32) -> Result<Array1<'static, Point3>, Failure> {
    let degree: i32 = mc.degree()?;
    let mut local: Array1<'static, Point3> = new_array(degree + 1)?;
    mc.curve_3d(1, &mut local)?;
    if max_deg <= degree {
        return Ok(local);
    }
    let mut raised: Array1<'static, Point3> = new_array(max_deg + 1)?;
    bsplclib::increase_degree_bezier(max_deg, &local, None, &mut raised, None)?;
    Ok(raised)
}

// Multiplicities MaxDeg + 1 at both ends and MaxDeg inside.
fn interior_mults(nb_knots: i32, max_deg: i32) -> Result<Array1<'static, i32>, Failure> {
    let mut mults: Array1<'static, i32> = new_array(nb_knots)?;
    mults[1] = max_deg + 1;
    for i in 2..nb_knots {
        mults[i] = max_deg;
    }
    mults[nb_knots] = max_deg + 1;
    Ok(mults)
}

// "Try to smooth": each interior knot of multiplicity MaxDeg is lowered to
// MaxDeg - 1 when that stays within the fit error; failures are ignored.
fn smooth(
    curve: &GeomBSplineCurve,
    nb_knots: i32,
    max_deg: i32,
    tolerance: f64,
) -> Result<(), Failure> {
    for i in 2..nb_knots {
        if curve.multiplicity(i)? == max_deg {
            curve.remove_knot(i, max_deg - 1, tolerance)?;
        }
    }
    Ok(())
}

/// A 1-based array of `length` default values.
pub(crate) fn new_array<T: Clone + Default>(length: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(1, length).map_err(|error| array_failure(error, "ProjLib"))
}
