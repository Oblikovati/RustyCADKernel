//! Native 3D endpoint replacement and parameter-driven 2D insertion.
use super::*;
use crate::{
    brep_mesh::EdgeParameterProvider,
    imesh_data::{CurveHandle, PCurveHandle, ParametersListArrayAdaptor},
};
use rustycad_tkbrep::{brep::BRepTool, topexp};
use rustycad_tkmath::gp::Point3;
impl BRepMeshEdgeDiscret {
    /// Append native endpoints or insert only interior samples; `BRepMeshEdgeDiscret::tessellate_3d(&edge, &tessellator, true)?`.
    pub fn tessellate_3d(
        edge: &EdgeHandle,
        tessellator: &Handle<dyn CurveTessellator>,
        update_ends: bool,
    ) -> Result<(), Failure> {
        let curve = edge.get_curve();
        let (first, last) = topexp::vertices(&edge.get_edge()?, false);
        if first.is_null() || last.is_null() {
            return Ok(());
        }
        if update_ends {
            append_endpoint(&curve, tessellator, 1, &first)?;
        }
        if !edge.get_degenerated() {
            insert_interior(&curve, tessellator, update_ends)?;
        }
        if update_ends {
            append_endpoint(&curve, tessellator, tessellator.points_nb(), &last)?;
        }
        Ok(())
    }
    /// Transfer 3D parameters through each native pcurve; `BRepMeshEdgeDiscret::tessellate_2d(&edge, true)?`.
    pub fn tessellate_2d(edge: &EdgeHandle, update_ends: bool) -> Result<(), Failure> {
        for index in 0..edge.pcurves_nb() {
            tessellate_pcurve(edge, &edge.get_pcurve(index)?, update_ends)?;
        }
        Ok(())
    }
}
fn append_endpoint(
    curve: &CurveHandle,
    tessellator: &Handle<dyn CurveTessellator>,
    index: i32,
    vertex: &rustycad_tkbrep::topods::Vertex,
) -> Result<(), Failure> {
    let mut ignored_point = Point3::default();
    let mut parameter = 0.0;
    tessellator.value(index, &mut ignored_point, &mut parameter)?;
    curve.add_point(BRepTool::pnt(vertex)?, parameter);
    Ok(())
}
fn insert_interior(
    curve: &CurveHandle,
    tessellator: &Handle<dyn CurveTessellator>,
    update_ends: bool,
) -> Result<(), Failure> {
    for index in 2..tessellator.points_nb() {
        let mut point = Point3::default();
        let mut parameter = 0.0;
        if !tessellator.value(index, &mut point, &mut parameter)? {
            continue;
        }
        if update_ends {
            curve.add_point(point, parameter);
        } else {
            curve.insert_point(curve.parameters_nb() - 1, point, parameter)?;
        }
    }
    Ok(())
}
fn tessellate_pcurve(
    edge: &EdgeHandle,
    pcurve: &PCurveHandle,
    update_ends: bool,
) -> Result<(), Failure> {
    let curve = edge.get_curve();
    let face = pcurve.get_face().upgrade();
    let array = Handle::new(ParametersListArrayAdaptor::new(curve.cast()));
    let provider = EdgeParameterProvider::with_edge(edge, pcurve.get_orientation(), &face, array)?;
    transfer_pcurve_parameters(pcurve, &provider, &curve, update_ends)
}
fn transfer_pcurve_parameters(
    pcurve: &PCurveHandle,
    provider: &EdgeParameterProvider,
    curve: &CurveHandle,
    update_ends: bool,
) -> Result<(), Failure> {
    let geometry = provider.get_pcurve()?;
    let (start, end) = if update_ends {
        (0, curve.parameters_nb())
    } else {
        (1, curve.parameters_nb() - 1)
    };
    for index in start..end {
        let parameter = provider.parameter(index, curve.get_point(index)?)?;
        let point = geometry.d0(parameter)?;
        if update_ends {
            pcurve.add_point(point, parameter);
        } else {
            pcurve.insert_point(pcurve.parameters_nb() - 1, point, parameter)?;
        }
    }
    Ok(())
}
