use super::closed_mode::PeriodicAxis;
use super::helpers::null_pcurve;
use crate::shape_analysis::ShapeAnalysisCurve;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::{
    bnd::BoundingBox2,
    gp::{Point2, Vector2},
};

pub(super) struct EdgePcurveScan {
    pub(super) curve: Handle<dyn Geom2dCurve>,
    pub(super) first: f64,
    pub(super) last: f64,
    pub(super) start: Point2,
    pub(super) end: Point2,
    pub(super) next_shift: Vector2,
    pub(super) iterations: usize,
    owns_curve: bool,
}

impl EdgePcurveScan {
    pub(super) fn new(
        curve: &Handle<dyn Geom2dCurve>,
        first: f64,
        last: f64,
        axis: Option<PeriodicAxis>,
    ) -> Result<Self, Failure> {
        let (shift, next_shift, iterations) = initial_period_shifts(curve, first, last, axis)?;
        let (curve, owns_curve) = translated_curve(curve, shift)?;
        let first_point = curve.get().ok_or_else(null_pcurve)?.value(first)?;
        let last_point = curve.get().ok_or_else(null_pcurve)?.value(last)?;
        let (start, end) = adjust_endpoints(first_point, last_point, axis);
        Ok(Self {
            curve,
            first,
            last,
            start,
            end,
            next_shift,
            iterations,
            owns_curve,
        })
    }

    pub(super) fn advance_period(&mut self) -> Result<(), Failure> {
        if self.next_shift.x() == 0.0 && self.next_shift.y() == 0.0 {
            return Ok(());
        }
        if !self.owns_curve {
            self.curve = copy_curve(&self.curve)?;
            self.owns_curve = true;
        }
        self.curve
            .get()
            .ok_or_else(null_pcurve)?
            .translate(self.next_shift)
    }
}

fn initial_period_shifts(
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
    axis: Option<PeriodicAxis>,
) -> Result<(Vector2, Vector2, usize), Failure> {
    let Some(axis) = axis else {
        return Ok((Vector2::default(), Vector2::default(), 1));
    };
    let bounds = curve_bounds(curve, first, last)?;
    match axis {
        PeriodicAxis::U {
            line_coordinate,
            period,
        } => u_shifts(bounds, line_coordinate, period),
        PeriodicAxis::V {
            line_coordinate,
            period,
        } => v_shifts(bounds, line_coordinate, period),
    }
}

fn curve_bounds(
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<[f64; 4], Failure> {
    let mut bounds = BoundingBox2::default();
    ShapeAnalysisCurve.fill_bnd_box(curve, first, last, 41, true, &mut bounds)?;
    bounds.get()
}

fn u_shifts(
    bounds: [f64; 4],
    line_coordinate: f64,
    period: f64,
) -> Result<(Vector2, Vector2, usize), Failure> {
    let shift = crate::shape_analysis::adjust_to_period(
        bounds[0],
        line_coordinate - period,
        line_coordinate,
    );
    let maximum = bounds[2] + shift - line_coordinate;
    let next = if maximum > 0.0 { -period } else { period };
    let iterations = (1.0 + maximum.abs() / period) as usize;
    Ok((
        Vector2::new(shift, 0.0),
        Vector2::new(next, 0.0),
        iterations.max(1),
    ))
}

fn v_shifts(
    bounds: [f64; 4],
    line_coordinate: f64,
    period: f64,
) -> Result<(Vector2, Vector2, usize), Failure> {
    let shift = crate::shape_analysis::adjust_to_period(
        bounds[1],
        line_coordinate - period,
        line_coordinate,
    );
    let maximum = bounds[3] + shift - line_coordinate;
    let next = if maximum > 0.0 { -period } else { period };
    let iterations = (1.0 + maximum.abs() / period) as usize;
    Ok((
        Vector2::new(0.0, shift),
        Vector2::new(0.0, next),
        iterations.max(1),
    ))
}

fn translated_curve(
    curve: &Handle<dyn Geom2dCurve>,
    shift: Vector2,
) -> Result<(Handle<dyn Geom2dCurve>, bool), Failure> {
    if shift.x() == 0.0 && shift.y() == 0.0 {
        return Ok((curve.clone(), false));
    }
    let translated = copy_curve(curve)?;
    translated.get().ok_or_else(null_pcurve)?.translate(shift)?;
    Ok((translated, true))
}

fn copy_curve(curve: &Handle<dyn Geom2dCurve>) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let copy = curve.get().ok_or_else(null_pcurve)?.copy()?;
    let translated = copy.cast::<dyn Geom2dCurve>();
    if translated.is_null() {
        return Err(null_pcurve());
    }
    Ok(translated)
}

fn adjust_endpoints(first: Point2, last: Point2, axis: Option<PeriodicAxis>) -> (Point2, Point2) {
    let Some(axis) = axis else {
        return (first, last);
    };
    match axis {
        PeriodicAxis::U {
            line_coordinate,
            period,
        } => (
            Point2::new(
                crate::shape_analysis::adjust_by_period(first.x(), line_coordinate, period)
                    + first.x(),
                first.y(),
            ),
            Point2::new(
                crate::shape_analysis::adjust_by_period(last.x(), line_coordinate, period)
                    + last.x(),
                last.y(),
            ),
        ),
        PeriodicAxis::V {
            line_coordinate,
            period,
        } => (
            Point2::new(
                first.x(),
                crate::shape_analysis::adjust_by_period(first.y(), line_coordinate, period)
                    + first.y(),
            ),
            Point2::new(
                last.x(),
                crate::shape_analysis::adjust_by_period(last.y(), line_coordinate, period)
                    + last.y(),
            ),
        ),
    }
}
