use super::pcurve_circle::merge_same_support_circles;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Direction2, Line2, Point2, Vector2};

// Native only enables its duplicate seam pass when the chain has one nonplanar support.

/// Mirrors native support selection and p-curve union for each merged edge chain.
pub(super) fn union_pcurves(
    shape: &Shape,
    first: &Edge,
    second: &Edge,
    merged: &Edge,
) -> Result<(), Failure> {
    let supports = nonplanar_supports(shape, first)?;
    let detect_seam = supports.len() == 1;
    for face in supports {
        union_pcurves_on_face(&face, first, second, merged, detect_seam)?;
    }
    Ok(())
}

fn nonplanar_supports(shape: &Shape, first: &Edge) -> Result<Vec<Face>, Failure> {
    let mut supports = Vec::new();
    for candidate in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        let face = rustycad_tkbrep::topods::face(&candidate)?;
        if !contains_edge(&face, first)? {
            continue;
        }
        if BRepAdaptorSurface::with_face(&face, false)?.get_type()? == SurfaceType::Plane {
            continue;
        }
        add_unique_support(&mut supports, face)?;
    }
    Ok(supports)
}

fn add_unique_support(supports: &mut Vec<Face>, face: Face) -> Result<(), Failure> {
    let (surface, location) = BRepTool::surface_with_location(&face)?;
    for existing in supports.iter() {
        let (known_surface, known_location) = BRepTool::surface_with_location(existing)?;
        if surface == known_surface && location == known_location {
            return Ok(());
        }
    }
    supports.push(face);
    Ok(())
}

fn contains_edge(face: &Face, edge: &Edge) -> Result<bool, Failure> {
    for candidate in Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )? {
        if rustycad_tkbrep::topods::edge(&candidate)?.is_same(edge) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn union_pcurves_on_face(
    face: &Face,
    first: &Edge,
    second: &Edge,
    merged: &Edge,
    detect_seam: bool,
) -> Result<(), Failure> {
    let seam = detect_seam && is_seam_pair(first, face)?;
    let primary = merged_pcurve_pass(face, merged, first, second, false)?;
    if !seam {
        return store_single_pcurve(face, merged, &primary);
    }
    let secondary = merged_pcurve_pass(face, merged, first, second, true)?;
    store_seam_pcurves(face, merged, &primary, &secondary)
}

fn store_single_pcurve(face: &Face, merged: &Edge, result: &MergedPCurve) -> Result<(), Failure> {
    let (surface, location) = BRepTool::surface_with_location(face)?;
    let tolerance = BRepTool::edge_tolerance(merged)?;
    let builder = BRepBuilder::default();
    builder.update_edge_pcurve(merged, &result.curve, &surface, &location, tolerance, None)?;
    builder.range_on_surface(merged, &surface, &location, result.range.0, result.range.1)
}

fn store_seam_pcurves(
    face: &Face,
    merged: &Edge,
    primary: &MergedPCurve,
    secondary: &MergedPCurve,
) -> Result<(), Failure> {
    let (surface, location) = BRepTool::surface_with_location(face)?;
    let tolerance = BRepTool::edge_tolerance(merged)?;
    let builder = BRepBuilder::default();
    builder.update_edge_pcurves(
        merged,
        (&primary.curve, &secondary.curve),
        &surface,
        &location,
        tolerance,
        None,
    )?;
    builder.range_on_surface(
        merged,
        &surface,
        &location,
        primary.range.0,
        primary.range.1,
    )
}

fn merged_pcurve_pass(
    face: &Face,
    merged: &Edge,
    first: &Edge,
    second: &Edge,
    reverse_occurrences: bool,
) -> Result<MergedPCurve, Failure> {
    let first = stored_oriented_pcurve(first, face, reverse_occurrences)?;
    let second = stored_oriented_pcurve(second, face, reverse_occurrences)?;
    if let Some(result) = merge_same_support_lines(merged, &first, &second)? {
        return Ok(result);
    }
    if let Some(result) = merge_same_support_circle_pair(face, merged, &first, &second)? {
        return Ok(result);
    }
    merge_matching_or_general_curve(merged, &first, &second)
}

fn merge_matching_or_general_curve(
    merged: &Edge,
    first: &OrientedPCurve,
    second: &OrientedPCurve,
) -> Result<MergedPCurve, Failure> {
    if same_pcurve_handle(first, second) {
        return single_pcurve_result(first);
    }
    if has_one_stored_curve(first, second) {
        let source = if first.pcurve.is_stored {
            first
        } else {
            second
        };
        return single_pcurve_result(source);
    }
    general_pcurve_result(merged, first, second)
}

fn general_pcurve_result(
    merged: &Edge,
    first: &OrientedPCurve,
    second: &OrientedPCurve,
) -> Result<MergedPCurve, Failure> {
    let Some(spline) = super::pcurve_concat::concatenate_general_curves(merged, first, second)?
    else {
        return single_pcurve_result(first);
    };
    let first_parameter = spline.first_parameter();
    let last_parameter = spline.last_parameter();
    Ok(MergedPCurve {
        curve: spline.cast(),
        range: (first_parameter, last_parameter),
    })
}

pub(super) struct MergedPCurve {
    pub(super) curve: Handle<dyn Geom2dCurve>,
    pub(super) range: (f64, f64),
}

fn single_pcurve_result(source: &OrientedPCurve) -> Result<MergedPCurve, Failure> {
    let curve = source.pcurve.pcurve.copy()?.cast::<dyn Geom2dCurve>();
    let mut range = (source.pcurve.first, source.pcurve.last);
    if source.edge_orientation == Orientation::Reversed {
        range = (
            curve
                .get()
                .ok_or_else(null_pcurve)?
                .reversed_parameter(range.1)?,
            curve
                .get()
                .ok_or_else(null_pcurve)?
                .reversed_parameter(range.0)?,
        );
        curve.get().ok_or_else(null_pcurve)?.reverse()?;
    }
    Ok(MergedPCurve { curve, range })
}

fn has_one_stored_curve(first: &OrientedPCurve, second: &OrientedPCurve) -> bool {
    first.pcurve.is_stored != second.pcurve.is_stored
}

fn same_pcurve_handle(first: &OrientedPCurve, second: &OrientedPCurve) -> bool {
    !first.pcurve.pcurve.is_null() && first.pcurve.pcurve == second.pcurve.pcurve
}

fn merge_same_support_lines(
    merged: &Edge,
    first: &OrientedPCurve,
    second: &OrientedPCurve,
) -> Result<Option<MergedPCurve>, Failure> {
    let Some((first_line, _)) = line_pair(first, second)? else {
        return Ok(None);
    };
    let Some(extension) = joined_line_range(second, first_line)? else {
        return Ok(None);
    };
    let range = extend_native_range(first, extension, first.edge_orientation);
    let edge_range = BRepTool::range(merged)?;
    let pcurve = native_oriented_line(first_line, range, edge_range, first.edge_orientation);
    Ok(Some(MergedPCurve {
        curve: Handle::new(pcurve).cast(),
        range: edge_range,
    }))
}

fn merge_same_support_circle_pair(
    face: &Face,
    merged: &Edge,
    first: &OrientedPCurve,
    second: &OrientedPCurve,
) -> Result<Option<MergedPCurve>, Failure> {
    merge_same_support_circles(face, merged, first, second)
}

fn null_pcurve() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "null p-curve; expected stored curve",
        "",
    )
}

fn is_seam_pair(first: &Edge, face: &Face) -> Result<bool, Failure> {
    let mut support = face.clone();
    support.set_orientation(Orientation::Forward);
    let forward = BRepTool::curve_on_face(first, &support)?;
    let mut reversed_occurrence = first.clone();
    reversed_occurrence.reverse();
    let reversed = BRepTool::curve_on_face(&reversed_occurrence, &support)?;
    Ok(forward.pcurve != reversed.pcurve)
}

pub(super) struct OrientedPCurve {
    pub(super) source_edge: Edge,
    pub(super) edge_orientation: Orientation,
    pub(super) pcurve: rustycad_tkbrep::brep::EdgePCurve,
}

fn stored_oriented_pcurve(
    edge: &Edge,
    face: &Face,
    reverse_occurrence: bool,
) -> Result<OrientedPCurve, Failure> {
    let mut support = face.clone();
    support.set_orientation(Orientation::Forward);
    let mut occurrence = edge.clone();
    if reverse_occurrence {
        occurrence.reverse();
    }
    Ok(OrientedPCurve {
        source_edge: edge.clone(),
        edge_orientation: edge.orientation(),
        pcurve: BRepTool::curve_on_face(&occurrence, &support)?,
    })
}

fn line_pair(
    first: &OrientedPCurve,
    second: &OrientedPCurve,
) -> Result<Option<(Line2, Line2)>, Failure> {
    if !has_stored_pcurves(first, second) {
        return Ok(None);
    }
    let Some(first_line) = try_make_line(&first.pcurve)? else {
        return Ok(None);
    };
    let Some(second_line) = try_make_line(&second.pcurve)? else {
        return Ok(None);
    };
    Ok(Some((first_line, second_line)))
}

fn has_stored_pcurves(first: &OrientedPCurve, second: &OrientedPCurve) -> bool {
    first.pcurve.is_stored
        && second.pcurve.is_stored
        && !first.pcurve.pcurve.is_null()
        && !second.pcurve.pcurve.is_null()
}

fn try_make_line(curve: &rustycad_tkbrep::brep::EdgePCurve) -> Result<Option<Line2>, Failure> {
    let adaptor = Geom2dAdaptorCurve::with_curve(&curve.pcurve)?;
    if adaptor.get_type()? == rustycad_tkmath::geom_abs::CurveType::Line {
        return Ok(Some(adaptor.line()?));
    }
    try_sampled_line(curve, adaptor.get_type()?)
}

fn try_sampled_line(
    curve: &rustycad_tkbrep::brep::EdgePCurve,
    curve_type: rustycad_tkmath::geom_abs::CurveType,
) -> Result<Option<Line2>, Failure> {
    if !matches!(
        curve_type,
        rustycad_tkmath::geom_abs::CurveType::BezierCurve
            | rustycad_tkmath::geom_abs::CurveType::BSplineCurve
    ) {
        return Ok(None);
    }
    if !is_line_domain(curve)? {
        return Ok(None);
    }
    let start = curve.pcurve.eval_d0(curve.first)?;
    let end = curve.pcurve.eval_d0(curve.last)?;
    let delta = Vector2::between(start, end);
    sampled_line(curve, start, delta)
}

fn is_line_domain(curve: &rustycad_tkbrep::brep::EdgePCurve) -> Result<bool, Failure> {
    let start = curve.pcurve.eval_d0(curve.first)?;
    let end = curve.pcurve.eval_d0(curve.last)?;
    let delta = Vector2::between(start, end);
    Ok(
        ((delta.square_magnitude()) - (curve.last - curve.first).powi(2)).abs()
            <= precision::confusion(),
    )
}

fn sampled_line(
    curve: &rustycad_tkbrep::brep::EdgePCurve,
    start: Point2,
    delta: Vector2,
) -> Result<Option<Line2>, Failure> {
    let direction = Direction2::new(delta.x(), delta.y())?;
    let line = line_through_parameter_origin(start, direction, curve.first);
    for sample in 1..10 {
        let parameter = sample_parameter(curve, sample);
        if line.distance(curve.pcurve.eval_d0(parameter)?) > precision::confusion() {
            return Ok(None);
        }
    }
    Ok(Some(line))
}

fn line_through_parameter_origin(start: Point2, direction: Direction2, first: f64) -> Line2 {
    let offset = Vector2::new(-direction.x() * first, -direction.y() * first);
    Line2::new(start.translated(offset), direction)
}

fn sample_parameter(curve: &rustycad_tkbrep::brep::EdgePCurve, sample: i32) -> f64 {
    curve.first + f64::from(sample) * (curve.last - curve.first) / 10.0
}

fn joined_line_range(second: &OrientedPCurve, line: Line2) -> Result<Option<(f64, f64)>, Failure> {
    let endpoints = line_endpoints(second)?;
    if endpoints
        .iter()
        .any(|point| !line.contains(*point, precision::confusion()))
    {
        return Ok(None);
    }
    let parameters = endpoints.map(|point| line_parameter(line, point));
    let extension = (
        parameters[0].min(parameters[1]),
        parameters[0].max(parameters[1]),
    );
    Ok(Some(extension))
}

fn line_endpoints(curve: &OrientedPCurve) -> Result<[Point2; 2], Failure> {
    Ok([
        curve.pcurve.pcurve.eval_d0(curve.pcurve.first)?,
        curve.pcurve.pcurve.eval_d0(curve.pcurve.last)?,
    ])
}

fn extend_native_range(
    first: &OrientedPCurve,
    extension: (f64, f64),
    orientation: Orientation,
) -> (f64, f64) {
    if orientation != Orientation::Reversed {
        return (first.pcurve.first, first.pcurve.last.max(extension.1));
    }
    (first.pcurve.first.min(extension.0), first.pcurve.last)
}

fn line_parameter(line: Line2, point: Point2) -> f64 {
    Vector2::between(line.location(), point)
        .dot(Vector2::new(line.direction().x(), line.direction().y()))
}

fn native_oriented_line(
    mut line: Line2,
    mut range: (f64, f64),
    edge_range: (f64, f64),
    orientation: Orientation,
) -> Geom2dLine {
    if orientation == Orientation::Reversed {
        line.reverse();
        range = (-range.1, -range.0);
    }
    if range == edge_range {
        return Geom2dLine::from_lin2d(line);
    }
    let origin = line.location().translated(Vector2::new(
        line.direction().x() * (range.0 - edge_range.0),
        line.direction().y() * (range.0 - edge_range.0),
    ));
    Geom2dLine::from_lin2d(Line2::new(origin, line.direction()))
}

#[cfg(test)]
#[path = "pcurve_union_tests.rs"]
mod pcurve_union_native_tests;
