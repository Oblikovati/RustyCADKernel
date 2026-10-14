//! A well-defined parameter-length mismatch fails inside actual mesh Copy().
use super::fixture::{SharedCopyFixture, copy_fixture, shared_copy_fixture};
use super::geometry::{AnalyticSnapshot, analytic_snapshot};
use super::mesh::{MeshSnapshot, mesh_snapshot};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::poly::Polygon3D;
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};

fn attach_mismatched_parameters(edge: &Edge) {
    let mut nodes = Array1::with_bounds(1, 2).unwrap();
    nodes.set_value(1, Point3::default()).unwrap();
    nodes.set_value(2, Point3::new(2.0, 0.0, 0.0)).unwrap();
    let mut parameters = Array1::with_bounds(1, 3).unwrap();
    parameters.init(0.0);
    let polygon = Handle::new(Polygon3D::from_nodes_and_parameters(&nodes, &parameters).unwrap());
    BRepBuilder::default()
        .update_edge_polygon3d(edge, &polygon, &Location::default())
        .unwrap();
}

fn fail_on_mesh_parameters(copier: &mut BRepBuilderAPICopy) -> bool {
    let failing = shared_copy_fixture();
    attach_mismatched_parameters(&failing.edge);
    let before = analytic_snapshot(&failing);
    let failure = copier.perform(&failing.root, true, true).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_DimensionMismatch");
    assert_eq!(analytic_snapshot(&failing), before);
    assert!(!copier.is_done());
    copier.is_done()
}

fn assert_published_copy(
    fixture: &SharedCopyFixture,
    analytic: &AnalyticSnapshot,
    mesh: &MeshSnapshot,
) {
    assert_eq!(&analytic_snapshot(fixture), analytic);
    assert_eq!(&mesh_snapshot(&fixture.edge, &fixture.faces[0]), mesh);
}

#[test]
fn previous_published_copy_survives_mesh_failure_and_builder_recovery() {
    let source = shared_copy_fixture();
    let (mut copier, published) = copy_fixture(&source);
    let before = analytic_snapshot(&published);
    let mesh_before = mesh_snapshot(&published.edge, &published.faces[0]);
    let failure_done = fail_on_mesh_parameters(&mut copier);
    assert_published_copy(&published, &before, &mesh_before);
    copier.perform(&source.root, true, true).unwrap();
    assert!(copier.is_done());
    let replay = failure_trace(failure_done, &published.root, &mut copier);
    assert_eq!(replay, include_str!("native_mesh_failure.expected"));
    assert_published_copy(&published, &before, &mesh_before);
}

fn failure_trace(failure_done: bool, published: &Shape, copier: &mut BRepBuilderAPICopy) -> String {
    format!(
        "mesh-failure {} {}\nrecover {} {}\n",
        failure_done as u8,
        (!published.is_null()) as u8,
        copier.is_done() as u8,
        published.is_same(copier.shape().unwrap()) as u8
    )
}
