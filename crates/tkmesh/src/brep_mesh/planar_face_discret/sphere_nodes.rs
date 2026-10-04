// Native algorithm: BRepMesh_SphereRangeSplitter.cxx at pinned OCCT 3d097a03.
use super::*;
use crate::brep_mesh::DegreeOfFreedom;
use crate::imesh_data::FaceHandle;
use crate::imesh_tools::MeshParameters;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkgeombase::gcpnts::TangentialDeflection;
use rustycad_tkmath::{geom_abs::SurfaceType, gp::Point2};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

pub(super) fn register_interior(
    face: &FaceHandle,
    parameters: &MeshParameters,
    splitter: &DefaultRangeSplitter,
    mesh: &Handle<BRepMeshDataStructureOfDelaun>,
    points: &mut Vec<rustycad_tkmath::gp::Point3>,
) -> Result<Vec<i32>, Failure> {
    if face.get_surface().get_type()? != SurfaceType::Sphere {
        return Ok(Vec::new());
    }
    let coords = generate_nodes(face, parameters, splitter)?;
    insert_nodes(face, splitter, mesh, points, &coords)
}

fn generate_nodes(
    face: &FaceHandle,
    parameters: &MeshParameters,
    splitter: &DefaultRangeSplitter,
) -> Result<Vec<Point2>, Failure> {
    let radius = face.get_surface().sphere()?.radius();
    let step = 0.7
        * TangentialDeflection::arc_angular_step(
            radius,
            face.get_deflection(),
            parameters.angle,
            parameters.min_size,
        )?;
    Ok(generate_grid(
        splitter.get_range_v(),
        splitter.get_range_u(),
        step,
    ))
}

fn generate_grid(range_v: (f64, f64), range_u: (f64, f64), step: f64) -> Vec<Point2> {
    let v_step = compute_step(range_v, step);
    let u_step = compute_step(range_u, step);
    let mut nodes = Vec::new();
    let mut shifted = false;
    let mut v = range_v.0 + v_step.0;
    while v < v_step.1 {
        shifted = !shifted;
        let offset = if shifted { u_step.0 * 0.5 } else { 0.0 };
        append_row(&mut nodes, range_u, v, u_step, offset);
        v += v_step.0;
    }
    nodes
}

fn compute_step(range: (f64, f64), default_step: f64) -> (f64, f64) {
    let difference = range.1 - range.0;
    let count = (difference / default_step).trunc() + 1.0;
    (
        difference / count,
        range.1 - rustycad_tkernel::precision::p_confusion(),
    )
}

fn append_row(nodes: &mut Vec<Point2>, range_u: (f64, f64), v: f64, step: (f64, f64), offset: f64) {
    let mut u = range_u.0 + offset;
    while u < step.1 {
        nodes.push(Point2::new(u, v));
        u += step.0;
    }
}

fn insert_nodes(
    face: &FaceHandle,
    splitter: &DefaultRangeSplitter,
    mesh: &Handle<BRepMeshDataStructureOfDelaun>,
    points: &mut Vec<rustycad_tkmath::gp::Point3>,
    nodes: &[Point2],
) -> Result<Vec<i32>, Failure> {
    let classifier = BRepTopAdaptorFClass2d::new(&face.get_face()?, 1e-9)?;
    let mut inserted = Vec::new();
    for uv in nodes {
        if let Some(index) = insert_node(*uv, &classifier, splitter, mesh, points)? {
            inserted.push(index);
        }
    }
    Ok(inserted)
}

fn insert_node(
    uv: Point2,
    classifier: &BRepTopAdaptorFClass2d,
    splitter: &DefaultRangeSplitter,
    mesh: &Handle<BRepMeshDataStructureOfDelaun>,
    points: &mut Vec<rustycad_tkmath::gp::Point3>,
) -> Result<Option<i32>, Failure> {
    if classifier.perform(uv, true)? != State::In {
        return Ok(None);
    }
    add_free_node(uv, splitter, mesh, points)
}

fn add_free_node(
    uv: Point2,
    splitter: &DefaultRangeSplitter,
    mesh: &Handle<BRepMeshDataStructureOfDelaun>,
    points: &mut Vec<rustycad_tkmath::gp::Point3>,
) -> Result<Option<i32>, Failure> {
    let location = points.len() as i32 + 1;
    let scaled = splitter.scale(uv, true);
    let vertex = BRepMeshVertex::new(
        Xy::new(scaled.x(), scaled.y()),
        location,
        DegreeOfFreedom::Free,
    );
    let index = mesh.add_node(&vertex, false)?;
    if index != location {
        return Ok(None);
    }
    points.push(splitter.point(uv)?);
    Ok(Some(index))
}
