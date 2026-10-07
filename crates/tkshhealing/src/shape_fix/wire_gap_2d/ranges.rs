//! The range branch of ShapeFix_Wire_1.cxx::FixGap2d (`convert == false`), line by line:
//! the pcurves keep their geometry and only their ranges move to a common point that an
//! intersection, the extrema or the projections find; a point outside the surface bounds
//! is first clipped by the bounding lines.
use super::{Face, Failure, PcurveGap, ShapeFixWire, convert, range_clip, range_search, replace};
use crate::shape_extend::ShapeExtendStatus;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkmath::gp::Point2;
use std::f64::consts::PI;

/// The native locals of one range repair, curve 1 then curve 2 in each pair.
pub(super) struct RangeGap {
    /// `pc1`, `pc2`: the basic curves (trims removed, offsets merged).
    pub(super) curves: [Handle<dyn Geom2dCurve>; 2],
    /// `(first1, last1)`, `(first2, last2)`: the ranges in edge orientation.
    pub(super) bounds: [(f64, f64); 2],
    /// `reversed1`, `reversed2`.
    pub(super) reversed: [bool; 2],
    /// `cfirst1`, `clast2`: the parameters at the far ends of the junction.
    pub(super) far: [f64; 2],
    /// `clast1`, `cfirst2`: the parameters at the junction, also the initial `ipar`.
    pub(super) near: [f64; 2],
    /// `cpnt1`, `cpnt2`: the pcurve points at the junction.
    pub(super) points: [Point2; 2],
    /// `gap`: the distance between both junction points.
    pub(super) gap: f64,
    /// `(domfirst, domlast)` of each curve, where the search runs.
    pub(super) domains: [(f64, f64); 2],
    /// `ipar1`, `ipar2`: the new junction parameters.
    pub(super) ipar: [f64; 2],
    trimmed: [bool; 2],
}

/// Runs the range branch; `ranges::repair_range(tool, &face, &first, &second)?`.
pub(super) fn repair_range(
    tool: &ShapeFixWire,
    face: &Face,
    first: &PcurveGap,
    second: &PcurveGap,
) -> Result<bool, Failure> {
    let mut state = RangeGap::new(first, second)?;
    let done = if first.index == second.index {
        state.close_conic()
    } else {
        state.adjust_pair(tool, face)?
    };
    if !done.0 && !done.1 {
        return Ok(false);
    }
    let repair = |i: usize, done: bool| {
        done.then(|| replace::CurveRepair {
            curve: state.curves[i].clone(),
            first: state.bounds[i].0,
            last: state.bounds[i].1,
        })
    };
    let repairs = (repair(0, done.0), repair(1, done.1));
    replace::replace_pcurves_with_ranges(tool, face, (first, second), (&repairs.0, &repairs.1))?;
    tool.set_last_fix_status(ShapeExtendStatus::Done1);
    Ok(true)
}

impl RangeGap {
    fn new(first: &PcurveGap, second: &PcurveGap) -> Result<Self, Failure> {
        let entries = [first, second];
        let curves = [
            convert::basic_gap_curve(&first.curve)?,
            convert::basic_gap_curve(&second.curve)?,
        ];
        let near = [native_end(first), native_start(second)];
        let mut state = Self {
            domains: [(0., 0.); 2],
            bounds: entries.map(|e| (e.first, e.last)),
            reversed: entries.map(|e| e.reversed),
            far: [native_start(first), native_end(second)],
            near,
            points: [first.point, second.point],
            gap: first.point.distance(second.point),
            ipar: near,
            trimmed: entries.map(|e| e.trimmed),
            curves,
        };
        state.domains = [state.search_domain(0), state.search_domain(1)];
        Ok(state)
    }

    // n1 == n2: a closed circle or ellipse opens its range to a whole turn.
    fn close_conic(&mut self) -> (bool, bool) {
        if !is_circle_or_ellipse(&self.curves[0]) {
            return (false, false);
        }
        // `M_PI - std::abs(clast1 - cfirst2) * 0.5`, cfirst2 being cfirst1 here.
        let diff = PI - (self.near[0] - self.far[0]).abs() * 0.5;
        self.bounds[0].0 -= diff;
        self.bounds[0].1 += diff;
        (true, false)
    }

    fn adjust_pair(&mut self, tool: &ShapeFixWire, face: &Face) -> Result<(bool, bool), Failure> {
        let found = range_search::find(self)?;
        let mut done = (found, found);
        if found {
            range_clip::clip(self, tool, face)?;
        }
        // The native try block keeps every change made before a trim raises.
        let _ = self.apply(&mut done);
        Ok(done)
    }

    // The native search domain: the whole B-spline or Bezier, a whole turn of a conic,
    // a line or open conic extended ten times away from the junction.
    fn search_domain(&self, i: usize) -> (f64, f64) {
        let curve = &self.curves[i];
        let (mut first, mut last) = self.bounds[i];
        if is_bspline_or_bezier(curve) {
            return curve
                .get()
                .map_or((first, last), |c| (c.first_parameter(), c.last_parameter()));
        }
        if is_line_or_open_conic(curve) {
            let diff = last - first;
            if self.take_first(i) {
                first -= 10. * diff;
            } else {
                last += 10. * diff;
            }
            return (first, last);
        }
        if is_circle_or_ellipse(curve) {
            return (0., 2. * PI);
        }
        (first, last)
    }

    /// The `takefirst` flag of AdjustOnPeriodic2d: `reversed1` or `!reversed2`.
    pub(super) fn take_first(&self, i: usize) -> bool {
        if i == 0 {
            self.reversed[0]
        } else {
            !self.reversed[1]
        }
    }

    /// AdjustOnPeriodic2d of `parameter` on curve `i` over its range.
    pub(super) fn adjust(&self, i: usize, parameter: f64) -> Result<f64, Failure> {
        adjust_on_periodic(
            &self.curves[i],
            self.take_first(i),
            self.bounds[i],
            parameter,
        )
    }

    /// The native distance criterion that accepts a pair of new junction parameters.
    pub(super) fn accepts(&self, u: (f64, f64)) -> Result<bool, Failure> {
        let p1 = self.curves[0].value(u.0)?;
        let p2 = self.curves[1].value(u.1)?;
        let tolerance = rustycad_tkernel::precision::p_confusion();
        let (b1, b2) = (self.bounds[0], self.bounds[1]);
        Ok(p1.distance(p2) <= self.gap
            && (self.far[0] - u.0).abs() > tolerance
            && (self.far[1] - u.1).abs() > tolerance
            && ((u.0 > b1.0 && u.0 < b1.1)
                || (u.1 > b2.0 && u.1 < b2.1)
                || self.points[0].distance(p1) <= self.gap
                || self.points[1].distance(p2) <= self.gap))
    }

    // The native try block: a parameter equal to the old junction one cancels the
    // change, another one moves the bound and retrims a trimmed pcurve.
    fn apply(&mut self, done: &mut (bool, bool)) -> Result<(), Failure> {
        if done.0 {
            done.0 = self.move_bound(0)?;
        }
        if done.1 {
            done.1 = self.move_bound(1)?;
        }
        Ok(())
    }

    fn move_bound(&mut self, i: usize) -> Result<bool, Failure> {
        if self.ipar[i] == self.near[i] {
            return Ok(false);
        }
        // `reversed1 ? first1 : last1` and `reversed2 ? last2 : first2`.
        if self.take_first(i) {
            self.bounds[i].0 = self.ipar[i];
        } else {
            self.bounds[i].1 = self.ipar[i];
        }
        if self.trimmed[i] {
            let (first, last) = self.bounds[i];
            let trim = Geom2dTrimmedCurve::new(&self.curves[i], first, last, true, true)?;
            self.curves[i] = Handle::new(trim).cast();
        }
        Ok(true)
    }
}

/// AdjustOnPeriodic2d (OCC966): moves a parameter of a periodic curve by one period
/// toward the range; `adjust_on_periodic(&curve, true, (0., 1.), 7.)?`.
pub(super) fn adjust_on_periodic(
    curve: &Handle<dyn Geom2dCurve>,
    take_first: bool,
    (first, last): (f64, f64),
    parameter: f64,
) -> Result<f64, Failure> {
    if !crate::shape_analysis::ShapeAnalysisCurve::is_periodic_2d(curve)? {
        return Ok(parameter);
    }
    let period = curve.period()?;
    let mut shift = -(first / period).trunc() * period;
    if first < 0. {
        shift += period;
    }
    let (sfirst, slast) = (first + shift, last + shift);
    if take_first && parameter > slast && parameter > sfirst {
        return Ok(parameter - period - shift);
    }
    if !take_first && parameter < slast && parameter < sfirst {
        return Ok(parameter + period - shift);
    }
    Ok(parameter)
}

fn native_start(entry: &PcurveGap) -> f64 {
    if entry.reversed {
        entry.last
    } else {
        entry.first
    }
}

fn native_end(entry: &PcurveGap) -> f64 {
    if entry.reversed {
        entry.first
    } else {
        entry.last
    }
}

fn is_circle_or_ellipse(curve: &Handle<dyn Geom2dCurve>) -> bool {
    !curve.cast::<Geom2dCircle>().is_null() || !curve.cast::<Geom2dEllipse>().is_null()
}

fn is_bspline_or_bezier(curve: &Handle<dyn Geom2dCurve>) -> bool {
    !curve.cast::<Geom2dBSplineCurve>().is_null() || !curve.cast::<Geom2dBezierCurve>().is_null()
}

fn is_line_or_open_conic(curve: &Handle<dyn Geom2dCurve>) -> bool {
    !curve.cast::<Geom2dLine>().is_null()
        || !curve.cast::<Geom2dParabola>().is_null()
        || !curve.cast::<Geom2dHyperbola>().is_null()
}
