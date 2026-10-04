use super::fallback::{adjust_pair, native_end_parameter, native_start_parameter};
use super::{Failure, PcurveGap};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkgeomalgo::geom2d_api::ProjectPointOnCurve;
use rustycad_tkmath::gp::Point2;

pub(super) fn project_and_select_parameters(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    first_domain: (f64, f64),
    second_domain: (f64, f64),
) -> Result<(f64, f64), Failure> {
    let mut pair = project_endpoints(
        first,
        second,
        first_curve,
        second_curve,
        first_domain,
        second_domain,
    )?;
    adjust_initial_pair(first, second, first_curve, second_curve, &mut pair)?;
    let selected = select_projection_pair(
        first,
        second,
        first_curve,
        second_curve,
        pair,
        (first_domain, second_domain),
    )?;
    Ok((selected.first, selected.second))
}

fn select_projection_pair(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    mut pair: ProjectionPair,
    domains: ((f64, f64), (f64, f64)),
) -> Result<ProjectionPair, Failure> {
    if projections_cross_original_ends(first, second, pair.first, pair.second) {
        return project_midpoint(first, second, first_curve, second_curve, domains);
    }
    retain_nearer_endpoint(first, second, &mut pair);
    Ok(pair)
}

#[derive(Clone, Copy)]
struct ProjectionPair {
    first_point: Point2,
    second_point: Point2,
    first: f64,
    second: f64,
}

fn project_endpoints(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    first_domain: (f64, f64),
    second_domain: (f64, f64),
) -> Result<ProjectionPair, Failure> {
    project_pair(
        first.point,
        second.point,
        first,
        second,
        first_curve,
        second_curve,
        (first_domain, second_domain),
    )
}

fn project_pair(
    first_point: Point2,
    second_point: Point2,
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    domains: ((f64, f64), (f64, f64)),
) -> Result<ProjectionPair, Failure> {
    let mut pair = initial_projection_pair(first_point, second_point, first, second);
    project_endpoints_into_pair(
        &mut pair,
        (first, second),
        (first_curve, second_curve),
        domains,
    )?;
    Ok(pair)
}

fn initial_projection_pair(
    first_point: Point2,
    second_point: Point2,
    first: &PcurveGap,
    second: &PcurveGap,
) -> ProjectionPair {
    ProjectionPair {
        first_point,
        second_point,
        first: native_end_parameter(first),
        second: native_start_parameter(second),
    }
}

fn project_endpoints_into_pair(
    pair: &mut ProjectionPair,
    entries: (&PcurveGap, &PcurveGap),
    curves: (&Handle<dyn Geom2dCurve>, &Handle<dyn Geom2dCurve>),
    domains: ((f64, f64), (f64, f64)),
) -> Result<(), Failure> {
    project_point(
        entries.1.point,
        curves.0,
        domains.0,
        &mut pair.first_point,
        &mut pair.first,
    )?;
    project_point(
        entries.0.point,
        curves.1,
        domains.1,
        &mut pair.second_point,
        &mut pair.second,
    )
}

fn project_point(
    point: Point2,
    curve: &Handle<dyn Geom2dCurve>,
    domain: (f64, f64),
    projected: &mut Point2,
    parameter: &mut f64,
) -> Result<(), Failure> {
    let projection = ProjectPointOnCurve::with_range(point, curve, domain.0, domain.1)?;
    if projection.nb_points() == 0 {
        return Ok(());
    }
    *projected = projection.nearest_point()?;
    *parameter = projection.lower_distance_parameter()?;
    Ok(())
}

fn adjust_initial_pair(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    pair: &mut ProjectionPair,
) -> Result<(), Failure> {
    (pair.first, pair.second) = adjust_pair(
        first,
        second,
        first_curve,
        second_curve,
        (pair.first, pair.second),
    )?;
    Ok(())
}

fn projections_cross_original_ends(
    first: &PcurveGap,
    second: &PcurveGap,
    first_parameter: f64,
    second_parameter: f64,
) -> bool {
    let first_inside = on_gap_side(first, first_parameter, native_end_parameter(first), true);
    let second_inside = on_gap_side(
        second,
        second_parameter,
        native_start_parameter(second),
        false,
    );
    let first_outside = off_gap_side(first, first_parameter, native_end_parameter(first), true);
    let second_outside = off_gap_side(
        second,
        second_parameter,
        native_start_parameter(second),
        false,
    );
    first_inside && second_inside || first_outside && second_outside
}

fn on_gap_side(entry: &PcurveGap, parameter: f64, endpoint: f64, first_edge: bool) -> bool {
    if entry.reversed == first_edge {
        return parameter > endpoint;
    }
    parameter < endpoint
}

fn off_gap_side(entry: &PcurveGap, parameter: f64, endpoint: f64, first_edge: bool) -> bool {
    parameter != endpoint && !on_gap_side(entry, parameter, endpoint, first_edge)
}

fn project_midpoint(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    domains: ((f64, f64), (f64, f64)),
) -> Result<ProjectionPair, Failure> {
    let midpoint = midpoint(first.point, second.point);
    let mut pair = initial_projection_pair(first.point, second.point, first, second);
    project_point(
        midpoint,
        first_curve,
        domains.0,
        &mut pair.first_point,
        &mut pair.first,
    )?;
    project_point(
        midpoint,
        second_curve,
        domains.1,
        &mut pair.second_point,
        &mut pair.second,
    )?;
    Ok(pair)
}

fn midpoint(first: Point2, second: Point2) -> Point2 {
    Point2::new(
        (first.x() + second.x()) * 0.5,
        (first.y() + second.y()) * 0.5,
    )
}

fn retain_nearer_endpoint(first: &PcurveGap, second: &PcurveGap, pair: &mut ProjectionPair) {
    if first.point.distance(pair.second_point) < second.point.distance(pair.first_point) {
        pair.first = native_end_parameter(first);
    } else {
        pair.second = native_start_parameter(second);
    }
}
