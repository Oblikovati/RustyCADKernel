//! Checked topology faults retain native partial outputs through repair and entity reuse.
use super::primitive_array_matching_tests::matching_volume;
use super::sensitive_set::SetPrimitives;
use super::triangulation_failure::catch_triangulation_failure;
use super::triangulation_oracle_geometry::{oracle_edges, oracle_location, oracle_mesh};
use super::triangulation_oracle_snapshot::{compare_oracle_snapshot, failure_code};
use super::{Select3DSensitiveEntity, Select3DSensitiveSet, Select3DSensitiveTriangulation};
use crate::select_basics::SelectBasicsPickResult;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bvh::BvhPrimitiveSetAccess;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::poly::Triangle;

struct FaultOutputs {
    triangle: Triangle,
    nodes: [Point3; 3],
    pick: SelectBasicsPickResult,
}
impl Default for FaultOutputs {
    fn default() -> Self {
        Self {
            triangle: Triangle::new(77, 78, 79),
            nodes: [
                Point3::new(77., 78., 79.),
                Point3::new(80., 81., 82.),
                Point3::new(83., 84., 85.),
            ],
            pick: SelectBasicsPickResult::default(),
        }
    }
}
fn fault_probe(kind: i32) -> Select3DSensitiveTriangulation {
    Select3DSensitiveTriangulation::with_free_edge_handle(
        &Handle::default(),
        &oracle_mesh(1),
        &oracle_location(i32::from(kind >= 6)),
        &oracle_edges(1, 8),
        Point3::default(),
        kind != 0,
    )
    .unwrap()
}
fn mutate_fault(probe: &Select3DSensitiveTriangulation, kind: i32, repair: bool) {
    match kind {
        0 => mutate_edge_node(probe, if repair { 1 } else { 99 }),
        1 => mutate_primitive_map(probe, repair),
        2 => probe.set.set_detected_idx(if repair { 0 } else { 99 }),
        3..=8 => mutate_triangle_node(probe, (kind - 3) % 3, repair),
        _ => unreachable!(),
    }
}
fn mutate_edge_node(probe: &Select3DSensitiveTriangulation, node: i32) {
    let edges = probe.state.read().free_edges.clone();
    let edges = edges.get().unwrap();
    edges.change_array1().set_value(1, node).unwrap();
}
fn mutate_primitive_map(probe: &Select3DSensitiveTriangulation, repair: bool) {
    let mut state = probe.state.write();
    let map = state.bvh_prim_indexes.as_mut().unwrap();
    map[0] = if repair { 0 } else { 99 };
}
fn mutate_triangle_node(probe: &Select3DSensitiveTriangulation, slot: i32, repair: bool) {
    let mesh = probe.triangulation();
    let mut nodes = [1, 2, 3];
    if !repair {
        nodes[slot as usize] = 99;
    }
    mesh.get()
        .unwrap()
        .set_triangle(1, Triangle::new(nodes[0], nodes[1], nodes[2]))
        .unwrap();
}
fn fault_operation(
    probe: &Select3DSensitiveTriangulation,
    operation: i32,
    outputs: &mut FaultOutputs,
) -> Result<(), Failure> {
    // Catch only the checked native exception analogue; real Rust bugs must remain panics.
    catch_triangulation_failure(|| run_fault_operation(probe, operation, outputs))
}
fn run_fault_operation(
    probe: &Select3DSensitiveTriangulation,
    operation: i32,
    outputs: &mut FaultOutputs,
) -> Result<(), Failure> {
    match operation {
        2 => probe
            .last_detected_triangle_into(&mut outputs.triangle)
            .map(|_| ()),
        3 => probe
            .last_detected_triangle_nodes_into(&mut outputs.triangle, &mut outputs.nodes)
            .map(|_| ()),
        4 => probe
            .matches(&matching_volume(0, false), &mut outputs.pick)
            .map(|_| ()),
        5 => probe.bvh(),
        0 | 1 => checked_fault_box(probe, -operation),
        _ => unreachable!(),
    }
}
fn checked_fault_box(probe: &Select3DSensitiveTriangulation, index: i32) -> Result<(), Failure> {
    // Nonfallible hooks use the same checked Failure payload caught by the public fallible APIs.
    let _ = probe.element_box(index);
    Ok(())
}
fn fault_snapshot(kind: i32, operation: i32) -> Vec<f64> {
    let probe = fault_probe(kind);
    probe.set.set_detected_idx(0);
    mutate_fault(&probe, kind, false);
    let mut failed = FaultOutputs::default();
    let failure = fault_operation(&probe, operation, &mut failed);
    let mut columns = vec![f64::from(kind), f64::from(operation)];
    append_fault_state(&mut columns, &probe, failure.as_ref().err(), &failed);
    mutate_fault(&probe, kind, true);
    let mut repaired = FaultOutputs::default();
    let outcome = fault_operation(&probe, operation, &mut repaired);
    append_fault_state(&mut columns, &probe, outcome.as_ref().err(), &repaired);
    columns.push(f64::from(probe.last_detected_triangle_index()));
    columns
}
fn append_fault_state(
    columns: &mut Vec<f64>,
    probe: &Select3DSensitiveTriangulation,
    failure: Option<&Failure>,
    outputs: &FaultOutputs,
) {
    columns.extend([
        failure_code(failure),
        f64::from(probe.to_build_bvh()),
        f64::from(probe.set.detected_idx()),
    ]);
    append_fault_pick(columns, &outputs.pick);
    columns.push(f64::from(retained_fault_tree_size(probe)));
    let (first, middle, last) = outputs.triangle.get();
    columns.extend([f64::from(first), f64::from(middle), f64::from(last)]);
    for point in outputs.nodes {
        columns.extend([point.x(), point.y(), point.z()]);
    }
}
fn append_fault_pick(columns: &mut Vec<f64>, pick: &SelectBasicsPickResult) {
    let depth = if pick.is_valid() { pick.depth() } else { 0. };
    columns.extend([f64::from(pick.is_valid()), depth]);
}
fn retained_fault_tree_size(probe: &Select3DSensitiveTriangulation) -> i32 {
    if probe.to_build_bvh() {
        return -1;
    }
    let mut content = probe.set.lock_content();
    let mut primitives = SetPrimitives {
        set: probe,
        content: &mut content,
    };
    primitives.bvh().unwrap().length()
}
#[test]
fn native_checked_topology_failure_repair_and_retained_selection_state() {
    let native = include_str!("../../tests/fixtures/sensitive-triangulation-faults.txt");
    for line in native.lines() {
        let indexes: Vec<i32> = line
            .split_whitespace()
            .take(2)
            .map(|value| value.parse().unwrap())
            .collect();
        compare_oracle_snapshot(&fault_snapshot(indexes[0], indexes[1]), line);
    }
    assert_eq!(native.lines().count(), 54);
}
#[test]
fn native_failure_adapter_resumes_unrelated_panics() {
    let result = std::panic::catch_unwind(|| {
        catch_triangulation_failure::<()>(|| std::panic::panic_any(73_i32))
    });
    assert_eq!(*result.unwrap_err().downcast::<i32>().unwrap(), 73);
}
#[test]
fn detection_output_adapters_preserve_sentinels_on_a_miss() {
    let probe = fault_probe(1);
    let mut outputs = FaultOutputs::default();
    assert!(
        !probe
            .last_detected_triangle_into(&mut outputs.triangle)
            .unwrap()
    );
    assert!(
        !probe
            .last_detected_triangle_nodes_into(&mut outputs.triangle, &mut outputs.nodes)
            .unwrap()
    );
    assert_eq!(outputs.triangle.get(), (77, 78, 79));
    assert_eq!(outputs.nodes, FaultOutputs::default().nodes);
}
