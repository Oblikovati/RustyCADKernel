//! Native parameter scaling, ordering guard, shared collection and RTTI regressions.
use rustycad_tkernel::{
    ncollection::HArray1OfReal,
    standard::{Handle, Transient},
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmesh::{
    brep_mesh::EdgeParameterProvider,
    brep_mesh_data::DiscreteModel,
    imesh_data::{EdgeHandle, FaceHandle, MeshModel, ParametersListArrayAdaptor},
};
#[path = "../../tkshhealing/tests/support/wire_foundation.rs"]
mod wire_fixture;
fn provider_inputs(same: bool) -> (EdgeHandle, FaceHandle, Handle<HArray1OfReal>) {
    let (_, face, ordered) = wire_fixture::order_fixture(0, 0, 0);
    let model = DiscreteModel::new(&Default::default());
    let edge = model.add_edge(&ordered.edge(1).unwrap()).unwrap();
    edge.set_same_param(same);
    let face = model.add_face(&face).unwrap();
    let parameters = Handle::new(HArray1OfReal::with_bounds(1, 3).unwrap());
    for index in 1..=3 {
        parameters
            .set_value(index, 5.0 + f64::from(index) * 5.0)
            .unwrap();
    }
    (edge, face, parameters)
}
#[test]
fn empty_provider_and_native_transient_identity() {
    let provider = EdgeParameterProvider::default();
    assert!(provider.is_instance_name("Standard_Transient"));
    assert!(provider.parameter(0, Point3::default()).is_err());
    assert!(provider.get_pcurve().is_err());
}
#[test]
fn same_parameter_reads_shared_collection_without_projection() {
    let (edge, face, parameters) = provider_inputs(true);
    let provider =
        EdgeParameterProvider::with_edge(&edge, Orientation::Forward, &face, parameters.clone())
            .unwrap();
    assert_eq!(provider.parameter(2, Point3::default()).unwrap(), 15.0);
    parameters.set_value(2, 37.0).unwrap();
    assert_eq!(provider.parameter(2, Point3::default()).unwrap(), 37.0);
    assert!(!provider.get_pcurve().unwrap().is_null());
}
#[test]
fn scaled_projection_rejects_backward_crossing() {
    let (edge, face, parameters) = provider_inputs(false);
    let provider =
        EdgeParameterProvider::with_edge(&edge, Orientation::Forward, &face, parameters).unwrap();
    assert_eq!(
        provider.parameter(1, Point3::new(1.0, 0.0, 0.0)).unwrap(),
        0.0
    );
    assert_eq!(
        provider.parameter(2, Point3::new(1.5, 0.0, 0.0)).unwrap(),
        1.5
    );
    assert_eq!(
        provider.parameter(3, Point3::new(0.5, 0.0, 0.0)).unwrap(),
        2.5
    );
}
#[test]
fn zero_based_curve_array_and_reinitialization() {
    let (edge, face, _) = provider_inputs(true);
    edge.get_curve().add_point(Point3::default(), 4.0);
    edge.get_curve().add_point(Point3::default(), 9.0);
    let array = Handle::new(ParametersListArrayAdaptor::new(edge.get_curve().cast()));
    let provider = EdgeParameterProvider::new();
    provider
        .init(&edge, Orientation::Reversed, &face, array)
        .unwrap();
    assert_eq!(provider.parameter(0, Point3::default()).unwrap(), 4.0);
    edge.get_curve().set_parameter(1, 11.0).unwrap();
    assert_eq!(provider.parameter(1, Point3::default()).unwrap(), 11.0);
}
