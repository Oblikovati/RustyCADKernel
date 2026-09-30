//! Face transfer retains UV only when the native surface/location identity checks agree.
use super::*;
use crate::shape_analysis::ShapeAnalysisSurface;
pub(super) fn copy_vertex(
    vertex: &Vertex,
    target: &Face,
    source: &Face,
) -> Result<Vertex, Failure> {
    let (from_surface, from_location) = BRepTool::surface_with_location(source)?;
    let (to_surface, to_location) = BRepTool::surface_with_location(target)?;
    let relative = from_location.predivided(vertex.location())?;
    let copy = topods::vertex(&vertex.empty_copied()?)?;
    let parameters = copy_points(vertex, &copy, &from_surface, &relative)?;
    let mut tolerance = BRepTool::vertex_tolerance(&copy)?;
    let uv = if let Some(uv) =
        parameters.filter(|_| from_surface == to_surface && relative == to_location)
    {
        uv
    } else {
        project_vertex(&copy, target, &mut tolerance)?
    };
    BRepBuilder::default().update_vertex_on_face(&copy, uv.0, uv.1, target, tolerance)?;
    Ok(copy)
}
fn copy_points(
    source: &Vertex,
    target: &Vertex,
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
) -> Result<Option<(f64, f64)>, Failure> {
    let mut parameters = None;
    for representation in vertex_tshape(source, "TransferParametersProj::CopyNMVertex")?.points() {
        if representation.is_null() {
            continue;
        }
        if representation.is_point_on_surface_at(surface, location) {
            parameters = Some((representation.parameter(), representation.parameter2()?));
        } else {
            append_copy(target, &representation)?;
        }
    }
    Ok(parameters)
}
fn project_vertex(
    vertex: &Vertex,
    face: &Face,
    tolerance: &mut f64,
) -> Result<(f64, f64), Failure> {
    let point = BRepTool::pnt(vertex)?;
    let surface = BRepTool::surface(face)?;
    let tool = ShapeAnalysisSurface::new(&surface)?;
    let uv = tool.value_of_uv(point, precision::confusion())?;
    if *tolerance < tool.gap() {
        *tolerance = tool.gap() + 0.1 * precision::confusion();
    }
    Ok((uv.x(), uv.y()))
}
