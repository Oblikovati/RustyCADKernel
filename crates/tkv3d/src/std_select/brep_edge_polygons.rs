//! Existing edge polygons take precedence over analytical geometry; lower bounds are normalized in native order.
use rustycad_tkbrep::{brep::BRepTool, topods::Edge};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;
pub(super) fn brep_selection_polygon_points(edge: &Edge) -> Result<Option<Vec<Point3>>, Failure> {
    let (polygon, placement) = BRepTool::polygon3d(edge)?;
    if !polygon.is_null() {
        let nodes = polygon.nodes();
        let mut points = Vec::with_capacity(nodes.len());
        for index in nodes.lower()..=nodes.upper() {
            let point = *nodes
                .value(index)
                .map_err(|error| error.into_failure("StdSelect_BRepSelectionTool polygon point"))?;
            points.push(if placement.is_identity() {
                point
            } else {
                point.transformed(placement.transformation())
            });
        }
        return Ok(Some(points));
    }
    brep_selection_triangulated_polygon(edge)
}
fn brep_selection_triangulated_polygon(edge: &Edge) -> Result<Option<Vec<Point3>>, Failure> {
    let (polygon, mesh, placement) = BRepTool::first_polygon_on_triangulation(edge)?;
    if polygon.is_null() {
        return Ok(None);
    }
    let indices = polygon.nodes();
    let mut points = Vec::with_capacity(indices.len());
    for index in indices.lower()..=indices.upper() {
        let point =
            mesh.node(*indices.value(index).map_err(|error| {
                error.into_failure("StdSelect_BRepSelectionTool polygon index")
            })?)?;
        points.push(if placement.is_identity() {
            point
        } else {
            point.transformed(placement.transformation())
        });
    }
    Ok(Some(points))
}
