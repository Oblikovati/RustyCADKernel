//! Persistent parameter-provider comparison on production faces and edges.
use super::super::{flag, orientation, point2, real};
use super::model_builder::fixtures;
use rustycad_tkbrep::{brep_adaptor::BRepAdaptorCurve, topods};
use rustycad_tkernel::{ncollection::HArray1OfReal, standard::Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmesh::{
    brep_mesh::{BRepMeshModelBuilder, EdgeParameterProvider},
    imesh_data::{EdgeHandle, FaceHandle, ParametersListArrayAdaptor},
    imesh_tools::{MeshParameters, ModelBuilder},
};
pub fn edge_parameter_case(v: &[i32]) {
    let model = BRepMeshModelBuilder::new()
        .perform(&fixtures::model_shape(v[0]), &MeshParameters::default());
    let edge = model.get_edge(v[1] % model.edges_nb()).unwrap();
    let face = edge.get_pcurve(0).unwrap().get_face().upgrade();
    let oriented = topods::edge(&edge.get_shape().oriented(orientation(v[2]))).unwrap();
    let adaptor =
        BRepAdaptorCurve::with_edge_on_face(&oriented, &face.get_face().unwrap()).unwrap();
    let (first, last) = (
        adaptor.first_parameter().unwrap(),
        adaptor.last_parameter().unwrap(),
    );
    let provider = EdgeParameterProvider::default();
    for same in [v[5] != 0, v[5] == 0, v[5] != 0] {
        edge.set_same_param(same);
        initialize_provider(&provider, &edge, &face, v, (first, last));
        report_provider(&provider, &adaptor, v, (first, last));
    }
    flag(Handle::new(provider).is_instance_name("Standard_Transient"));
    println!();
}
fn initialize_provider(
    provider: &EdgeParameterProvider,
    edge: &EdgeHandle,
    face: &FaceHandle,
    v: &[i32],
    (first, last): (f64, f64),
) {
    let values = provider_parameters(edge, v[3], (first, last));
    if v[4] == 0 {
        provider
            .init(edge, orientation(v[2]), face, values)
            .unwrap();
    } else {
        provider
            .init(
                edge,
                orientation(v[2]),
                face,
                Handle::new(ParametersListArrayAdaptor::new(edge.get_curve().cast())),
            )
            .unwrap();
    }
}
fn provider_parameters(
    edge: &EdgeHandle,
    mode: i32,
    (first, last): (f64, f64),
) -> Handle<HArray1OfReal> {
    let (start, end) = match mode {
        0 => (first, last),
        1 => (10.0, 20.0),
        2 => (10.0, 10.0),
        _ => (20.0, 10.0),
    };
    let values = Handle::new(HArray1OfReal::with_bounds(1, 5).unwrap());
    edge.get_curve().clear(false);
    for index in 0..5 {
        let parameter = start + (end - start) * f64::from(index) / 4.0;
        values.set_value(index + 1, parameter).unwrap();
        edge.get_curve().add_point(Default::default(), parameter);
    }
    values
}
fn report_provider(
    provider: &EdgeParameterProvider,
    adaptor: &BRepAdaptorCurve,
    v: &[i32],
    (first, last): (f64, f64),
) {
    let order: &[i32] = match v[6] {
        0 => &[0, 1, 2, 3, 4],
        1 => &[4, 3, 2, 1, 0],
        _ => &[0, 4, 1, 3, 2, 4, 0],
    };
    let pcurve = provider.get_pcurve().unwrap();
    for &index in order {
        let parameter = first + (last - first) * f64::from(index) / 4.0;
        let point = adaptor.value(parameter).unwrap();
        let result = provider
            .parameter(index + i32::from(v[4] == 0), point)
            .unwrap();
        real(result);
        point2(pcurve.d0(result).unwrap());
    }
}
