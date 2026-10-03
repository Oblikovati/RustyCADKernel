//! Native projection/extrema range adjustment, separate from spline conversion.
mod candidates;
mod extrema;
use super::{
    Failure, GapCurve, GapPair, GeomCurve, Handle, ShapeExtendStatus, ShapeFixWire, convert,
    replace,
};
use replace::CurveRepair;
use rustycad_tkg3d::geom::{GeomCircle, GeomEllipse, GeomTrimmedCurve};
pub(super) struct RangeCurve<'a> {
    entry: &'a GapCurve,
    repair: CurveRepair,
    trimmed: bool,
    first_end: bool,
}
pub(super) fn repair(tool: &ShapeFixWire, pair: &GapPair) -> Result<bool, Failure> {
    let mut first = RangeCurve::new(&pair.first, true)?;
    let mut second = RangeCurve::new(&pair.second, false)?;
    let flags = if pair.first.index == pair.second.index {
        close_conic(&mut first)
    } else {
        adjust_pair(pair, &mut first, &mut second)?
    };
    if !flags.0 && !flags.1 {
        return Ok(false);
    }
    replace::replace_gap_curves(
        tool,
        pair,
        flags.0.then_some(&first.repair),
        flags.1.then_some(&second.repair),
    )?;
    tool.set_last_fix_status(ShapeExtendStatus::Done1);
    Ok(true)
}
impl<'a> RangeCurve<'a> {
    fn new(entry: &'a GapCurve, first_edge: bool) -> Result<Self, Failure> {
        let mut outer = entry.curve.clone();
        let mut trimmed = false;
        loop {
            let trim = outer.cast::<GeomTrimmedCurve>();
            if let Some(trim) = trim.get() {
                trimmed = true;
                outer = trim.basis_curve();
                continue;
            }
            let offset = outer.cast::<rustycad_tkg3d::geom::GeomOffsetCurve>();
            let Some(offset) = offset.get() else {
                break;
            };
            outer = offset.basis_curve();
        }
        let curve = convert::basic_gap_curve(&entry.curve)?;
        Ok(Self {
            entry,
            repair: CurveRepair {
                curve,
                first: entry.first,
                last: entry.last,
            },
            trimmed,
            first_end: if first_edge {
                entry.reversed
            } else {
                !entry.reversed
            },
        })
    }
    fn endpoint(&self) -> f64 {
        if self.first_end {
            self.entry.first
        } else {
            self.entry.last
        }
    }
    fn opposite_endpoint(&self) -> f64 {
        if self.first_end {
            self.entry.last
        } else {
            self.entry.first
        }
    }
    fn apply_parameter(&mut self, parameter: f64, done: &mut bool) -> Result<(), Failure> {
        if parameter == self.endpoint() {
            *done = false;
            return Ok(());
        }
        if self.first_end {
            self.repair.first = parameter;
        } else {
            self.repair.last = parameter;
        }
        if self.trimmed {
            self.repair.curve = Handle::new(GeomTrimmedCurve::new(
                &self.repair.curve,
                self.repair.first,
                self.repair.last,
                true,
                true,
            )?)
            .cast();
        }
        Ok(())
    }
}
fn close_conic(first: &mut RangeCurve<'_>) -> (bool, bool) {
    if !is_ellipse_or_circle(&first.repair.curve) {
        return (false, false);
    }
    let delta = std::f64::consts::PI - (first.entry.last - first.entry.first).abs() * 0.5;
    first.repair.first -= delta;
    first.repair.last += delta;
    (true, false)
}
fn is_ellipse_or_circle(curve: &Handle<dyn GeomCurve>) -> bool {
    !curve.cast::<GeomCircle>().is_null() || !curve.cast::<GeomEllipse>().is_null()
}
fn adjust_pair(
    pair: &GapPair,
    first: &mut RangeCurve<'_>,
    second: &mut RangeCurve<'_>,
) -> Result<(bool, bool), Failure> {
    let projected = (
        candidates::project(first, pair.target)?,
        candidates::project(second, pair.target)?,
    );
    let parameters = if candidates::acceptable(pair, first, second, projected)? {
        Some(projected)
    } else {
        extrema::closest(pair, first, second)?
    };
    let Some((u, v)) = parameters else {
        return Ok((false, false));
    };
    let mut done = (true, true);
    // OCCT catches trimming failures after mutating the corresponding range and flags.
    if first.apply_parameter(u, &mut done.0).is_ok() {
        let _ = second.apply_parameter(v, &mut done.1);
    }
    Ok(done)
}
