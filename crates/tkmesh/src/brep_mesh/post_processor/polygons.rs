//! Commit free and surface-associated edge polygons in native insertion order.
use super::*;
use crate::{
    brep_mesh::ShapeTool,
    imesh_data::{EdgeHandle, PCurveHandle},
};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::poly::{Polygon3D, PolygonOnTriangulation};
pub(super) fn commit_edge(edge: &EdgeHandle) -> Result<(), Failure> {
    if edge.get_curve().parameters_nb() == 0 {
        return Ok(());
    }
    if !edge.is_free() {
        return commit_pcurves(edge);
    }
    if edge.status().is_set(Status::Reused) {
        return Ok(());
    }
    commit_polygon3d(edge)
}
fn commit_polygon3d(edge: &EdgeHandle) -> Result<(), Failure> {
    let curve = edge.get_curve();
    let mut nodes = Array1::with_bounds(1, curve.parameters_nb())
        .map_err(|e| e.into_failure("commitPolygon3D"))?;
    let mut parameters = Array1::with_bounds(1, curve.parameters_nb())
        .map_err(|e| e.into_failure("commitPolygon3D"))?;
    for index in 1..=curve.parameters_nb() {
        nodes[index] = curve.get_point(index - 1)?;
        parameters[index] = curve.get_parameter(index - 1)?;
    }
    let polygon = Handle::new(Polygon3D::from_nodes_and_parameters(&nodes, &parameters)?);
    polygon.set_deflection(edge.get_deflection());
    ShapeTool::update_edge_polygon3d(&edge.get_edge()?, &polygon)
}
fn commit_pcurves(edge: &EdgeHandle) -> Result<(), Failure> {
    let mut groups: Vec<(FaceHandle, Vec<PCurveHandle>)> = Vec::new();
    for index in 0..edge.pcurves_nb() {
        let pcurve = edge.get_pcurve(index)?;
        let face = pcurve.get_face().upgrade();
        if skip_face(&face) {
            continue;
        }
        if let Some((_, curves)) = groups.iter_mut().find(|(key, _)| *key == face) {
            curves.push(pcurve);
        } else {
            groups.push((face, vec![pcurve]));
        }
    }
    for (face, curves) in groups {
        commit_face_polygons(edge, &face, &curves)?;
    }
    Ok(())
}
fn commit_face_polygons(
    edge: &EdgeHandle,
    face: &FaceHandle,
    curves: &[PCurveHandle],
) -> Result<(), Failure> {
    let (triangulation, location) = BRepTool::triangulation(&face.get_face()?, mesh_purpose::NONE)?;
    if triangulation.is_null() {
        return Ok(());
    }
    let first = collect_polygon(&curves[0], edge.get_deflection())?;
    let topology = edge.get_edge()?;
    if curves.len() == 2 {
        let last = collect_polygon(&curves[1], edge.get_deflection())?;
        return ShapeTool::update_edge_seam(&topology, (&first, &last), &triangulation, &location);
    }
    ShapeTool::update_edge_on_triangulation(&topology, &first, &triangulation, &location)
}
fn collect_polygon(
    pcurve: &PCurveHandle,
    deflection: f64,
) -> Result<Handle<PolygonOnTriangulation>, Failure> {
    let polygon = Handle::new(PolygonOnTriangulation::new(pcurve.parameters_nb(), true)?);
    for index in 1..=pcurve.parameters_nb() {
        polygon.set_node(index, pcurve.get_index(index - 1)?)?;
        polygon.set_parameter(index, pcurve.get_parameter(index - 1)?)?;
    }
    polygon.set_deflection(deflection);
    Ok(polygon)
}
