//! The native existing-polygon decision and edge-status transitions.
use super::*;
use crate::imesh_data::PCurveHandle;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkmath::poly::mesh_purpose;
pub(super) fn discretize_edge(
    edge: &EdgeHandle,
    maximum: f64,
    parameters: &MeshParameters,
) -> Result<(), Failure> {
    Deflection::compute_edge(edge, maximum, parameters)?;
    let tessellator = if edge.is_free() {
        if reuse_free_polygon(edge, parameters)? {
            return Ok(());
        }
        BRepMeshEdgeDiscret::create_edge_tessellator(edge, parameters, 2)?
    } else {
        face_tessellator(edge, parameters)?
    };
    BRepMeshEdgeDiscret::tessellate_3d(edge, &tessellator, true)?;
    if !edge.is_free() {
        BRepMeshEdgeDiscret::tessellate_2d(edge, true)?;
    }
    Ok(())
}
fn reuse_free_polygon(edge: &EdgeHandle, parameters: &MeshParameters) -> Result<bool, Failure> {
    let (polygon, _) = BRepTool::polygon3d(&edge.get_edge()?)?;
    if polygon.is_null() {
        return Ok(false);
    }
    let consistent = polygon_fits(
        polygon.has_parameters(),
        polygon.deflection(),
        edge,
        parameters,
    );
    edge.status().set_status(if consistent {
        Status::Reused
    } else {
        Status::Outdated
    });
    Ok(consistent)
}
fn face_tessellator(
    edge: &EdgeHandle,
    parameters: &MeshParameters,
) -> Result<Handle<dyn CurveTessellator>, Failure> {
    let (minimum, selected) = smoothest_polygon(edge, parameters)?;
    if selected != -1 {
        edge.set_deflection(minimum);
        return BRepMeshEdgeDiscret::create_edge_tessellation_extractor(
            edge,
            &edge.get_pcurve(selected)?.get_face().upgrade(),
        );
    }
    let pcurve = edge.get_pcurve(0)?;
    BRepMeshEdgeDiscret::create_face_tessellator(
        edge,
        pcurve.get_orientation(),
        &pcurve.get_face().upgrade(),
        parameters,
        2,
    )
}
fn smoothest_polygon(
    edge: &EdgeHandle,
    parameters: &MeshParameters,
) -> Result<(f64, i32), Failure> {
    let mut minimum = f64::MAX;
    let mut selected = -1;
    for index in 0..edge.pcurves_nb() {
        let pcurve = edge.get_pcurve(index)?;
        let deflection = existing_polygon_deflection(edge, &pcurve, parameters)?;
        if deflection < minimum {
            minimum = deflection;
            selected = index;
        }
        ShapeTool::check_and_update_flags(edge, &pcurve)?;
    }
    Ok((minimum, selected))
}
fn existing_polygon_deflection(
    edge: &EdgeHandle,
    pcurve: &PCurveHandle,
    parameters: &MeshParameters,
) -> Result<f64, Failure> {
    let face = pcurve.get_face().upgrade().get_face()?;
    let (triangulation, location) = BRepTool::triangulation(&face, mesh_purpose::NONE)?;
    if triangulation.is_null() {
        return Ok(f64::MAX);
    }
    let polygon = BRepTool::polygon_on_triangulation(&edge.get_edge()?, &triangulation, &location)?;
    if polygon.is_null() {
        return Ok(f64::MAX);
    }
    if polygon_fits(
        polygon.has_parameters(),
        polygon.deflection(),
        edge,
        parameters,
    ) {
        return Ok(polygon.deflection());
    }
    edge.status().set_status(Status::Outdated);
    Ok(f64::MAX)
}

fn polygon_fits(
    has_parameters: bool,
    deflection: f64,
    edge: &EdgeHandle,
    parameters: &MeshParameters,
) -> bool {
    has_parameters
        && Deflection::is_consistent(
            deflection,
            edge.get_deflection(),
            parameters.allow_quality_decrease,
            0.1,
        )
}
