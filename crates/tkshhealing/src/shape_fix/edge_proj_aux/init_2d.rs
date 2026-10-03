use super::EdgeProjAuxState;
use super::parameter_update::update_parameters;
use super::projection::project_pcurve;
use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisSurface, adjust_to_period};
use crate::shape_fix::edge_proj_aux::projection;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Point2, Point3};

#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
struct ProjectionRange {
    first: f64,
    last: f64,
    periodic_u: bool,
    periodic_v: bool,
}

type EdgeProjectionGeometry = (Handle<dyn GeomSurface>, Handle<dyn Geom2dCurve>);
type EndpointPoints = (Point3, Point3);
type CurveAndRange = (Handle<dyn GeomCurve>, (f64, f64));
type EndpointGeometry = (EndpointPoints, Option<CurveAndRange>);

pub(super) fn initialize(state: &mut EdgeProjAuxState, tolerance: f64) -> Result<(), Failure> {
    state.first_done = false;
    state.last_done = false;
    let (surface, curve) = input_geometry(state)?;
    if curve.is_null() {
        return Ok(());
    }
    state.first_param = 0.0;
    state.last_param = 0.0;
    let vertices = topexp::vertices(&state.edge, false);
    let (points, curve3d) = endpoint_points(state, &vertices)?;
    if degenerate_endpoints(state, &surface, &curve, points.0, &vertices, tolerance)? {
        return Ok(());
    }
    project_endpoints(state, surface, curve, curve3d, points, tolerance)
}

fn input_geometry(state: &EdgeProjAuxState) -> Result<EdgeProjectionGeometry, Failure> {
    let surface = BRepTool::surface(&state.face)?;
    let curve = BRepTool::curve_on_face(&state.edge, &state.face)?.pcurve;
    Ok((surface, curve))
}

fn endpoint_points(
    state: &EdgeProjAuxState,
    vertices: &(
        rustycad_tkbrep::topods::Vertex,
        rustycad_tkbrep::topods::Vertex,
    ),
) -> Result<EndpointGeometry, Failure> {
    let (curve, first, last) = ShapeAnalysisEdge::new().curve_3d(&state.edge, false)?;
    if curve.is_some() {
        let geometry = curve.get().ok_or_else(null_curve)?;
        let first_point = geometry.value(first)?;
        let last_point = geometry.value(last)?;
        return Ok(((first_point, last_point), Some((curve, (first, last)))));
    }
    let points = (BRepTool::pnt(&vertices.0)?, BRepTool::pnt(&vertices.1)?);
    Ok((points, None))
}

fn null_curve() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_EdgeProjAux: 3D curve vanished; expected the previously detected edge curve",
    )
}

fn degenerate_endpoints(
    state: &mut EdgeProjAuxState,
    surface: &Handle<dyn GeomSurface>,
    curve: &Handle<dyn Geom2dCurve>,
    point: Point3,
    vertices: &(
        rustycad_tkbrep::topods::Vertex,
        rustycad_tkbrep::topods::Vertex,
    ),
    tolerance: f64,
) -> Result<bool, Failure> {
    if !vertices.0.is_same(&vertices.1) {
        return Ok(false);
    }
    let analysis = ShapeAnalysisSurface::new(surface)?;
    let Some(singularity) = analysis.degenerated_values(point, tolerance, true)? else {
        return Ok(false);
    };
    accept_degenerate_line(state, curve, &singularity, tolerance)
}

fn accept_degenerate_line(
    state: &mut EdgeProjAuxState,
    curve: &Handle<dyn Geom2dCurve>,
    singularity: &crate::shape_analysis::SurfaceSingularity,
    tolerance: f64,
) -> Result<bool, Failure> {
    if curve.cast::<Geom2dLine>().is_null() {
        return Ok(false);
    }
    let geometry = curve.get().ok_or_else(null_curve)?;
    let first_distance = geometry
        .value(singularity.first_parameter)?
        .distance(singularity.first_uv);
    if !distance_is_within(first_distance, tolerance) {
        return Ok(false);
    }
    let last_distance = geometry
        .value(singularity.last_parameter)?
        .distance(singularity.last_uv);
    if !distance_is_within(last_distance, tolerance) {
        return Ok(false);
    }
    state.first_param = singularity.first_parameter;
    state.last_param = singularity.last_parameter;
    state.first_done = true;
    state.last_done = true;
    Ok(true)
}

fn project_endpoints(
    state: &mut EdgeProjAuxState,
    surface: Handle<dyn GeomSurface>,
    curve: Handle<dyn Geom2dCurve>,
    curve3d: Option<(Handle<dyn GeomCurve>, (f64, f64))>,
    points: (Point3, Point3),
    tolerance: f64,
) -> Result<(), Failure> {
    let (curve, range) = clipped_curve(&curve, &surface)?;
    let first = project_pcurve(
        &curve,
        &surface,
        (range.first, range.last),
        points.0,
        tolerance,
    )?;
    if precision::is_infinite(first.0) {
        return Ok(());
    }
    state.first_param = first.1;
    state.first_done = true;
    let last = project_pcurve(
        &curve,
        &surface,
        (range.first, range.last),
        points.1,
        tolerance,
    )?;
    if precision::is_infinite(last.0) {
        return Ok(());
    }
    state.last_param = last.1;
    state.last_done = true;
    adjust_periodic_endpoints(state, &curve, &surface, curve3d, range, tolerance)
}

fn clipped_curve(
    curve: &Handle<dyn Geom2dCurve>,
    surface: &Handle<dyn GeomSurface>,
) -> Result<(Handle<dyn Geom2dCurve>, ProjectionRange), Failure> {
    let range = natural_range(curve);
    if !precision::is_infinite(range.first) && !precision::is_infinite(range.last) {
        return Ok((curve.clone(), range));
    }
    if let Some(line) = curve.cast::<Geom2dLine>().get() {
        return clip_line(line, curve, surface, range);
    }
    reparameterized_bspline(curve, range)
}

fn natural_range(curve: &Handle<dyn Geom2dCurve>) -> ProjectionRange {
    let (first, last) = curve.get().map_or((0.0, 0.0), |curve| {
        (curve.first_parameter(), curve.last_parameter())
    });
    ProjectionRange {
        first,
        last,
        periodic_u: false,
        periodic_v: false,
    }
}

fn reparameterized_bspline(
    curve: &Handle<dyn Geom2dCurve>,
    mut range: ProjectionRange,
) -> Result<(Handle<dyn Geom2dCurve>, ProjectionRange), Failure> {
    range.first = -10000.0;
    range.last = 10000.0;
    let spline = curve.cast::<Geom2dBSplineCurve>();
    let Some(spline) = spline.get() else {
        return Ok((curve.clone(), range));
    };
    let copy = Geom2dBSplineCurve::copy_of(spline)?;
    let mut knots = copy.knots();
    bsplclib::reparametrize(range.first, range.last, &mut knots)?;
    copy.set_knots(&knots)?;
    Ok((Handle::new(copy).cast(), range))
}

fn clip_line(
    line: &Geom2dLine,
    original: &Handle<dyn Geom2dCurve>,
    surface: &Handle<dyn GeomSurface>,
    mut range: ProjectionRange,
) -> Result<(Handle<dyn Geom2dCurve>, ProjectionRange), Failure> {
    let (mut u_first, mut u_last, mut v_first, mut v_last) = surface.bounds();
    hyperbolic_bounds(
        surface,
        &mut u_first,
        &mut u_last,
        &mut v_first,
        &mut v_last,
    )?;
    if finite_bounds((u_first, u_last, v_first, v_last)) {
        clip_finite_line(line, &mut range, (u_first, u_last, v_first, v_last));
    } else if !precision::is_infinite(u_first) && !precision::is_infinite(u_last) {
        clip_u_line(line, &mut range, (u_first, u_last));
    } else {
        range.first = -10000.0;
        range.last = 10000.0;
    }
    Ok((original.clone(), range))
}

fn finite_bounds(bounds: (f64, f64, f64, f64)) -> bool {
    !precision::is_infinite(bounds.0)
        && !precision::is_infinite(bounds.1)
        && !precision::is_infinite(bounds.2)
        && !precision::is_infinite(bounds.3)
}

fn hyperbolic_bounds(
    surface: &Handle<dyn GeomSurface>,
    u_first: &mut f64,
    u_last: &mut f64,
    v_first: &mut f64,
    v_last: &mut f64,
) -> Result<(), Failure> {
    let adapted = GeomAdaptorSurface::with_surface(surface)?;
    let kind = adapted.get_type()?;
    if kind == SurfaceType::SurfaceOfExtrusion && basis_is_hyperbola(&adapted)? {
        *u_first = native_max(*u_first, -23.0);
        *u_last = native_min(*u_last, 23.0);
    }
    if kind == SurfaceType::SurfaceOfRevolution && basis_is_hyperbola(&adapted)? {
        *v_first = native_max(*v_first, -23.0);
        *v_last = native_min(*v_last, 23.0);
    }
    Ok(())
}

fn basis_is_hyperbola(surface: &GeomAdaptorSurface) -> Result<bool, Failure> {
    let basis = surface.basis_curve()?;
    Ok(basis.get().is_some_and(|curve| {
        curve
            .get_type()
            .is_ok_and(|kind| kind == CurveType::Hyperbola)
    }))
}

fn native_max(first: f64, second: f64) -> f64 {
    if first < second { second } else { first }
}

fn native_min(first: f64, second: f64) -> f64 {
    if second < first { second } else { first }
}

fn distance_is_within(distance: f64, tolerance: f64) -> bool {
    matches!(
        distance.partial_cmp(&tolerance),
        Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)
    )
}

fn clip_finite_line(line: &Geom2dLine, range: &mut ProjectionRange, bounds: (f64, f64, f64, f64)) {
    let location = line.location();
    let direction = line.direction();
    range.periodic_u = direction.y() == 0.0;
    range.periodic_v = direction.x() == 0.0;
    let candidates = line_intersections(location, direction, bounds);
    (range.first, range.last) = if candidates.0 < candidates.1 {
        candidates
    } else {
        (candidates.1, candidates.0)
    };
}

fn line_intersections(
    point: Point2,
    direction: rustycad_tkmath::gp::Direction2,
    bounds: (f64, f64, f64, f64),
) -> (f64, f64) {
    let (uf, ul, vf, vl) = bounds;
    let (dx, dy) = (direction.x(), direction.y());
    if dy == 0.0 {
        return ((uf - point.x()) / dx, (ul - point.x()) / dx);
    }
    if dx == 0.0 {
        return ((vf - point.y()) / dy, (vl - point.y()) / dy);
    }
    diagonal_intersections(point, direction, bounds)
}

fn diagonal_intersections(
    point: Point2,
    direction: rustycad_tkmath::gp::Direction2,
    bounds: (f64, f64, f64, f64),
) -> (f64, f64) {
    let (uf, ul, vf, vl) = bounds;
    let (dx, dy) = (direction.x(), direction.y());
    let (xf, xl, yf, yl) = (
        (uf - point.x()) / dx,
        (ul - point.x()) / dx,
        (vf - point.y()) / dy,
        (vl - point.y()) / dy,
    );
    if dx * dy > 0.0 {
        (
            if (xl - xf).abs() < (xl - yf).abs() {
                xf
            } else {
                yf
            },
            if (xf - xl).abs() < (xf - yl).abs() {
                xl
            } else {
                yl
            },
        )
    } else {
        (
            if (xl - xf).abs() < (xl - yl).abs() {
                xf
            } else {
                yl
            },
            if (yl - xl).abs() < (yl - yf).abs() {
                xl
            } else {
                yf
            },
        )
    }
}

fn clip_u_line(line: &Geom2dLine, range: &mut ProjectionRange, bounds: (f64, f64)) {
    let direction = line.direction();
    if direction.x() == 0.0 {
        range.first = -10000.0;
        range.last = 10000.0;
        return;
    }
    range.periodic_u = direction.y() == 0.0;
    let point = line.location();
    let first = (bounds.0 - point.x()) / direction.x();
    let last = (bounds.1 - point.x()) / direction.x();
    (range.first, range.last) = if first < last {
        (first, last)
    } else {
        (last, first)
    };
}

fn adjust_periodic_endpoints(
    state: &mut EdgeProjAuxState,
    curve: &Handle<dyn Geom2dCurve>,
    surface: &Handle<dyn GeomSurface>,
    curve3d: Option<(Handle<dyn GeomCurve>, (f64, f64))>,
    range: ProjectionRange,
    tolerance: f64,
) -> Result<(), Failure> {
    if (state.first_param - state.last_param).abs() < precision::p_confusion()
        && !surface.is_u_periodic()
        && !surface.is_v_periodic()
    {
        return Ok(());
    }
    if reverse_pcurve_for_endpoint_order(state, curve, range)? {
        return Ok(());
    }
    let adaptor = projection::adapt_geometry(curve, surface, (range.first, range.last))?;
    repair_closed_surface_endpoints(state, &adaptor, range)?;
    if !(range.periodic_u || range.periodic_v) {
        return update_parameters(state, curve);
    }
    adjust_periodic_line(state, curve, surface, curve3d, range, tolerance)
}

fn repair_closed_surface_endpoints(
    state: &mut EdgeProjAuxState,
    adaptor: &impl Adaptor3dCurve,
    range: ProjectionRange,
) -> Result<(), Failure> {
    let first = adaptor.value(range.first)?;
    let last = adaptor.value(range.last)?;
    let is_closed = first.distance(&last) < precision::confusion();
    if !is_closed {
        return Ok(());
    }
    let tolerance = precision::p_confusion();
    if (state.first_param - range.first).abs() < tolerance
        && (state.last_param - range.first).abs() < tolerance
    {
        state.last_param = range.last;
    } else if (state.first_param - range.last).abs() < tolerance
        && (state.last_param - range.last).abs() < tolerance
    {
        state.first_param = range.first;
    }
    Ok(())
}

fn reverse_pcurve_for_endpoint_order(
    state: &mut EdgeProjAuxState,
    curve: &Handle<dyn Geom2dCurve>,
    range: ProjectionRange,
) -> Result<bool, Failure> {
    if state.first_param == range.first && state.last_param == range.last {
        return Ok(true);
    }
    if state.first_param != range.last || state.last_param != range.first {
        return Ok(false);
    }
    let geometry = curve.get().ok_or_else(null_curve)?;
    state.first_param = geometry.reversed_parameter(range.last)?;
    state.last_param = geometry.reversed_parameter(range.first)?;
    geometry.reverse()?;
    Ok(true)
}

fn adjust_periodic_line(
    state: &mut EdgeProjAuxState,
    curve: &Handle<dyn Geom2dCurve>,
    surface: &Handle<dyn GeomSurface>,
    curve3d: Option<(Handle<dyn GeomCurve>, (f64, f64))>,
    range: ProjectionRange,
    tolerance: f64,
) -> Result<(), Failure> {
    let bounds = surface.bounds();
    let period = if range.periodic_u {
        bounds.1 - bounds.0
    } else {
        bounds.3 - bounds.2
    };
    state.first_param += adjust_to_period(state.first_param, 0.0, period);
    state.last_param += adjust_to_period(state.last_param, 0.0, period);
    let Some((curve3d, curve_range)) = curve3d else {
        return update_parameters(state, curve);
    };
    let middle = projection::midpoint(&curve3d, curve_range)?;
    let (_, middle_parameter) =
        project_pcurve(curve, surface, (range.first, range.last), middle, tolerance)?;
    let middle_parameter = middle_parameter + adjust_to_period(middle_parameter, 0.0, period);
    choose_periodic_span(state, middle_parameter, period, curve)
}

fn choose_periodic_span(
    state: &mut EdgeProjAuxState,
    middle: f64,
    period: f64,
    curve: &Handle<dyn Geom2dCurve>,
) -> Result<(), Failure> {
    if state.first_param >= state.last_param {
        if state.last_param > middle {
            state.first_param -= period;
        } else if state.first_param > middle {
            update_parameters(state, curve)?;
        } else {
            state.last_param += period;
        }
    } else if state.first_param > middle {
        state.last_param -= period;
        update_parameters(state, curve)?;
    } else if state.last_param < middle {
        state.first_param += period;
        update_parameters(state, curve)?;
    }
    update_parameters(state, curve)
}
