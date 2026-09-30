use super::super::{flag, int, real};
use super::face_checker::fixture as checker_fixture;
use super::model_builder::fixtures as model_fixture;
use rustycad_tkernel::{message::ProgressRange, standard::Handle};
use rustycad_tkmesh::{
    brep_mesh::BRepMeshModelHealer,
    imesh_data::MeshModel,
    imesh_tools::{MeshParameters, ModelAlgo},
};
#[path = "../../tests/support/model_healer_fixture.rs"]
mod fixture;
pub fn healer_case(v: &[i32]) {
    let parameters = MeshParameters {
        in_parallel: v[2] != 0,
        relative: v[3] != 0,
        min_size: if v[4] == 0 { 0.001 } else { 10.0 },
        deflection: 0.1,
        ..Default::default()
    };
    let model = fixture::healer_model(v[0], v[1], &parameters);
    let healer = Handle::new(BRepMeshModelHealer::new());
    flag(healer.is_instance_name("BRepMesh_ModelHealer"));
    flag(healer.is_kind_name("IMeshTools_ModelAlgo"));
    for _ in 0..2 {
        flag(healer.perform(&model, &parameters, &ProgressRange::default()));
        if !model.is_null() {
            report_model(&model);
        }
    }
    println!();
}
fn report_model(model: &Handle<dyn MeshModel>) {
    super::edge_discret::report_edge_discret(model);
    int(model.faces_nb());
    for index in 0..model.faces_nb() {
        let face = model.get_face(index).unwrap();
        int(face.status().get_status_mask());
        real(face.get_deflection());
        int(face.wires_nb());
        for index in 0..face.wires_nb() {
            let wire = face.get_wire(index).unwrap();
            int(wire.status().get_status_mask());
            real(wire.get_deflection());
        }
    }
}
