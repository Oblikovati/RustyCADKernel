use super::super::{flag, int, real};
use super::post_processor::fixtures as post_fixture;
use rustycad_tkernel::{message::ProgressRange, standard::Handle};
use rustycad_tkmesh::{
    brep_mesh::BRepMeshModelPreProcessor,
    imesh_data::MeshModel,
    imesh_tools::{MeshParameters, ModelAlgo},
};
#[path = "../../tests/support/pre_processor_fixture.rs"]
mod fixture;
pub fn pre_case(v: &[i32]) {
    let model = fixture::existing_model(v);
    let parameters = MeshParameters {
        in_parallel: v[7] != 0,
        allow_quality_decrease: v[6] != 0,
        ..Default::default()
    };
    let stage = BRepMeshModelPreProcessor::new();
    flag(stage.perform(&model, &parameters, &ProgressRange::default()));
    if !model.is_null() {
        report_faces(&model);
        super::post_processor::report_post_model(&model);
    }
    println!();
}
pub fn cone_case(v: &[i32]) {
    let parameters = MeshParameters {
        deflection: [0.01, 0.1, 1.0][v[1] as usize],
        angle: [0.3, 0.7][v[2] as usize],
        min_size: 0.001,
        in_parallel: v[3] != 0,
        ..Default::default()
    };
    let model = fixture::cone_model(v[0], &parameters);
    fixture::alter_cone(&model, v[4]);
    flag(BRepMeshModelPreProcessor::new().perform(&model, &parameters, &ProgressRange::default()));
    report_faces(&model);
    super::edge_discret::report_edge_discret(&model);
    println!();
}
fn report_faces(model: &Handle<dyn MeshModel>) {
    int(model.faces_nb());
    for index in 0..model.faces_nb() {
        let face = model.get_face(index).unwrap();
        int(face.status().get_status_mask());
        real(face.get_deflection());
    }
}
