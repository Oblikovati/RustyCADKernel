//! Native postprocessor publication oracle twin.
use super::super::{flag, int, point3, real};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkernel::{message::ProgressRange, standard::Handle};
use rustycad_tkmath::poly::mesh_purpose;
use rustycad_tkmesh::{
    brep_mesh::BRepMeshModelPostProcessor,
    imesh_data::MeshModel,
    imesh_tools::{MeshParameters, ModelAlgo},
};
#[path = "../../tests/support/post_processor_fixture.rs"]
mod fixtures;
pub fn post_processor_case(v: &[i32]) {
    let model = fixtures::post_model(v);
    let parameters = MeshParameters {
        in_parallel: v[6] != 0,
        deflection: 0.25,
        angle: 0.75,
        min_size: 0.125,
        ..Default::default()
    };
    let processor = BRepMeshModelPostProcessor::new();
    flag(processor.perform(&model, &parameters, &ProgressRange::default()));
    if !model.is_null() {
        report_post_model(&model);
    }
    println!();
}
fn report_post_model(model: &Handle<dyn MeshModel>) {
    let edge = model.get_edge(0).unwrap().get_edge().unwrap();
    let (polygon, _) = BRepTool::polygon3d(&edge).unwrap();
    flag(!polygon.is_null());
    if !polygon.is_null() {
        real(polygon.deflection());
        int(polygon.nb_nodes());
        for index in 1..=polygon.nb_nodes() {
            point3(polygon.nodes()[index]);
            real(polygon.parameters().unwrap()[index]);
        }
    }
    for index in 0..model.faces_nb() {
        report_post_face(model, index);
    }
}
fn report_post_face(model: &Handle<dyn MeshModel>, index: i32) {
    let face = model.get_face(index).unwrap().get_face().unwrap();
    let (mesh, location) = BRepTool::triangulation(&face, mesh_purpose::NONE).unwrap();
    flag(!mesh.is_null());
    if mesh.is_null() {
        return;
    }
    real(mesh.deflection());
    let parameters = mesh.parameters();
    flag(!parameters.is_null());
    if !parameters.is_null() {
        real(parameters.deflection());
        real(parameters.angle());
        real(parameters.min_size());
    }
    report_post_polygons(model, &mesh, &location);
}
fn report_post_polygons(
    model: &Handle<dyn MeshModel>,
    mesh: &Handle<rustycad_tkmath::poly::Triangulation>,
    location: &rustycad_tkmath::toploc::Location,
) {
    for orientation in [
        rustycad_tkg3d::top_abs::Orientation::Forward,
        rustycad_tkg3d::top_abs::Orientation::Reversed,
    ] {
        let edge = rustycad_tkbrep::topods::edge(
            &model.get_edge(0).unwrap().get_shape().oriented(orientation),
        )
        .unwrap();
        let polygon = BRepTool::polygon_on_triangulation(&edge, mesh, location).unwrap();
        flag(!polygon.is_null());
        if polygon.is_null() {
            continue;
        }
        report_post_polygon(&polygon);
    }
}

fn report_post_polygon(polygon: &Handle<rustycad_tkmath::poly::PolygonOnTriangulation>) {
    real(polygon.deflection());
    int(polygon.nb_nodes());
    for index in 1..=polygon.nb_nodes() {
        int(polygon.node(index).unwrap());
        real(polygon.parameter(index).unwrap());
    }
}
