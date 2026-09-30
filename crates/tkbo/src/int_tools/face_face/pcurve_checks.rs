// IsCurveValid and CheckPCurve from IntTools_FaceFace.cxx at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Self-intersection checks and the native periodically shifted sampling domain.
use super::Context;
use rustycad_tkbrep::{brep::BRepTool, topods::Face};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::{
    adaptor2d::Adaptor2dCurve2d,
    geom2d::{Geom2dCurve, arrays},
    geom2d_adaptor::Geom2dAdaptorCurve,
};
use rustycad_tkg3d::geom::GeomRectangularTrimmedSurface;
use rustycad_tkgeomalgo::{geom2d_int::GInter, int_res2d::Domain};
use rustycad_tkmath::{geom_abs::Shape, gp::Point2};
pub(super) fn curve_valid(curve: &Handle<dyn Geom2dCurve>) -> Result<bool, Failure> {
    if curve.is_null() {
        return Ok(false);
    }
    if curve.is_closed()? || curve.is_periodic() {
        return Ok(true);
    }
    let (adaptor, domain) = intersection_domain(curve)?;
    let mut intersection = GInter::new();
    intersection.perform_self_on(&adaptor, &domain, 1e-10, 1e-10)?;
    Ok(!intersection.is_done() || intersection.nb_points()? <= 0)
}
pub(super) fn check_pcurve(
    curve: &Handle<dyn Geom2dCurve>,
    face: &Face,
    context: &Context,
) -> Result<bool, Failure> {
    let (u1, u2, v1, v2) = context.uv_bounds(face)?;
    let mut bounds = [u1, u2, v1, v2];
    let tolerance = [
        native_max((u2 - u1) * 0.01, precision::confusion()),
        native_max((v2 - v1) * 0.01, precision::confusion()),
    ];
    shift_domain(curve, face, &mut bounds)?;
    sample_bounds(curve, bounds, tolerance)
}
fn native_max(first: f64, second: f64) -> f64 {
    if first < second { second } else { first }
}
fn shift_domain(
    curve: &Handle<dyn Geom2dCurve>,
    face: &Face,
    bounds: &mut [f64; 4],
) -> Result<(), Failure> {
    let (mut surface, _) = BRepTool::surface_with_location(face)?;
    let trimmed = surface.cast::<GeomRectangularTrimmedSurface>();
    if !trimmed.is_null() {
        surface = trimmed.basis_surface();
    }
    let midpoint = curve.eval_d0((curve.first_parameter() + curve.last_parameter()) / 2.)?;
    if surface.is_u_periodic() {
        shift_axis(bounds, 0, midpoint.x(), surface.u_period()?);
    }
    if surface.is_v_periodic() {
        shift_axis(bounds, 2, midpoint.y(), surface.v_period()?);
    }
    Ok(())
}
fn shift_axis(bounds: &mut [f64; 4], axis: usize, coordinate: f64, period: f64) {
    let mut shift = ((coordinate - bounds[axis]) / period) as i32;
    if coordinate < bounds[axis] + period * f64::from(shift) {
        shift = shift.wrapping_sub(1);
    }
    bounds[axis] += period * f64::from(shift);
    bounds[axis + 1] += period * f64::from(shift);
}
fn sample_bounds(
    curve: &Handle<dyn Geom2dCurve>,
    bounds: [f64; 4],
    tolerance: [f64; 2],
) -> Result<bool, Failure> {
    let adaptor = Geom2dAdaptorCurve::with_curve(curve)?;
    let count = adaptor.nb_intervals(Shape::CN)?;
    let mut intervals = arrays::one_based(count + 1)?;
    adaptor.intervals(&mut intervals, Shape::CN)?;
    let mut parameter = adaptor.first_parameter()?;
    for index in 1..=count {
        let step = (arrays::at(&intervals, index + 1)? - arrays::at(&intervals, index)?) / 23.;
        if !sample_interval(&adaptor, &mut parameter, step, bounds, tolerance)? {
            return Ok(false);
        }
    }
    Ok(true)
}
fn sample_interval(
    curve: &Geom2dAdaptorCurve,
    parameter: &mut f64,
    step: f64,
    bounds: [f64; 4],
    tolerance: [f64; 2],
) -> Result<bool, Failure> {
    // Native carries the 22nd sample into the next CN interval instead of resetting it.
    for _ in 1..23 {
        *parameter += step;
        if outside(curve.eval_d0(*parameter)?, bounds, tolerance) {
            return Ok(false);
        }
    }
    Ok(true)
}
fn outside(point: Point2, bounds: [f64; 4], tolerance: [f64; 2]) -> bool {
    bounds[0] - point.x() > tolerance[0]
        || point.x() - bounds[1] > tolerance[0]
        || bounds[2] - point.y() > tolerance[1]
        || point.y() - bounds[3] > tolerance[1]
}

fn intersection_domain(
    curve: &Handle<dyn Geom2dCurve>,
) -> Result<(Geom2dAdaptorCurve, Domain), Failure> {
    let mut first = curve.first_parameter();
    let mut last = curve.last_parameter();
    let endpoints = [curve.eval_d0(first)?, curve.eval_d0(last)?];
    let adaptor = Geom2dAdaptorCurve::with_curve(curve)?;
    adjust_interval(&adaptor, &mut first, &mut last)?;
    let domain = Domain::bounded(endpoints[0], first, 1e-10, endpoints[1], last, 1e-10);
    Ok((adaptor, domain))
}
fn adjust_interval(
    adaptor: &Geom2dAdaptorCurve,
    first: &mut f64,
    last: &mut f64,
) -> Result<(), Failure> {
    if !adaptor.is_periodic()? {
        if adaptor.first_parameter()? > *first {
            *first = adaptor.first_parameter()?;
        }
        if adaptor.last_parameter()? < *last {
            *last = adaptor.last_parameter()?;
        }
    }
    Ok(())
}
