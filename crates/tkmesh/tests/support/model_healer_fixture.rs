//! Actual model-builder and edge-discretizer inputs for native healer comparisons.
use rustycad_tkernel::{message::ProgressRange, standard::Handle};
use rustycad_tkmath::gp::Point2;
use rustycad_tkmesh::{
    brep_mesh::{BRepMeshEdgeDiscret, BRepMeshModelBuilder},
    imesh_data::{MeshModel, Status},
    imesh_tools::{MeshParameters, ModelAlgo, ModelBuilder},
};
#[path = "healer_vertex_fixture.rs"]
mod vertex_fixture;
pub fn healer_model(kind: i32, mode: i32, parameters: &MeshParameters) -> Handle<dyn MeshModel> {
    if kind >= 10 {
        return healer_polyline(kind - 10, mode, parameters);
    }
    let model =
        BRepMeshModelBuilder::new().perform(&super::model_fixture::model_shape(kind), parameters);
    BRepMeshEdgeDiscret::new().perform(&model, parameters, &ProgressRange::default());
    if !model.is_null() {
        alter_model(&model, mode);
    }
    model
}
pub fn alter_model(model: &Handle<dyn MeshModel>, mode: i32) {
    for index in 0..model.edges_nb() {
        alter_edge(&model.get_edge(index).unwrap(), index, mode);
    }
    if mode == 5 && model.faces_nb() > 0 {
        model
            .get_face(0)
            .unwrap()
            .status()
            .set_status(Status::Failure);
    }
}
fn alter_edge(edge: &rustycad_tkmesh::imesh_data::EdgeHandle, index: i32, mode: i32) {
    if mode >= 6 {
        vertex_fixture::separate_vertices(&mut edge.get_edge().unwrap(), index, mode);
    }
    if mode == 4 {
        edge.status().set_status(Status::Outdated);
    }
    for curve_index in 0..edge.pcurves_nb() {
        let curve = edge.get_pcurve(curve_index).unwrap();
        if mode == 3 {
            curve.clear(false);
            continue;
        }
        alter_curve(&curve, index, mode);
    }
}
fn alter_curve(curve: &rustycad_tkmesh::imesh_data::PCurveHandle, index: i32, mode: i32) {
    let count = curve.parameters_nb();
    if count == 0 {
        return;
    }
    if mode == 1 {
        let point = curve.get_point(0).unwrap();
        curve
            .set_point(
                0,
                Point2::new(point.x() + (index + 1) as f64 * 0.01, point.y() - 0.02),
            )
            .unwrap();
    }
    if mode == 2 {
        reverse_curve(curve, count);
    }
}

fn reverse_curve(curve: &rustycad_tkmesh::imesh_data::PCurveHandle, count: i32) {
    let points: Vec<Point2> = (0..count).map(|i| curve.get_point(i).unwrap()).collect();
    for i in 0..count {
        curve
            .set_point(i, points[(count - i - 1) as usize])
            .unwrap();
    }
}

fn healer_polyline(kind: i32, mode: i32, parameters: &MeshParameters) -> Handle<dyn MeshModel> {
    let (model, _face) = super::checker_fixture::checker_model(&[kind, mode % 2, 0, 0, 0, 0, 0]);
    for index in 0..model.edges_nb() {
        seed_polyline_edge(&model.get_edge(index).unwrap(), parameters);
    }
    model
}
fn seed_polyline_edge(edge: &rustycad_tkmesh::imesh_data::EdgeHandle, parameters: &MeshParameters) {
    edge.set_deflection(0.01);
    edge.set_angular_deflection(parameters.angle);
    edge.set_same_param(true);
    edge.set_same_range(true);
    let (first, last) = rustycad_tkbrep::topexp::vertices(&edge.get_edge().unwrap(), false);
    let curve = edge.get_curve();
    curve.add_point(rustycad_tkbrep::brep::BRepTool::pnt(&first).unwrap(), 0.0);
    curve.add_point(rustycad_tkbrep::brep::BRepTool::pnt(&last).unwrap(), 2.0);
}
