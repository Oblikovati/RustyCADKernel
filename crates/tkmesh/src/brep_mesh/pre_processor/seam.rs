use super::*;
use crate::imesh_data::{EdgeHandle, PCurveHandle};
use rustycad_tkernel::precision;
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, top_abs::Orientation};
use rustycad_tkmath::{geom_abs::SurfaceType, gp};
pub(super) fn amplify_face_seam(
    face: &FaceHandle,
    parameters: &MeshParameters,
) -> Result<(), Failure> {
    if face.get_surface().get_type()? != SurfaceType::Cone || face.status().is_set(Status::Failure)
    {
        return Ok(());
    }
    let wire = face.get_wire(0)?;
    for index in 0..wire.edges_nb() - 1 {
        let edge = wire.get_edge(index)?.upgrade();
        if edge.get_pcurve_for(&face.downgrade(), Orientation::Forward)?
            == edge.get_pcurve_for(&face.downgrade(), Orientation::Reversed)?
        {
            continue;
        }
        amplify_edge(&edge, face, parameters)?;
        return Ok(());
    }
    Ok(())
}
fn amplify_edge(
    edge: &EdgeHandle,
    face: &FaceHandle,
    parameters: &MeshParameters,
) -> Result<(), Failure> {
    if edge.get_curve().parameters_nb() != 2 {
        return Ok(());
    }
    if !split_edge(edge, face, cone_step(face, parameters)?.abs())? {
        return Ok(());
    }
    let (mesh, _) = BRepTool::triangulation(&face.get_face()?, mesh_purpose::NONE)?;
    if !mesh.is_null() {
        face.status().set_status(Status::Outdated);
    }
    Ok(())
}
fn cone_step(face: &FaceHandle, parameters: &MeshParameters) -> Result<f64, Failure> {
    let mut splitter = ConeRangeSplitter::default();
    splitter.reset(face, parameters);
    let wire = face.get_wire(0)?;
    for edge_index in 0..wire.edges_nb() {
        let edge = wire.get_edge(edge_index)?.upgrade();
        let pcurve =
            edge.get_pcurve_for(&face.downgrade(), wire.get_edge_orientation(edge_index)?)?;
        for point_index in 0..pcurve.parameters_nb() {
            splitter.add_point(pcurve.get_point(point_index)?);
        }
    }
    Ok(splitter.get_split_steps(parameters, &mut (0, 0))?.1)
}
fn split_edge(edge: &EdgeHandle, face: &FaceHandle, step: f64) -> Result<bool, Failure> {
    let topology = edge.get_edge()?;
    let topology_face = face.get_face()?;
    let (curve, first, last) = BRepTool::curve(&topology)?;
    let pcurves = (edge.get_pcurve(0)?, edge.get_pcurve(1)?);
    let start = pcurves.0.get_point(0)?;
    let end = pcurves.0.get_point(pcurves.0.parameters_nb() - 1)?;
    let modulus = (start.y() - end.y()).abs();
    if modulus < gp::resolution() {
        return Ok(false);
    }
    let delta = (last - first).abs() / modulus * step;
    if !split_curve_3d(&edge.get_curve(), &curve, delta)? {
        return Ok(false);
    }
    split_pcurves(topology, &topology_face, &pcurves, end.x(), delta)
}
fn split_curve_3d(
    discrete: &crate::imesh_data::CurveHandle,
    curve: &Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
    delta: f64,
) -> Result<bool, Failure> {
    super::split_curve::split_parameters(
        discrete.get_parameter(0)?,
        discrete.get_parameter(discrete.parameters_nb() - 1)?,
        delta,
        |parameter| {
            discrete.insert_point(
                discrete.parameters_nb() - 1,
                curve.value(parameter)?,
                parameter,
            )
        },
    )
}
fn split_pcurves(
    mut topology: rustycad_tkbrep::topods::Edge,
    face: &rustycad_tkbrep::topods::Face,
    pcurves: &(PCurveHandle, PCurveHandle),
    last_x: f64,
    delta: f64,
) -> Result<bool, Failure> {
    topology.set_orientation(Orientation::Forward);
    let mut first = BRepTool::curve_on_face(&topology, face)?.pcurve;
    topology.set_orientation(Orientation::Reversed);
    let mut second = BRepTool::curve_on_face(&topology, face)?.pcurve;
    if first.is_null() || second.is_null() {
        return Ok(false);
    }
    if (last_x - first.value(first.first_parameter())?.x()).abs() > precision::confusion() {
        std::mem::swap(&mut first, &mut second);
    }
    split_pcurve(&pcurves.0, &first, delta)?;
    split_pcurve(&pcurves.1, &second, delta)?;
    Ok(true)
}
fn split_pcurve(
    discrete: &PCurveHandle,
    curve: &Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
    delta: f64,
) -> Result<(), Failure> {
    super::split_curve::split_parameters(
        discrete.get_parameter(0)?,
        discrete.get_parameter(discrete.parameters_nb() - 1)?,
        delta,
        |parameter| {
            discrete.insert_point(
                discrete.parameters_nb() - 1,
                curve.value(parameter)?,
                parameter,
            )
        },
    )?;
    Ok(())
}
