//! Exact Rust port of the selected BRepMesh incremental-mesh legacy declaration.
use rustycad_tkbrep::topods::{Builder, Compound, Shape, Wire};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkmesh::{brep_mesh::BRepMeshIncrementalMesh, imesh_tools::MeshParameters};

#[test]
#[allow(non_snake_case)]
fn BRepMesh_IncrementalMeshTest__OCC31125_EmptyCompoundDoesNotCrash() {
    let mut empty_compound = Compound::default();
    Builder.make_compound(&mut empty_compound);
    let parameters = MeshParameters {
        deflection: 1.0,
        ..Default::default()
    };
    let result = BRepMeshIncrementalMesh::with_parameters(
        &empty_compound,
        &parameters,
        &ProgressRange::default(),
    );
    assert!(result.is_ok(), "empty compound meshing should not throw");
}

#[test]
fn invalid_linear_deflection_returns_native_numeric_error() {
    let mut empty_compound = Compound::default();
    Builder.make_compound(&mut empty_compound);
    let parameters = MeshParameters {
        deflection: 0.0,
        ..Default::default()
    };
    let result = BRepMeshIncrementalMesh::with_parameters(
        &empty_compound,
        &parameters,
        &ProgressRange::default(),
    );
    let failure = match result {
        Ok(_) => panic!("zero deflection must raise Standard_NumericError"),
        Err(failure) => failure,
    };
    assert_eq!(failure.exception_type(), "Standard_NumericError");
}

#[test]
fn nonempty_shapes_report_the_unported_meshing_stage() {
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    let parameters = MeshParameters::default();
    let result = BRepMeshIncrementalMesh::with_parameters(
        &Shape::from(wire),
        &parameters,
        &ProgressRange::default(),
    );
    let failure = match result {
        Ok(_) => panic!("wire meshing requires the unported face discretizer"),
        Err(failure) => failure,
    };
    assert_eq!(failure.exception_type(), "Standard_NotImplemented");
}
