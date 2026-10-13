//! Martensite copy isolation is proved by actual mutations, independently of flags.
#[path = "copy_isolation/fixture.rs"]
mod fixture;
#[path = "copy_isolation/mesh.rs"]
mod mesh;
use fixture::{copy_fixture, shared_copy_fixture};

#[test]
fn copied_mesh_storage_survives_source_and_reverse_mutations() {
    let source = shared_copy_fixture();
    let (_, copied) = copy_fixture(&source);
    mesh::assert_mesh_baseline(&source.edge, &source.faces[0]);
    let source_before = mesh::mesh_snapshot(&source.edge, &source.faces[0]);
    let copied_before = mesh::mesh_snapshot(&copied.edge, &copied.faces[0]);
    assert_eq!(source_before, copied_before);
    mesh::mutate_meshes(&source.edge, &source.faces[0], 9.0);
    assert_mesh_snapshot(&copied, &copied_before);
    let source_mutated = mesh::mesh_snapshot(&source.edge, &source.faces[0]);
    mesh::mutate_meshes(&copied.edge, &copied.faces[0], 17.0);
    assert_mesh_snapshot(&source, &source_mutated);
    assert_ne!(
        mesh::mesh_snapshot(&copied.edge, &copied.faces[0]),
        copied_before
    );
}

#[path = "copy_isolation/geometry.rs"]
mod geometry;

#[test]
fn analytic_representations_and_shared_topology_isolate_both_directions() {
    let source = shared_copy_fixture();
    geometry::assert_analytic_baseline(&source);
    let source_before = geometry::analytic_snapshot(&source);
    let (_, copied) = copy_fixture(&source);
    let copied_before = geometry::analytic_snapshot(&copied);
    assert_eq!(geometry::analytic_snapshot(&source), source_before);
    geometry::mutate_analytic_storage(&source, 9.0);
    assert_eq!(geometry::analytic_snapshot(&copied), copied_before);
    let source_mutated = geometry::analytic_snapshot(&source);
    geometry::mutate_analytic_storage(&copied, 17.0);
    assert_eq!(geometry::analytic_snapshot(&source), source_mutated);
    assert_eq!(source_mutated.curve_origin.x(), 9.0);
    assert_eq!(geometry::analytic_snapshot(&copied).pcurve_origin.x(), 17.0);
}

#[path = "copy_isolation/topology.rs"]
mod topology;

fn assert_mesh_snapshot(fixture: &fixture::SharedCopyFixture, expected: &mesh::MeshSnapshot) {
    let current = mesh::mesh_snapshot(&fixture.edge, &fixture.faces[0]);
    assert_eq!(&current, expected);
}

#[path = "copy_isolation/representation_failure.rs"]
mod representation_failure;
