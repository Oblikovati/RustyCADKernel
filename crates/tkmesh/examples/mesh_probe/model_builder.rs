//! Concrete BRepMesh_ModelBuilder and ShapeVisitor oracle twin.
use super::super::{flag, int, orientation, real};
use rustycad_tkernel::{message::Status, standard::Handle};
use rustycad_tkmesh::{
    brep_mesh::BRepMeshModelBuilder,
    imesh_data::{FaceHandle, MeshModel},
    imesh_tools::{MeshParameters, ModelBuilder},
};
#[path = "../../tests/support/model_builder_fixture.rs"]
pub(super) mod fixtures;
pub fn model_builder_case(v: &[i32]) {
    let shape = fixtures::model_shape(v[0]).oriented(orientation(v[4]));
    let parameters = MeshParameters {
        relative: v[1] != 0,
        deflection: [0.001, -1., 1.][v[2] as usize],
        deflection_interior: [-1., 0.2, 4.][v[3] as usize],
        ..Default::default()
    };
    let builder = BRepMeshModelBuilder::new();
    let model = builder.perform(&shape, &parameters);
    for status in [Status::DONE1, Status::FAIL1, Status::FAIL2] {
        flag(builder.get_status().is_set(status));
    }
    flag(model.is_null());
    if model.is_null() {
        println!();
        return;
    }
    real(model.get_max_size());
    int(model.faces_nb());
    int(model.edges_nb());
    for index in 0..model.faces_nb() {
        report_model_face(&model, &model.get_face(index).unwrap());
    }
    report_model_edges(&model);
    println!();
}
fn report_model_face(model: &Handle<dyn MeshModel>, face: &FaceHandle) {
    int(face.status().get_status_mask());
    int(face.wires_nb());
    for index in 0..face.wires_nb() {
        let wire = face.get_wire(index).unwrap();
        int(wire.status().get_status_mask());
        int(wire.edges_nb());
        for entry in 0..wire.edges_nb() {
            let edge = wire.get_edge(entry).unwrap().upgrade();
            let index = (0..model.edges_nb())
                .find(|&i| model.get_edge(i).unwrap() == edge)
                .unwrap();
            int(index);
            int(wire.get_edge_orientation(entry).unwrap() as i32);
        }
    }
}
fn report_model_edges(model: &Handle<dyn MeshModel>) {
    for index in 0..model.edges_nb() {
        let edge = model.get_edge(index).unwrap();
        flag(edge.get_same_param());
        flag(edge.get_same_range());
        flag(edge.get_degenerated());
        int(edge.pcurves_nb());
        for index in 0..edge.pcurves_nb() {
            let pcurve = edge.get_pcurve(index).unwrap();
            let face = pcurve.get_face().upgrade();
            let index = (0..model.faces_nb())
                .find(|&i| model.get_face(i).unwrap() == face)
                .unwrap();
            int(index);
            int(pcurve.get_orientation() as i32);
        }
    }
}
