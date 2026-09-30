//! Existing polygon extraction preserves indexing, parameter reuse and runtime ancestry.
use rustycad_tkernel::{message::ProgressRange, standard::Transient};
use rustycad_tkmath::gp::Point3;
use rustycad_tkmesh::{
    brep_mesh::{BRepMeshModelPostProcessor, EdgeTessellationExtractor},
    imesh_tools::{CurveTessellator, MeshParameters, ModelAlgo},
};
#[path = "support/post_processor_fixture.rs"]
mod fixtures;
#[path = "../../tkshhealing/tests/support/wire_foundation.rs"]
mod wire_fixture;
#[test]
fn stored_nodes_and_parameters_are_extracted() {
    let model = fixtures::post_model(&[1, 0, 0, 3, 1, 1, 0]);
    assert!(BRepMeshModelPostProcessor::new().perform(
        &model,
        &MeshParameters::default(),
        &ProgressRange::default()
    ));
    let extractor =
        EdgeTessellationExtractor::new(&model.get_edge(0).unwrap(), &model.get_face(0).unwrap())
            .unwrap();
    assert!(extractor.is_kind_name("IMeshTools_CurveTessellator"));
    assert_eq!(extractor.points_nb(), 2);
    let mut point = Point3::default();
    let mut parameter = -1.0;
    assert!(extractor.value(2, &mut point, &mut parameter).unwrap());
    assert_eq!(point, Point3::new(2.0, 0.0, 0.3));
    assert_eq!(parameter, 1.0);
    assert!(extractor.value(0, &mut point, &mut parameter).is_err());
    assert_eq!(point, Point3::new(2.0, 0.0, 0.3));
    assert_eq!(parameter, 1.0);
}
#[test]
fn absent_polygon_returns_native_null_failure() {
    let model = fixtures::post_model(&[1, 0, 0, 3, 1, 1, 0]);
    let result =
        EdgeTessellationExtractor::new(&model.get_edge(0).unwrap(), &model.get_face(0).unwrap());
    assert_eq!(
        result.err().unwrap().exception_type(),
        "Standard_NullObject"
    );
}
