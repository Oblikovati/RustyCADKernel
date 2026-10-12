//! Actual native root/frame edge aliases remain live through in-place reset and slot reuse.
use super::{StdPrsToolRFace, StdPrsToolRFaceEdgeRef, isoline_face_fixture::isoline_fixture_face};
use crate::select3d::compare_oracle_snapshot;
use rustycad_tkbrep::{brep_adaptor::BRepAdaptorSurface, topods::Edge};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::Orientation;
const NATIVE_EDGE: &str = include_str!("../../tests/fixtures/tool-r-face/native-edge.txt");
#[test]
fn native_edge_root_and_iterator_slot_lifetimes_and_shared_flags() {
    let mut rows = NATIVE_EDGE.lines();
    for scenario in 0..2 {
        replay_edge_alias_case(scenario, &mut rows);
    }
    assert!(rows.next().is_none());
}
fn replay_edge_alias_case(scenario: i32, rows: &mut std::str::Lines<'_>) {
    let tool = edge_tool_fixture(scenario);
    let root = tool.edge_ref().unwrap();
    let mut step = 0;
    compare_edge_step(&tool, &root, None, scenario, &mut step, rows);
    tool.init().unwrap();
    let slot = tool.edge_ref().unwrap();
    compare_edge_step(&tool, &root, Some(&slot), scenario, &mut step, rows);
    let mut detached = slot.snapshot().unwrap();
    detached.set_orientation(Orientation::Internal);
    detached.set_modified(false).unwrap();
    compare_edge_step(&tool, &root, Some(&slot), scenario, &mut step, rows);
    traverse_edge_alias_pass(&tool, &root, &slot, scenario, &mut step, rows);
    tool.init().unwrap();
    compare_edge_step(&tool, &root, Some(&slot), scenario, &mut step, rows);
    traverse_edge_alias_pass(&tool, &root, &slot, scenario, &mut step, rows);
}
fn traverse_edge_alias_pass(
    tool: &StdPrsToolRFace,
    root: &StdPrsToolRFaceEdgeRef<'_>,
    slot: &StdPrsToolRFaceEdgeRef<'_>,
    scenario: i32,
    step: &mut i32,
    rows: &mut std::str::Lines<'_>,
) {
    while tool.more() {
        tool.next().unwrap();
        compare_edge_step(tool, root, Some(slot), scenario, step, rows);
    }
}
fn compare_edge_step(
    tool: &StdPrsToolRFace,
    root: &StdPrsToolRFaceEdgeRef<'_>,
    slot: Option<&StdPrsToolRFaceEdgeRef<'_>>,
    scenario: i32,
    step: &mut i32,
    rows: &mut std::str::Lines<'_>,
) {
    let actual = edge_alias_snapshot(tool, root, slot, scenario, *step);
    compare_oracle_snapshot(&actual, rows.next().unwrap());
    *step += 1;
}
fn edge_alias_snapshot(
    tool: &StdPrsToolRFace,
    root: &StdPrsToolRFaceEdgeRef<'_>,
    slot: Option<&StdPrsToolRFaceEdgeRef<'_>>,
    scenario: i32,
    step: i32,
) -> Vec<f64> {
    let mut row = edge_snapshot_prefix(tool, slot, scenario, step);
    append_edge_reference_snapshot(&root.snapshot().unwrap(), &mut row);
    row.push(f64::from(slot.is_some()));
    if let Some(slot) = slot {
        append_edge_reference_snapshot(&slot.snapshot().unwrap(), &mut row);
    }
    append_current_edge_curve(tool, &mut row);
    row
}
fn append_edge_reference_snapshot(edge: &Edge, row: &mut Vec<f64>) {
    row.extend([
        f64::from(edge.is_null()),
        edge.orientation() as i32 as f64,
        f64::from(edge.location().is_identity()),
    ]);
    if edge.is_null() {
        row.extend([-1., -1.]);
        return;
    }
    row.extend([
        edge.shape_type().unwrap() as i32 as f64,
        f64::from(edge.modified().unwrap()),
    ]);
}
fn append_current_edge_curve(tool: &StdPrsToolRFace, row: &mut Vec<f64>) {
    if !tool.more() {
        row.extend([0., 0.]);
        return;
    }
    let curve = tool.value();
    let point = curve
        .value((curve.first_parameter().unwrap() + curve.last_parameter().unwrap()) * 0.5)
        .unwrap();
    row.extend([point.x(), point.y()]);
}

fn edge_tool_fixture(scenario: i32) -> StdPrsToolRFace {
    let face = isoline_fixture_face(if scenario == 0 { 0 } else { 9 });
    let source = Handle::new(BRepAdaptorSurface::with_face(&face, true).unwrap());
    StdPrsToolRFace::with_surface(&source).unwrap()
}
fn edge_snapshot_prefix(
    tool: &StdPrsToolRFace,
    slot: Option<&StdPrsToolRFaceEdgeRef<'_>>,
    scenario: i32,
    step: i32,
) -> Vec<f64> {
    let current = tool.edge_ref();
    let same = current
        .as_ref()
        .is_ok_and(|current| slot.is_some_and(|slot| slot.is_same_reference(current)));
    vec![
        f64::from(scenario),
        f64::from(step),
        f64::from(tool.more()),
        f64::from(edge_acquisition_failure_code(current.as_ref().err())),
        f64::from(same),
    ]
}

fn edge_acquisition_failure_code(failure: Option<&rustycad_tkernel::standard::Failure>) -> i32 {
    let Some(failure) = failure else { return 0 };
    if failure.exception_type() == "Standard_TypeMismatch" {
        1
    } else {
        2
    }
}
