//! Full native edge-stage status and point/parameter comparisons.
use super::super::{flag, int, point2, point3, real};
use super::{model_builder::fixtures, post_processor::fixtures as post_fixture};
use rustycad_tkernel::{message::ProgressRange, standard::Handle};
use rustycad_tkmesh::{
    brep_mesh::{BRepMeshEdgeDiscret, BRepMeshModelBuilder, BRepMeshModelPostProcessor},
    imesh_data::MeshModel,
    imesh_tools::{MeshParameters, ModelAlgo, ModelBuilder},
};
#[path = "../../tests/support/edge_discret_fixture.rs"]
mod existing;
pub fn edge_discret_case(v: &[i32]) {
    let parameters = MeshParameters {
        min_size: if v[1] == 0 { -1.0 } else { 0.001 },
        relative: v[2] != 0,
        in_parallel: v[3] != 0,
        allow_quality_decrease: v[4] != 0,
        deflection: 0.1,
        ..Default::default()
    };
    let model = edge_discret_model(v, &parameters);
    existing::seed_free_polygons(&model, v[5]);
    let algorithm = BRepMeshEdgeDiscret::new();
    flag(algorithm.perform(&model, &parameters, &ProgressRange::default()));
    if !model.is_null() {
        report_edge_discret(&model);
    }
    println!();
}
fn edge_discret_model(v: &[i32], parameters: &MeshParameters) -> Handle<dyn MeshModel> {
    if v[0] < 10 {
        return BRepMeshModelBuilder::new().perform(&fixtures::model_shape(v[0]), parameters);
    }
    let model = post_fixture::post_model(&[v[0] - 9, 0, 0, 3, 1, 1, 0]);
    BRepMeshModelPostProcessor::new().perform(&model, parameters, &ProgressRange::default());
    existing::clear_model_curves(&model);
    model
}
fn report_edge_discret(model: &Handle<dyn MeshModel>) {
    int(model.edges_nb());
    for index in 0..model.edges_nb() {
        let edge = model.get_edge(index).unwrap();
        int(edge.status().get_status_mask());
        real(edge.get_deflection());
        real(edge.get_angular_deflection());
        flag(edge.get_same_param());
        flag(edge.get_same_range());
        flag(edge.get_degenerated());
        let curve = edge.get_curve();
        int(curve.parameters_nb());
        for index in 0..curve.parameters_nb() {
            point3(curve.get_point(index).unwrap());
            real(curve.get_parameter(index).unwrap());
        }
        int(edge.pcurves_nb());
        for index in 0..edge.pcurves_nb() {
            report_discrete_pcurve(&edge.get_pcurve(index).unwrap());
        }
    }
}
fn report_discrete_pcurve(curve: &rustycad_tkmesh::imesh_data::PCurveHandle) {
    int(curve.parameters_nb());
    for index in 0..curve.parameters_nb() {
        point2(curve.get_point(index).unwrap());
        real(curve.get_parameter(index).unwrap());
        int(curve.get_index(index).unwrap());
    }
}

pub fn edge_tessellate_case(v: &[i32]) {
    let parameters = MeshParameters {
        min_size: 0.001,
        ..Default::default()
    };
    let model = BRepMeshModelBuilder::new().perform(&fixtures::model_shape(v[0]), &parameters);
    let edge = model.get_edge(v[1] % model.edges_nb()).unwrap();
    edge.set_deflection(0.1);
    edge.set_angular_deflection(0.5);
    edge.set_degenerated(v[3] != 0);
    edge.set_same_param(v[4] != 0);
    let pcurve = edge.get_pcurve(0).unwrap();
    let face = pcurve.get_face().upgrade();
    let tessellator = BRepMeshEdgeDiscret::create_face_tessellator(
        &edge,
        pcurve.get_orientation(),
        &face,
        &parameters,
        if v[5] == 0 { 2 } else { 5 },
    )
    .unwrap();
    if v[2] == 0 {
        seed_tessellation_ends(&edge, &tessellator);
    }
    BRepMeshEdgeDiscret::tessellate_3d(&edge, &tessellator, v[2] != 0).unwrap();
    BRepMeshEdgeDiscret::tessellate_2d(&edge, v[2] != 0).unwrap();
    report_edge_discret(&model);
    println!();
}
fn seed_tessellation_ends(
    edge: &rustycad_tkmesh::imesh_data::EdgeHandle,
    tessellator: &Handle<dyn rustycad_tkmesh::imesh_tools::CurveTessellator>,
) {
    let mut point = Default::default();
    let mut parameter = 0.0;
    for index in [1, tessellator.points_nb()] {
        tessellator
            .value(index, &mut point, &mut parameter)
            .unwrap();
        edge.get_curve().add_point(point, parameter);
    }
    BRepMeshEdgeDiscret::tessellate_2d(edge, true).unwrap();
}

#[path = "../../tests/support/mutating_tessellator.rs"]
mod callbacks;
pub fn edge_callback_case() {
    let model =
        BRepMeshModelBuilder::new().perform(&fixtures::model_shape(4), &MeshParameters::default());
    let edge = model.get_edge(0).unwrap();
    let (first, last) = rustycad_tkbrep::topexp::vertices(&edge.get_edge().unwrap(), false);
    let callback = Handle::new(callbacks::MutatingEndpoints {
        transient: Default::default(),
        first,
        last,
    })
    .cast();
    BRepMeshEdgeDiscret::tessellate_3d(&edge, &callback, true).unwrap();
    report_edge_discret(&model);
    println!();
}
