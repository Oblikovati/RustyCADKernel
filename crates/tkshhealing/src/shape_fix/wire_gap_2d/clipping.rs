use super::PcurveGap;
use rustycad_tkbrep::{brep::BRepTool, brep_tools::uv_bounds, topods::Face};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::{
    geom2d::{Geom2dCurve, Geom2dLine},
    geom2d_adaptor::Geom2dAdaptorCurve,
};
use rustycad_tkgeomalgo::{geom2d_int::GInter, int_res2d::Domain};
use rustycad_tkmath::gp::{Direction2, Point2};

pub(super) struct ClippedPair {
    pub(super) first_parameter: f64,
    pub(super) second_parameter: f64,
    pub(super) first_point: Point2,
    pub(super) second_point: Point2,
}

pub(super) fn apply(
    face: &Face,
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    parameters: (f64, f64),
) -> Result<ClippedPair, Failure> {
    let bounds = surface_bounds(face)?;
    let mut result = pair_at(first_curve, second_curve, parameters)?;
    clip_if_outside(first, first_curve, bounds, true, &mut result)?;
    clip_if_outside(second, second_curve, bounds, false, &mut result)?;
    Ok(result)
}

fn surface_bounds(face: &Face) -> Result<(f64, f64, f64, f64), Failure> {
    let (surface, _) = BRepTool::surface_with_location(face)?;
    let surface = surface.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "FixGap2d range clipping expected a face surface",
        )
    })?;
    let (mut umin, mut umax, mut vmin, mut vmax) = surface.bounds();
    fill_infinite_bounds(face, &mut umin, &mut umax, &mut vmin, &mut vmax)?;
    Ok((umin, umax, vmin, vmax))
}

fn fill_infinite_bounds(
    face: &Face,
    umin: &mut f64,
    umax: &mut f64,
    vmin: &mut f64,
    vmax: &mut f64,
) -> Result<(), Failure> {
    if !precision::is_infinite(*umin)
        && !precision::is_infinite(*umax)
        && !precision::is_infinite(*vmin)
        && !precision::is_infinite(*vmax)
    {
        return Ok(());
    }
    let face_bounds = uv_bounds(face)?;
    fill_axis_bounds((umin, umax), (face_bounds.0, face_bounds.1))?;
    fill_axis_bounds((vmin, vmax), (face_bounds.2, face_bounds.3))?;
    Ok(())
}

fn fill_axis_bounds(bounds: (&mut f64, &mut f64), face_bounds: (f64, f64)) -> Result<(), Failure> {
    replace_infinite(bounds.0, face_bounds.0 - precision::p_confusion());
    replace_infinite(bounds.1, face_bounds.1 + precision::p_confusion());
    Ok(())
}

fn replace_infinite(bound: &mut f64, replacement: f64) {
    if precision::is_infinite(*bound) {
        *bound = replacement;
    }
}

fn clip_if_outside(
    entry: &PcurveGap,
    curve: &Handle<dyn Geom2dCurve>,
    bounds: (f64, f64, f64, f64),
    first_edge: bool,
    pair: &mut ClippedPair,
) -> Result<(), Failure> {
    let parameter = edge_parameter(pair, first_edge);
    if !outside_old_range(entry, parameter) {
        return Ok(());
    }
    clip_axis(entry, curve, bounds, first_edge, true, pair)?;
    clip_axis(entry, curve, bounds, first_edge, false, pair)?;
    clamp_edge(entry, first_edge, pair);
    Ok(())
}

fn clip_axis(
    entry: &PcurveGap,
    curve: &Handle<dyn Geom2dCurve>,
    bounds: (f64, f64, f64, f64),
    first_edge: bool,
    u_axis: bool,
    pair: &mut ClippedPair,
) -> Result<(), Failure> {
    let point = edge_point(pair, first_edge);
    let Some(boundary) = outside_boundary(point, bounds, u_axis) else {
        return Ok(());
    };
    let Some((parameter, point)) = intersect_boundary(entry, curve, boundary, first_edge, pair)?
    else {
        return Ok(());
    };
    set_edge_point(pair, first_edge, point);
    set_edge_parameter(pair, first_edge, parameter);
    Ok(())
}

fn outside_boundary(
    point: Point2,
    bounds: (f64, f64, f64, f64),
    u_axis: bool,
) -> Option<(Point2, Point2)> {
    let (umin, umax, vmin, vmax) = bounds;
    if u_axis && point.x() < umin {
        return Some((Point2::new(umin, vmin), Point2::new(umin, vmax)));
    }
    if u_axis && point.x() > umax {
        return Some((Point2::new(umax, vmin), Point2::new(umax, vmax)));
    }
    if !u_axis && point.y() < vmin {
        return Some((Point2::new(umin, vmin), Point2::new(umax, vmin)));
    }
    if !u_axis && point.y() > vmax {
        return Some((Point2::new(umin, vmax), Point2::new(umax, vmax)));
    }
    None
}

fn intersect_boundary(
    entry: &PcurveGap,
    curve: &Handle<dyn Geom2dCurve>,
    boundary: (Point2, Point2),
    first_edge: bool,
    pair: &ClippedPair,
) -> Result<Option<(f64, Point2)>, Failure> {
    let line = boundary_line(boundary)?;
    let first = entry_parameter(entry, first_edge);
    let last = edge_parameter(pair, first_edge);
    let intersections = solve_boundary(curve, &line, boundary, (first, last))?;
    closest_boundary_hit(entry, &intersections, first_edge, first, curve)
}

fn solve_boundary(
    curve: &Handle<dyn Geom2dCurve>,
    line: &Handle<dyn Geom2dCurve>,
    boundary: (Point2, Point2),
    limits: (f64, f64),
) -> Result<GInter, Failure> {
    let domains = boundary_domains(curve, line, boundary, limits)?;
    let adaptors = (
        Geom2dAdaptorCurve::with_curve(line)?,
        Geom2dAdaptorCurve::with_curve(curve)?,
    );
    GInter::with_domains(
        &adaptors.0,
        &domains.0,
        &adaptors.1,
        &domains.1,
        precision::p_confusion(),
        precision::p_confusion(),
    )
}

fn boundary_domains(
    curve: &Handle<dyn Geom2dCurve>,
    line: &Handle<dyn Geom2dCurve>,
    boundary: (Point2, Point2),
    limits: (f64, f64),
) -> Result<(Domain, Domain), Failure> {
    let curve_domain = make_domain(curve, limits.0.min(limits.1), limits.0.max(limits.1))?;
    let line_domain = make_domain(line, 0.0, boundary.0.distance(boundary.1))?;
    Ok((line_domain, curve_domain))
}

fn boundary_line(boundary: (Point2, Point2)) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let direction = Direction2::new(
        boundary.1.x() - boundary.0.x(),
        boundary.1.y() - boundary.0.y(),
    )?;
    Ok(Handle::new(Geom2dLine::from_point(boundary.0, direction)).cast())
}

fn make_domain(curve: &Handle<dyn Geom2dCurve>, first: f64, last: f64) -> Result<Domain, Failure> {
    Ok(Domain::bounded(
        curve.value(first)?,
        first,
        precision::p_confusion(),
        curve.value(last)?,
        last,
        precision::p_confusion(),
    ))
}

fn closest_boundary_hit(
    entry: &PcurveGap,
    intersections: &GInter,
    first_edge: bool,
    parameter: f64,
    curve: &Handle<dyn Geom2dCurve>,
) -> Result<Option<(f64, Point2)>, Failure> {
    let points = closest_intersection_points(intersections, parameter)?;
    let best = closest_intersection_segments(intersections, parameter, points)?;
    adjusted_hit(entry, first_edge, best, curve)
}

fn closest_intersection_points(
    intersections: &GInter,
    parameter: f64,
) -> Result<Option<(f64, Point2)>, Failure> {
    let mut best = None;
    for index in 1..=intersections.nb_points()? {
        let point = intersections.point(index)?;
        best = nearer_boundary_hit(best, point.param_on_second(), point.value(), parameter);
    }
    Ok(best)
}

fn closest_intersection_segments(
    intersections: &GInter,
    parameter: f64,
    mut best: Option<(f64, Point2)>,
) -> Result<Option<(f64, Point2)>, Failure> {
    for index in 1..=intersections.nb_segments()? {
        best = segment_endpoint_hit(intersections.segment(index)?, parameter, best)?;
    }
    Ok(best)
}

fn segment_endpoint_hit(
    segment: rustycad_tkgeomalgo::int_res2d::IntersectionSegment,
    parameter: f64,
    mut best: Option<(f64, Point2)>,
) -> Result<Option<(f64, Point2)>, Failure> {
    if segment.has_first_point() {
        let point = segment.first_point()?;
        best = nearer_boundary_hit(best, point.param_on_second(), point.value(), parameter);
    }
    if segment.has_last_point() {
        let point = segment.last_point()?;
        best = nearer_boundary_hit(best, point.param_on_second(), point.value(), parameter);
    }
    Ok(best)
}

fn nearer_boundary_hit(
    best: Option<(f64, Point2)>,
    parameter: f64,
    point: Point2,
    target: f64,
) -> Option<(f64, Point2)> {
    if best.is_none_or(|current| (parameter - target).abs() < (current.0 - target).abs()) {
        return Some((parameter, point));
    }
    best
}

fn adjusted_hit(
    entry: &PcurveGap,
    first_edge: bool,
    hit: Option<(f64, Point2)>,
    curve: &Handle<dyn Geom2dCurve>,
) -> Result<Option<(f64, Point2)>, Failure> {
    let Some((parameter, point)) = hit else {
        return Ok(None);
    };
    let (take_first, endpoint) = adjustment_reference(entry, first_edge);
    let bounds = (endpoint.min(parameter), endpoint.max(parameter));
    let adjusted = super::fallback::adjust_periodic(curve, take_first, bounds, parameter)?;
    Ok(Some((adjusted, point)))
}

fn adjustment_reference(entry: &PcurveGap, first_edge: bool) -> (bool, f64) {
    if first_edge {
        return (entry.reversed, native_start(entry));
    }
    (!entry.reversed, native_end(entry))
}

fn pair_at(
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    parameters: (f64, f64),
) -> Result<ClippedPair, Failure> {
    Ok(ClippedPair {
        first_parameter: parameters.0,
        second_parameter: parameters.1,
        first_point: first_curve.value(parameters.0)?,
        second_point: second_curve.value(parameters.1)?,
    })
}

fn edge_parameter(pair: &ClippedPair, first_edge: bool) -> f64 {
    if first_edge {
        pair.first_parameter
    } else {
        pair.second_parameter
    }
}

fn entry_parameter(entry: &PcurveGap, first_edge: bool) -> f64 {
    if first_edge {
        native_start(entry)
    } else {
        native_end(entry)
    }
}

fn edge_point(pair: &ClippedPair, first_edge: bool) -> Point2 {
    if first_edge {
        pair.first_point
    } else {
        pair.second_point
    }
}

fn set_edge_parameter(pair: &mut ClippedPair, first_edge: bool, parameter: f64) {
    if first_edge {
        pair.first_parameter = parameter;
    } else {
        pair.second_parameter = parameter;
    }
}

fn set_edge_point(pair: &mut ClippedPair, first_edge: bool, point: Point2) {
    if first_edge {
        pair.first_point = point;
    } else {
        pair.second_point = point;
    }
}

fn outside_old_range(entry: &PcurveGap, parameter: f64) -> bool {
    parameter < entry.first || parameter > entry.last
}

fn clamp_edge(entry: &PcurveGap, first_edge: bool, pair: &mut ClippedPair) {
    if first_edge && entry.reversed && pair.first_parameter > entry.first {
        pair.first_parameter = entry.first;
    }
    if first_edge && !entry.reversed && pair.first_parameter < entry.last {
        pair.first_parameter = entry.last;
    }
    if !first_edge && entry.reversed && pair.second_parameter < entry.last {
        pair.second_parameter = entry.last;
    }
    if !first_edge && !entry.reversed && pair.second_parameter > entry.first {
        pair.second_parameter = entry.first;
    }
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
