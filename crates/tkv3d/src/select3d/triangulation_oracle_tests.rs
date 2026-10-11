//! Actual selecting volumes replay native triangulation selection and aliasing state.
use super::triangulation_oracle_geometry::{oracle_edges, oracle_location, oracle_mesh};
use super::triangulation_oracle_snapshot::{
    compare_oracle_snapshot, failure_code, oracle_snapshot,
};
use super::{
    Select3DSensitiveEntity, Select3DSensitiveEntityMembers, Select3DSensitiveTriangulation,
};
use crate::select_mgr::SelectMgrEntityOwner;
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Point3;

fn oracle_probe(ordinal: i32, edges: &Handle<HArray1<i32>>) -> Select3DSensitiveTriangulation {
    let mesh = oracle_mesh(ordinal / 144);
    let location = oracle_location(ordinal / 6 % 3);
    let owner = SelectMgrEntityOwner::new(13).cast();
    let interior = ordinal / 72 % 2 != 0;
    if ordinal / 18 % 4 == 0 {
        return Select3DSensitiveTriangulation::new(&owner, &mesh, &location, interior).unwrap();
    }
    Select3DSensitiveTriangulation::with_free_edge_handle(
        &owner,
        &mesh,
        &location,
        edges,
        Point3::new(7., 8., 9.),
        interior,
    )
    .unwrap()
}
fn replay_triangulation_scenario(ordinal: i32, native: &mut std::str::Lines<'_>) {
    let edges = oracle_edges(ordinal / 18 % 4, 8);
    let original = oracle_probe(ordinal, &edges);
    assert_input_edge_alias(&original, &edges, ordinal / 18 % 4);
    original.set_sensitivity_factor(7).unwrap();
    let connected = connected_triangulation(&original);
    let connected = connected.get().unwrap();
    assert_shared_triangulation(&original, connected);
    replay_initial_detection(&original, connected, ordinal, native);
    mutate_shared_edges(&edges);
    dirty_connected_pair(&original, connected);
    compare_oracle_snapshot(
        &oracle_snapshot(&original, ordinal, 4),
        native.next().unwrap(),
    );
    compare_oracle_snapshot(
        &oracle_snapshot(connected, ordinal, 5),
        native.next().unwrap(),
    );
}
fn mutate_shared_edges(edges: &Handle<HArray1<i32>>) {
    let Some(edges) = edges.get() else {
        return;
    };
    let lower = edges.array1().lower();
    edges.change_array1().set_value(lower, 4).unwrap();
}
fn replay_unpaired_edge_controls(native: &mut std::str::Lines<'_>) {
    for control in 0..12 {
        let outcome = unpaired_edge_probe(control);
        let actual = [
            f64::from(864 + control),
            -1.,
            failure_code(outcome.as_ref().err()),
        ];
        compare_oracle_snapshot(&actual, native.next().unwrap());
    }
}
fn unpaired_edge_probe(
    control: i32,
) -> Result<Select3DSensitiveTriangulation, rustycad_tkernel::standard::Failure> {
    let kind = if control < 6 { 1 } else { 3 };
    let length = [0, 1, 3][control as usize % 3];
    Select3DSensitiveTriangulation::with_free_edge_handle(
        &SelectMgrEntityOwner::new(13).cast(),
        &oracle_mesh(kind),
        &oracle_location(0),
        &oracle_edges(1, length),
        Point3::new(7., 8., 9.),
        control / 3 % 2 != 0,
    )
}
fn assert_shared_triangulation(
    original: &Select3DSensitiveTriangulation,
    connected: &Select3DSensitiveTriangulation,
) {
    assert_eq!(original.triangulation(), connected.triangulation());
    assert_eq!(
        original.state.read().free_edges,
        connected.state.read().free_edges
    );
}
fn replay_initial_detection(
    original: &Select3DSensitiveTriangulation,
    connected: &Select3DSensitiveTriangulation,
    ordinal: i32,
    native: &mut std::str::Lines<'_>,
) {
    for step in 0..4 {
        let probe = if step == 3 { connected } else { original };
        compare_oracle_snapshot(
            &oracle_snapshot(probe, ordinal, step),
            native.next().unwrap(),
        );
    }
}
#[test]
fn native_triangulation_detection_locations_and_shared_edges() {
    let mut native = super::triangulation_oracle_fixtures::TRIANGULATION_NATIVE_SNAPSHOTS.lines();
    for ordinal in 0..864 {
        replay_triangulation_scenario(ordinal, &mut native);
    }
    replay_unpaired_edge_controls(&mut native);
    assert!(native.next().is_none());
}

fn connected_triangulation(
    original: &Select3DSensitiveTriangulation,
) -> Handle<Select3DSensitiveTriangulation> {
    original
        .get_connected()
        .unwrap()
        .cast::<Select3DSensitiveTriangulation>()
}

#[test]
fn copied_slice_convenience_retains_native_connected_edge_sharing() {
    let mut indices = [1, 2, 2, 3, 3, 4, 4, 1];
    let probe = copied_edge_probe(&indices);
    indices[0] = 4;
    let retained = probe.state.read().free_edges.clone();
    assert_eq!(*retained.get().unwrap().array1().value(1).unwrap(), 1);
    let connected = connected_triangulation(&probe);
    assert_shared_triangulation(&probe, connected.get().unwrap());
    retained
        .get()
        .unwrap()
        .change_array1()
        .set_value(1, indices[0])
        .unwrap();
    let shared = connected.get().unwrap().state.read().free_edges.clone();
    assert_eq!(*shared.get().unwrap().array1().value(1).unwrap(), 4);
}
#[test]
fn missing_boundary_handle_reports_expected_storage_shape() {
    let outcome = Select3DSensitiveTriangulation::with_free_edge_handle(
        &SelectMgrEntityOwner::new(13).cast(),
        &oracle_mesh(1),
        &oracle_location(0),
        &Handle::default(),
        Point3::default(),
        false,
    );
    let failure = outcome.err().unwrap();
    assert_eq!(failure.exception_type(), "Standard_NullObject");
    assert!(failure.message().contains("null free-edge handle"));
    assert!(failure.message().contains("expected bounded pairs"));
}

fn assert_input_edge_alias(
    probe: &Select3DSensitiveTriangulation,
    edges: &Handle<HArray1<i32>>,
    policy: i32,
) {
    if policy != 0 {
        assert_eq!(probe.state.read().free_edges, *edges);
    }
}

fn dirty_connected_pair(
    original: &Select3DSensitiveTriangulation,
    connected: &Select3DSensitiveTriangulation,
) {
    original.set.mark_dirty();
    connected.set.mark_dirty();
}
fn copied_edge_probe(indices: &[i32]) -> Select3DSensitiveTriangulation {
    Select3DSensitiveTriangulation::with_free_edges(
        &SelectMgrEntityOwner::new(13).cast(),
        &oracle_mesh(1),
        &oracle_location(0),
        indices,
        Point3::new(7., 8., 9.),
        false,
    )
    .unwrap()
}
