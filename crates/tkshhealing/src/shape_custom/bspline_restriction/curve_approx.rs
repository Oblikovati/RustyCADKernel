//! The approximation tails of `ConvertCurve` and `ConvertCurve2d`: decreasing
//! continuities, one retry with GMaxSeg (degree priority) or GMaxDegree, and the copy of
//! the input when the approximation is not good enough.
use super::ShapeCustomBSplineRestriction;
use super::curve3d::copied;
use super::predicates::{continuity_to_integer, integer_to_continuity};
use crate::shape_analysis::ShapeAnalysisCurve;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurve, Geom2dTrimmedCurve,
};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBezierCurve, GeomCurve, GeomTrimmedCurve};
use rustycad_tkgeombase::{geom_convert, geom2d_convert};
use rustycad_tkmath::geom_abs::Shape as Continuity;

/// What the native body does after one approximation of the inner `j` loop.
enum Verdict {
    /// `aC = aC1; continue` with a raised limit.
    Retry,
    /// The copy of the input (`IsConvert`).
    Copy,
    /// The approximation is kept.
    Accept,
}

/// One approximation result, as the native checks read it.
struct Measured {
    degree: i32,
    segments: i32,
    done: bool,
    error: f64,
}

/// The mutable limits of the loop (MaxSeg, MaxDeg) and the restart continuity aC1.
struct CurveLoop {
    max_seg: i32,
    max_deg: i32,
    start: i32,
}

impl ShapeCustomBSplineRestriction {
    fn verdict(
        &self,
        m: &Measured,
        l: &mut CurveLoop,
        tol_ref: f64,
        segments_exceed: bool,
        is_convert: bool,
    ) -> Result<Verdict, Failure> {
        let parameters = self.parameters()?;
        let bad = !m.done || m.error >= tol_ref;
        if self.read().degree_priority && (m.degree > l.max_deg || bad) {
            if l.max_seg < parameters.g_max_seg() {
                l.max_seg = parameters.g_max_seg();
                return Ok(Verdict::Retry);
            }
            if is_convert {
                return Ok(Verdict::Copy);
            }
        }
        if !self.read().degree_priority && (segments_exceed || bad) {
            if l.max_deg < parameters.g_max_degree() {
                l.max_deg = parameters.g_max_degree();
                return Ok(Verdict::Retry);
            }
            if is_convert {
                return Ok(Verdict::Copy);
            }
        }
        Ok(Verdict::Accept)
    }

    /// The approximation tail of `ConvertCurve` for B-splines, Beziers and kept offsets.
    pub(super) fn approximate_curve(
        &self,
        curve: &Handle<dyn GeomCurve>,
        result: &mut Handle<dyn GeomCurve>,
        is_convert: bool,
        range: (f64, f64),
        tol_cur: &mut f64,
    ) -> Result<bool, Failure> {
        if self.curve_within_limits(curve) {
            if is_convert {
                (*result, *tol_cur) = (copied(curve)?, precision::confusion());
            }
            return Ok(is_convert);
        }
        let target = trimmed_3d(curve, range)?;
        let state = self.read();
        let mut ac = start_continuity(state.continuity_3d, curve.continuity());
        let mut l = CurveLoop {
            max_seg: state.nb_max_seg,
            max_deg: state.max_degree,
            start: ac,
        };
        while ac >= 0 {
            // A raised exception (Err) is the native catch: on to the next continuity.
            if let Ok(Some(done)) =
                self.curve_passes(curve, &target, result, is_convert, tol_cur, &mut ac, &mut l)
            {
                return Ok(done);
            }
            ac -= 1;
        }
        Ok(false)
    }

    // The native `for (j = 1; j <= 2; j++)` body inside the `try`.
    #[allow(clippy::too_many_arguments)]
    fn curve_passes(
        &self,
        curve: &Handle<dyn GeomCurve>,
        target: &Handle<dyn GeomCurve>,
        result: &mut Handle<dyn GeomCurve>,
        is_convert: bool,
        tol_cur: &mut f64,
        ac: &mut i32,
        l: &mut CurveLoop,
    ) -> Result<Option<bool>, Failure> {
        let tol_3d = self.read().tol_3d;
        for _ in 1..=2 {
            let approx = geom_convert::ApproxCurve::from_curve(
                target,
                tol_3d,
                integer_to_continuity(*ac),
                l.max_seg,
                l.max_deg,
            )?;
            *result = approx.curve().cast();
            let spline = approx.curve();
            let spline = spline.get().ok_or_else(null_approximation)?;
            let m = Measured {
                degree: spline.degree(),
                segments: spline.nb_knots() - 1,
                done: approx.is_done(),
                error: approx.max_error(),
            };
            let exceed = m.segments > self.parameters()?.g_max_seg();
            match self.verdict(&m, l, tol_cur.max(tol_3d), exceed, is_convert)? {
                Verdict::Retry => {
                    *ac = l.start;
                    continue;
                }
                Verdict::Copy => {
                    (*result, *tol_cur) = (copied(curve)?, precision::confusion());
                    return Ok(Some(true));
                }
                Verdict::Accept => {}
            }
            *tol_cur = m.error;
            let mut state = self.state.write();
            state.converted = true;
            state.curve_3d_error = state.curve_3d_error.max(m.error);
            return Ok(Some(true));
        }
        Ok(None)
    }

    fn curve_within_limits(&self, curve: &Handle<dyn GeomCurve>) -> bool {
        let state = self.read();
        if let Some(s) = curve.cast::<GeomBSplineCurve>().get() {
            let rational = state.rational && s.is_rational();
            return !rational
                && s.degree() <= state.max_degree
                && s.nb_knots() - 1 <= state.nb_max_seg;
        }
        if let Some(s) = curve.cast::<GeomBezierCurve>().get() {
            return !(state.rational && s.is_rational()) && s.degree() <= state.max_degree;
        }
        false
    }

    /// The approximation tail of `ConvertCurve2d` for B-splines, Beziers and kept offsets.
    pub(super) fn approximate_curve_2d(
        &self,
        curve: &Handle<dyn Geom2dCurve>,
        result: &mut Handle<dyn Geom2dCurve>,
        is_convert: bool,
        range: (f64, f64),
        tol_cur: &mut f64,
    ) -> Result<bool, Failure> {
        if self.curve_2d_within_limits(curve) {
            if is_convert {
                (*result, *tol_cur) = (curve.copy()?.cast(), precision::p_confusion());
            }
            return Ok(is_convert);
        }
        let target = trimmed_2d(curve, range)?;
        let state = self.read();
        let mut ac = start_continuity(state.continuity_2d, curve.continuity());
        let mut l = CurveLoop {
            max_seg: state.nb_max_seg,
            max_deg: state.max_degree,
            start: ac,
        };
        while ac >= 0 {
            if let Ok(Some(done)) =
                self.curve_2d_passes(curve, &target, result, is_convert, tol_cur, &mut ac, &mut l)
            {
                return Ok(done);
            }
            ac -= 1;
        }
        Ok(false)
    }

    // The 2D `j` loop: the continuity is fixed before it (a retry keeps the old one) and
    // the segment test compares with the current MaxSeg, both native differences kept.
    #[allow(clippy::too_many_arguments)]
    fn curve_2d_passes(
        &self,
        curve: &Handle<dyn Geom2dCurve>,
        target: &Handle<dyn Geom2dCurve>,
        result: &mut Handle<dyn Geom2dCurve>,
        is_convert: bool,
        tol_cur: &mut f64,
        ac: &mut i32,
        l: &mut CurveLoop,
    ) -> Result<Option<bool>, Failure> {
        let tol_2d = self.read().tol_2d;
        let continuity = integer_to_continuity(*ac);
        for _ in 1..=2 {
            let approx = geom2d_convert::ApproxCurve::from_curve(
                target, tol_2d, continuity, l.max_seg, l.max_deg,
            )?;
            *result = approx.curve().cast();
            let spline = approx.curve();
            let spline = spline.get().ok_or_else(null_approximation)?;
            let m = Measured {
                degree: spline.degree(),
                segments: spline.nb_knots() - 1,
                done: approx.is_done(),
                error: approx.max_error(),
            };
            let exceed = m.segments >= l.max_seg;
            match self.verdict(&m, l, tol_2d.max(*tol_cur), exceed, is_convert)? {
                Verdict::Retry => {
                    *ac = l.start;
                    continue;
                }
                Verdict::Copy => {
                    (*result, *tol_cur) = (curve.copy()?.cast(), precision::p_confusion());
                    return Ok(Some(true));
                }
                Verdict::Accept => {}
            }
            *tol_cur = m.error;
            let mut state = self.state.write();
            state.converted = true;
            state.curve_2d_error = state.curve_2d_error.max(m.error);
            return Ok(Some(true));
        }
        Ok(None)
    }

    fn curve_2d_within_limits(&self, curve: &Handle<dyn Geom2dCurve>) -> bool {
        let state = self.read();
        if let Some(s) = curve.cast::<Geom2dBSplineCurve>().get() {
            let rational = state.rational && s.is_rational();
            return !rational
                && s.degree() <= state.max_degree
                && s.nb_knots() - 1 <= state.nb_max_seg;
        }
        if let Some(s) = curve.cast::<Geom2dBezierCurve>().get() {
            return !(state.rational && s.is_rational()) && s.degree() <= state.max_degree;
        }
        false
    }
}

/// `aC = min(ContToInteger(myContinuity), ContToInteger(curve continuity))`, the required
/// continuity when that is 0.
fn start_continuity(required: Continuity, actual: Continuity) -> i32 {
    let start = continuity_to_integer(required).min(continuity_to_integer(actual));
    if start == 0 {
        continuity_to_integer(required)
    } else {
        start
    }
}

/// Natively a null approximation is dereferenced inside the `try`; with OCC signal
/// handling that is a caught exception, which Rust models as this failure.
fn null_approximation() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeCustom_BSplineRestriction: GeomConvert_ApproxCurve has no curve",
        "",
    )
}

/// `aCurve1`: a periodic curve trimmed to the range, a wider curve trimmed to the
/// overlap (when not empty), the curve itself otherwise.
fn trimmed_3d(
    curve: &Handle<dyn GeomCurve>,
    (first, last): (f64, f64),
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let (pf, pl) = (curve.first_parameter(), curve.last_parameter());
    if ShapeAnalysisCurve::is_periodic(curve)? && first != last {
        return Ok(Handle::new(GeomTrimmedCurve::new(curve, first, last, true, true)?).cast());
    }
    if pf < first - precision::p_confusion() || pl > last + precision::p_confusion() {
        let (f, l) = (first.max(pf), last.min(pl));
        if f != l {
            return Ok(Handle::new(GeomTrimmedCurve::new(curve, f, l, true, true)?).cast());
        }
    }
    Ok(curve.clone())
}

fn trimmed_2d(
    curve: &Handle<dyn Geom2dCurve>,
    (first, last): (f64, f64),
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let (pf, pl) = (curve.first_parameter(), curve.last_parameter());
    if ShapeAnalysisCurve::is_periodic_2d(curve)? && first != last {
        return Ok(Handle::new(Geom2dTrimmedCurve::new(curve, first, last, true, true)?).cast());
    }
    if pf < first - precision::p_confusion() || pl > last + precision::p_confusion() {
        let (f, l) = (first.max(pf), last.min(pl));
        if f != l {
            return Ok(Handle::new(Geom2dTrimmedCurve::new(curve, f, l, true, true)?).cast());
        }
    }
    Ok(curve.clone())
}
