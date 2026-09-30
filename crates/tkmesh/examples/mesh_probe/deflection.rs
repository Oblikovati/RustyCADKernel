//! Native deflection overload comparisons on actual topology.
use super::super::{flag, real};
use super::model_builder::fixtures;
use rustycad_tkmesh::{
    brep_mesh::{BRepMeshModelBuilder, Deflection},
    imesh_data::MeshModel,
    imesh_tools::{MeshParameters, ModelBuilder},
};
pub fn deflection_case(v: &[i32]) {
    let shape = fixtures::model_shape(v[0]);
    let parameters = deflection_parameters(v);
    real(
        Deflection::compute_absolute(
            &shape,
            parameters.deflection,
            [-1.0, 0.1, 100.0][v[5] as usize],
        )
        .unwrap(),
    );
    let model = BRepMeshModelBuilder::new().perform(&shape, &parameters);
    if model.is_null() {
        println!();
        return;
    }
    report_edge_deflections(&model, &parameters);
    report_face_deflections(&model, &parameters);
    println!();
}
fn deflection_parameters(v: &[i32]) -> MeshParameters {
    MeshParameters {
        relative: v[1] != 0,
        force_face_deflection: v[2] != 0,
        deflection: [-1.0, 0.001, 2.0][v[3] as usize],
        deflection_interior: [-1.0, 0.01, 4.0][v[4] as usize],
        ..Default::default()
    }
}
fn report_edge_deflections(
    model: &rustycad_tkernel::standard::Handle<dyn MeshModel>,
    parameters: &MeshParameters,
) {
    for index in 0..model.edges_nb() {
        let edge = model.get_edge(index).unwrap();
        Deflection::compute_edge(&edge, model.get_max_size(), parameters).unwrap();
        real(edge.get_deflection());
        real(edge.get_angular_deflection());
    }
}
fn report_face_deflections(
    model: &rustycad_tkernel::standard::Handle<dyn MeshModel>,
    parameters: &MeshParameters,
) {
    for index in 0..model.faces_nb() {
        let face = model.get_face(index).unwrap();
        for index in 0..face.wires_nb() {
            let wire = face.get_wire(index).unwrap();
            Deflection::compute_wire(&wire, parameters).unwrap();
            real(wire.get_deflection());
        }
        Deflection::compute_face(&face, parameters).unwrap();
        real(face.get_deflection());
    }
}
pub fn consistent_case(v: &[i32]) {
    let values = [
        -1.0,
        -0.0,
        0.001,
        0.9,
        1.0,
        1.1,
        2.0,
        f64::INFINITY,
        f64::NAN,
    ];
    flag(Deflection::is_consistent(
        values[v[0] as usize],
        values[v[1] as usize],
        v[2] != 0,
        [-0.1, 0.0, 0.1, 1.0][v[3] as usize],
    ));
    println!();
}
