//! Existing topological interior-vertex insertion for constrained face meshing.
use crate::brep_mesh::{
    BRepMeshDataStructureOfDelaun, BRepMeshVertex, DefaultRangeSplitter, DegreeOfFreedom,
};
use crate::imesh_data::FaceHandle;
use rustycad_tkbrep::{brep::BRepTool, topexp::Explorer, topods};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::{Point2, Xy};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

pub(super) fn configure_vertex_tool(
    splitter: &DefaultRangeSplitter,
    mesh: &Handle<BRepMeshDataStructureOfDelaun>,
) -> Result<(), Failure> {
    let delta = splitter.get_delta();
    let tolerance = splitter.get_tolerance_uv();
    mesh.vertex_tool()
        .set_tolerances(tolerance.0 / delta.0, tolerance.1 / delta.1);
    mesh.vertex_tool()
        .set_cell_sizes(14.0 * tolerance.0 / delta.0, 14.0 * tolerance.1 / delta.1)?;
    Ok(())
}

pub(super) fn register_internal_vertices(
    face: &FaceHandle,
    splitter: &DefaultRangeSplitter,
    mesh: &Handle<BRepMeshDataStructureOfDelaun>,
    points: &mut Vec<rustycad_tkmath::gp::Point3>,
) -> Result<(), Failure> {
    let topological_face = face.get_face()?;
    let classifier = BRepTopAdaptorFClass2d::new(&topological_face, 1e-9)?;
    for shape in Explorer::new(&topological_face, ShapeEnum::Vertex, ShapeEnum::Edge)? {
        if shape.orientation() != Orientation::Internal {
            continue;
        }
        register_vertex(
            &shape,
            &topological_face,
            &classifier,
            splitter,
            mesh,
            points,
        )?;
    }
    Ok(())
}

fn register_vertex(
    shape: &rustycad_tkbrep::topods::Shape,
    face: &rustycad_tkbrep::topods::Face,
    classifier: &BRepTopAdaptorFClass2d,
    splitter: &DefaultRangeSplitter,
    mesh: &Handle<BRepMeshDataStructureOfDelaun>,
    points: &mut Vec<rustycad_tkmath::gp::Point3>,
) -> Result<(), Failure> {
    let vertex = topods::vertex(shape)?;
    let Some(uv) = internal_vertex_uv(&vertex, face, classifier)? else {
        return Ok(());
    };
    register_internal_node(uv, BRepTool::pnt(&vertex)?, splitter, mesh, points)
}

fn internal_vertex_uv(
    vertex: &rustycad_tkbrep::topods::Vertex,
    face: &rustycad_tkbrep::topods::Face,
    classifier: &BRepTopAdaptorFClass2d,
) -> Result<Option<Point2>, Failure> {
    let uv = BRepTool::parameters(vertex, face)?;
    Ok((classifier.perform(uv, true)? == State::In).then_some(uv))
}

pub(super) fn register_internal_node(
    uv: Point2,
    point: rustycad_tkmath::gp::Point3,
    splitter: &DefaultRangeSplitter,
    mesh: &Handle<BRepMeshDataStructureOfDelaun>,
    points: &mut Vec<rustycad_tkmath::gp::Point3>,
) -> Result<(), Failure> {
    let scaled = splitter.scale(uv, true);
    let location = points.len() as i32 + 1;
    let node = BRepMeshVertex::new(
        Xy::new(scaled.x(), scaled.y()),
        location,
        DegreeOfFreedom::Fixed,
    );
    if mesh.add_node(&node, false)? == location {
        points.push(point);
    }
    Ok(())
}
