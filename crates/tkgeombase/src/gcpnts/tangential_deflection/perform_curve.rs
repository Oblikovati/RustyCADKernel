// Rust translation of `getIntervalIdx`, `GCPnts_TangentialDeflection::EvaluateDu` and
// `GCPnts_TangentialDeflection::PerformCurve` of
// ModelingData/TKGeomBase/GCPnts/GCPnts_TangentialDeflection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `PerformCurve`: the adaptive march of `GCPnts_TangentialDeflection` on a general
//! curve, then the refinement of its spans.
mod march;
mod refine;

use super::{TangentialDeflection, US3};
use crate::cpnts::PntsCurve;
use crate::gcpnts::PoleCurve;
use crate::gcpnts::TCurveTypes;
use crate::gcpnts::intervals::{bound, interval_bounds};
use crate::gcpnts::t_curve_types::live;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Point3, Xyz};

/// The state of `PerformCurve` shared by its phases: the current span `[U1, U2]`, the
/// step and its saved value, whether EvaluateDu never succeeded, the curve points,
/// the CN intervals (clamped to the range by the line test) and their current
/// indexes, and the last point retained.
struct CurveMarch {
    u1: f64,
    u2: f64,
    du: f64,
    dusave: f64,
    not_done: bool,
    current_point: Point3,
    last_point: Point3,
    intervs: Array1<'static, f64>,
    idx: [i32; 2],
    is_need_to_check: bool,
    prev_point: Point3,
    more_points: bool,
}

/// `getIntervalIdx`: the first interval from `previous` containing `param`, or the
/// upper bound of the array when none does.
fn interval_idx(param: f64, intervs: &Array1<'_, f64>, previous: i32) -> Result<i32, Failure> {
    let mut idx: i32 = previous;
    while idx < intervs.upper() {
        // Inside of idx interval.
        if param >= bound(intervs, idx)? && param <= bound(intervs, idx + 1)? {
            break;
        }
        idx += 1;
    }
    Ok(idx)
}

fn difference(first: Point3, second: Point3) -> Xyz {
    first.xyz() - second.xyz()
}

impl TangentialDeflection {
    /// `EvaluateDu`: the point at `u` and, when the tangent, the second derivative and
    /// the curvature are not below Precision::Confusion(), the step
    /// `sqrt(8 max(CurvatureDeflection, MinLen) / curvature)` with `not_done` cleared
    /// (both left unchanged otherwise).
    fn evaluate_du(
        &self,
        curve: PntsCurve<'_>,
        u: f64,
        du: &mut f64,
        not_done: &mut bool,
    ) -> Result<Point3, Failure> {
        let (point, t, n) = curve.lifted_d2(u)?;
        let lt: f64 = t.magnitude();
        let l_tol: f64 = precision::confusion();
        if lt > l_tol && n.magnitude() > l_tol {
            let lc: f64 = n.cross_magnitude(t);
            let ln: f64 = lc / lt;
            if ln > l_tol {
                *du = (8.0 * std_max(self.my_curvature_deflection, self.my_min_len) / ln).sqrt();
                *not_done = false;
            }
        }
        Ok(point)
    }

    /// `PerformCurve`: the march with the deflection and angle criteria, preceded by
    /// the test for a line or a singular start, followed by the refinement.
    pub(super) fn perform_curve<C: TCurveTypes + ?Sized>(
        &mut self,
        curve: &C,
    ) -> Result<(), Failure> {
        let pnts: PntsCurve = curve.pnts_curve();
        let mut state: CurveMarch = self.start_march(pnts)?;
        if state.not_done || state.du > 5.0 * state.dusave {
            // It is either a line or a singularity.
            if self.line_or_singularity(curve, &mut state)? {
                return Ok(());
            }
        }
        if state.du > state.dusave {
            state.du = state.dusave;
        } else {
            state.dusave = state.du;
        }
        if state.du < self.my_u_tol {
            state.du = self.my_last_u - self.my_firstu;
            if state.du < self.my_u_tol {
                self.my_points.append(self.my_last_u, state.last_point);
                return Ok(());
            }
        }
        self.march(pnts, &mut state)?;
        self.refine(curve)
    }

    // The initialization of the computation: the first point and its step, a third of
    // the range when EvaluateDu fails, and the CN intervals.
    fn start_march(&mut self, curve: PntsCurve<'_>) -> Result<CurveMarch, Failure> {
        let last_point: Point3 = curve.lifted_d0(self.my_last_u)?;
        let mut not_done: bool = true;
        let dusave: f64 = (self.my_last_u - self.my_firstu) * US3;
        let mut du: f64 = dusave;
        let current_point: Point3 =
            self.evaluate_du(curve, self.my_firstu, &mut du, &mut not_done)?;
        self.my_points.append(self.my_firstu, current_point);
        // Used to detect "isLine" current bspline and in Du computation in general handling.
        let intervs: Array1<'static, f64> = interval_bounds(curve, Shape::CN)?;
        let lower: i32 = intervs.lower();
        Ok(CurveMarch {
            u1: self.my_firstu,
            u2: self.my_firstu,
            du,
            dusave,
            not_done,
            current_point,
            last_point,
            intervs,
            idx: [lower, lower],
            is_need_to_check: false,
            prev_point: current_point,
            more_points: true,
        })
    }

    // A null or huge first step: a line gets the linear distribution; a singular start
    // takes the step at the last tested parameter; a zero-length curve ends with its
    // two ends. True when the distribution is complete.
    fn line_or_singularity<C: TCurveTypes + ?Sized>(
        &mut self,
        curve: &C,
        state: &mut CurveMarch,
    ) -> Result<bool, Failure> {
        let pnts: PntsCurve = curve.pnts_curve();
        let v1: Xyz = difference(state.last_point, state.current_point);
        let l1: f64 = v1.modulus();
        if l1 > precision::confusion() {
            let (is_line, param) = self.line_test(curve, state, (v1, l1))?;
            if is_line {
                self.my_points.clear();
                self.perform_linear(pnts)?;
                return Ok(true);
            }
            // It was a singularity, continue:
            self.evaluate_du(pnts, param, &mut state.du, &mut state.not_done)?;
            return Ok(false);
        }
        state.du = (self.my_last_u - self.my_firstu) / 2.1;
        let middle_u: f64 = self.my_firstu + state.du;
        let middle_point: Point3 = pnts.lifted_d0(middle_u)?;
        if difference(middle_point, state.current_point).modulus() < precision::confusion() {
            // L1 < LTol: this is a zero-length curve, computation is finished:
            // return a segment of 2 points (protection)
            self.my_points.append(self.my_last_u, state.last_point);
            return Ok(true);
        }
        Ok(false)
    }

    // "If it is a line, we verify by computing minNbPoints": the angles between the
    // chord to the end and the chords to max(MinNbPnts, 3, degree + 1) points of each
    // CN interval meeting the range (its bounds clamped to the range in place) stay
    // below ATol; returns the verdict and the last tested parameter.
    fn line_test<C: TCurveTypes + ?Sized>(
        &self,
        curve: &C,
        state: &mut CurveMarch,
        (v1, l1): (Xyz, f64),
    ) -> Result<(bool, f64), Failure> {
        let nb_points: i32 = self.line_test_points(curve)?;
        let a_tol: f64 = angular_tolerance(self.my_angular_deflection);
        let (mut is_line, mut param) = (true, 0.0);
        let nb_interv: i32 = state.intervs.upper() - 1;
        let mut i: i32 = 1;
        while i <= nb_interv && is_line {
            if let Some((low, delta)) = self.clamp_interval(&mut state.intervs, i, nb_points)? {
                let mut j: i32 = 1;
                while j <= nb_points && is_line {
                    param = low + f64::from(j) * delta;
                    let middle_point: Point3 = curve.pnts_curve().lifted_d0(param)?;
                    let v2: Xyz = difference(middle_point, state.current_point);
                    let l2: f64 = v2.modulus();
                    if l2 > precision::confusion() {
                        is_line = v2.cross_magnitude(v1) / (l1 * l2) < a_tol;
                    }
                    j += 1;
                }
            }
            i += 1;
        }
        Ok((is_line, param))
    }

    // max(MinNbPnts, 3), raised to degree + 1 for a Bezier or B-spline curve.
    fn line_test_points<C: TCurveTypes + ?Sized>(&self, curve: &C) -> Result<i32, Failure> {
        let nb_points: i32 = if self.my_min_nb_pnts > 3 {
            self.my_min_nb_pnts
        } else {
            3
        };
        Ok(match curve.pnts_curve().get_type()? {
            CurveType::BSplineCurve => {
                (live(&curve.bspline_curve()?)?.pole_degree() + 1).max(nb_points)
            }
            CurveType::BezierCurve => {
                (live(&curve.bezier_curve()?)?.pole_degree() + 1).max(nb_points)
            }
            _ => nb_points,
        })
    }

    // Interval `i` of the line test: None outside [First, Last] ("Avoid usage intervals
    // out of [myFirstu, myLastU]"), else its lower bound and step after clamping its
    // bounds to the range ("Fix border points in applicable intervals").
    fn clamp_interval(
        &self,
        intervs: &mut Array1<'_, f64>,
        i: i32,
        nb_points: i32,
    ) -> Result<Option<(f64, f64)>, Failure> {
        let (first, last) = (self.my_firstu, self.my_last_u);
        if bound(intervs, i + 1)? < first || bound(intervs, i)? > last {
            return Ok(None);
        }
        if bound(intervs, i)? < first && bound(intervs, i + 1)? > first {
            intervs[i] = first;
        }
        if bound(intervs, i)? < last && bound(intervs, i + 1)? > last {
            intervs[i + 1] = last;
        }
        let (low, high) = (bound(intervs, i)?, bound(intervs, i + 1)?);
        Ok(Some((low, (high - low) / f64::from(nb_points))))
    }
}

// `ATol`: a hundredth of the angular deflection, bounded to [1e-7, 1e-2] with the
// native test order (`if (ATol > 1.e-2) ... else if (ATol < 1.e-7) ...`).
fn angular_tolerance(angular_deflection: f64) -> f64 {
    // f64::clamp returns a NaN input unchanged, as the two native tests do.
    (1.0e-2 * angular_deflection).clamp(1.0e-7, 1.0e-2)
}
