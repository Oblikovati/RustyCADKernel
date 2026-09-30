//! Native edge copy keeps an existing spatial parameter when both edge ranges match.
use super::*;
struct SourceVertexCurve {
    curve: Handle<dyn GeomCurve>,
    location: Location,
    bounds: (f64, f64),
}
pub(super) fn copy_vertex(
    vertex: &Vertex,
    target: &Edge,
    source: &Edge,
) -> Result<Vertex, Failure> {
    let old = source_curve(source, vertex)?;
    let (curve, first, last) = BRepTool::curve(target)?;
    let copy = topods::vertex(&vertex.empty_copied()?)?;
    let (parameter, has_spatial) = copy_points(vertex, &copy, source, &old)?;
    let tolerance = BRepTool::vertex_tolerance(vertex)?;
    let (parameter, tolerance) = parameter_on_target(
        &copy,
        &old,
        &curve,
        (first, last),
        parameter,
        has_spatial,
        tolerance,
    )?;
    BRepBuilder::default().update_vertex_on_edge(&copy, parameter, target, tolerance)?;
    update_pcurve_tolerance(&copy, target, parameter, tolerance)?;
    Ok(copy)
}
fn source_curve(edge: &Edge, vertex: &Vertex) -> Result<SourceVertexCurve, Failure> {
    let old = BRepTool::curve_with_location(edge)?;
    Ok(SourceVertexCurve {
        curve: old.curve,
        location: old.location.predivided(vertex.location())?,
        bounds: (old.first, old.last),
    })
}
fn copy_points(
    source: &Vertex,
    target: &Vertex,
    edge: &Edge,
    old: &SourceVertexCurve,
) -> Result<(f64, bool), Failure> {
    let (mut parameter, mut has_spatial) = (f64::MAX, false);
    for point in vertex_tshape(source, "TransferParametersProj::CopyNMVertex")?.points() {
        if point.is_null() {
            continue;
        }
        if point.is_point_on_curve_at(&old.curve, &old.location) {
            parameter = point.parameter();
            has_spatial = true;
            continue;
        }
        if point.is_point_on_curve_on_surface() && matches_source_pcurve(&point, edge, source)? {
            if !has_spatial {
                parameter = point.parameter();
            }
            continue;
        }
        append_copy(target, &point)?;
    }
    Ok((parameter, has_spatial))
}
fn matches_source_pcurve(
    point: &Handle<dyn BRepPointRepresentation>,
    edge: &Edge,
    vertex: &Vertex,
) -> Result<bool, Failure> {
    for representation in edge_tshape(edge, "TransferParametersProj::CopyNMVertex")?.curves() {
        let curve = as_gcurve(&representation);
        if curve.is_null() || !curve.is_curve_on_surface() {
            continue;
        }
        // The native match uses the representation location without composing the edge location.
        let location = curve.location().predivided(vertex.location())?;
        if point.is_point_on_curve_on_surface_at(&curve.pcurve()?, &curve.surface()?, &location) {
            return Ok(true);
        }
    }
    Ok(false)
}
fn parameter_on_target(
    vertex: &Vertex,
    old: &SourceVertexCurve,
    curve: &Handle<dyn GeomCurve>,
    bounds: (f64, f64),
    parameter: f64,
    has_spatial: bool,
    mut tolerance: f64,
) -> Result<(f64, f64), Failure> {
    let changed = (old.bounds.0 - bounds.0).abs() > precision::p_confusion()
        || (old.bounds.1 - bounds.1).abs() > precision::p_confusion();
    if has_spatial && !changed {
        return Ok((parameter, tolerance));
    }
    let point = BRepTool::pnt(vertex)?;
    let (distance, _, parameter) =
        ShapeAnalysisCurve.project_full(curve, point, precision::confusion(), true)?;
    if tolerance < distance {
        tolerance = distance;
    }
    Ok((parameter, tolerance))
}
fn update_pcurve_tolerance(
    vertex: &Vertex,
    edge: &Edge,
    parameter: f64,
    mut tolerance: f64,
) -> Result<(), Failure> {
    let point = vertex_tshape(vertex, "TransferParametersProj::CopyNMVertex")?.pnt();
    let mut changed = false;
    for representation in edge_tshape(edge, "TransferParametersProj::CopyNMVertex")?.curves() {
        let curve = as_gcurve(&representation);
        if curve.is_null() || !curve.is_curve_on_surface() {
            continue;
        }
        let distance = pcurve_distance(vertex, edge, &curve, parameter, point)?;
        if distance > tolerance {
            tolerance = distance;
            changed = true;
        }
    }
    if changed {
        BRepBuilder::default().update_vertex_tolerance(vertex, tolerance)?;
    }
    Ok(())
}

fn pcurve_distance(
    vertex: &Vertex,
    edge: &Edge,
    curve: &Handle<dyn rustycad_tkbrep::brep::BRepGCurve>,
    parameter: f64,
    point: Point3,
) -> Result<f64, Failure> {
    let location = (edge.location() * &curve.location())?.predivided(vertex.location())?;
    let uv = curve.pcurve()?.value(parameter)?;
    let position = curve
        .surface()?
        .value(uv.x(), uv.y())?
        .transformed(location.transformation());
    Ok(point.distance(&position))
}
