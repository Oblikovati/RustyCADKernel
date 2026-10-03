//! Native parity for `ShapeUpgrade_RemoveInternalWires` on a real planar face with a hole.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topexp::Explorer,
    topods::{Compound, Shape, ShapeIterator, Wire},
};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus, shape_upgrade::ShapeUpgradeRemoveInternalWires,
};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeWire,
};

#[test]
fn full_face_scan_and_threshold_match_native() {
    assert_native_case("remove_hole", configure(2.0, true), true);
    assert_native_case("remove_hole_no_face_mode", configure(2.0, false), true);
    assert_native_case("threshold_keeps_hole", configure(0.5, true), false);
    assert_native_case("nan_threshold_keeps_hole", configure(f64::NAN, true), false);
}

#[test]
fn selected_wire_scan_matches_native() {
    let (shape, hole) = fixture();
    let remover = ShapeUpgradeRemoveInternalWires::with_shape(&shape).unwrap();
    remover.set_min_area(2.0);
    remover.set_remove_face_mode(false);
    let mut selected = Sequence::default();
    selected.append(hole.into_shape());

    assert!(remover.perform_on(&selected).unwrap());
    assert_native_row("selected_hole", &remover, true);
}

#[test]
fn removes_adjacent_face_whose_outer_edges_all_belong_to_the_small_wire() {
    let shape = partition_fixture(1);
    let remover = ShapeUpgradeRemoveInternalWires::with_shape(&shape).unwrap();
    remover.set_min_area(2.0);

    assert!(remover.perform().unwrap());
    assert_native_row("remove_small_face", &remover, true);
}

#[test]
fn processes_multiple_adjacent_candidates_in_native_order() {
    let shape = partition_fixture(2);
    let remover = ShapeUpgradeRemoveInternalWires::with_shape(&shape).unwrap();
    remover.set_min_area(2.0);

    assert!(remover.perform().unwrap());
    assert_native_row("multiple_adjacent_faces", &remover, true);
}

#[test]
fn selected_foreign_face_is_processed_and_reports_missing_edge_ancestors() {
    let (input, _) = fixture();
    let (foreign, _) = fixture();
    let remover = ShapeUpgradeRemoveInternalWires::with_shape(&input).unwrap();
    remover.set_min_area(2.0);
    let mut selection = Sequence::default();
    selection.append(foreign);

    assert!(remover.perform_on(&selection).unwrap());
    assert_native_row("foreign_face", &remover, true);
}

#[test]
fn empty_input_reports_initial_shape_failure() {
    let remover = ShapeUpgradeRemoveInternalWires::new();

    assert!(!remover.perform().unwrap());
    assert!(remover.status(ShapeExtendStatus::Fail1));
    assert!(remover.removed_wires().sequence(|seq| seq.is_empty()));
}

#[test]
fn init_resets_threshold_but_keeps_recorded_context_replacements() {
    let (shape, _) = fixture();
    let remover = ShapeUpgradeRemoveInternalWires::with_shape(&shape).unwrap();
    remover.set_min_area(2.0);
    assert!(remover.perform().unwrap());

    remover.init(&shape).unwrap();
    assert_eq!(remover.min_area(), 0.0);
    assert!(remover.remove_face_mode());
    assert!(!remover.perform().unwrap());
    assert_native_row("persistent_context", &remover, false);
}

fn configure(area: f64, remove_faces: bool) -> ShapeUpgradeRemoveInternalWires {
    let (shape, _) = fixture();
    let remover = ShapeUpgradeRemoveInternalWires::with_shape(&shape).unwrap();
    remover.set_min_area(area);
    remover.set_remove_face_mode(remove_faces);
    assert_eq!(remover.min_area().is_nan(), area.is_nan());
    if !area.is_nan() {
        assert_eq!(remover.min_area(), area);
    }
    assert_eq!(remover.remove_face_mode(), remove_faces);
    remover
}

fn assert_native_case(name: &str, remover: ShapeUpgradeRemoveInternalWires, expected: bool) {
    assert_eq!(remover.perform().unwrap(), expected, "{name} done");
    assert_native_row(name, &remover, expected);
}

fn assert_native_row(name: &str, remover: &ShapeUpgradeRemoveInternalWires, expected: bool) {
    let values = expected_row(name);
    assert_eq!(values[1] == "1", expected);
    assert_statuses(name, remover, &values);
    assert_result_counts(remover, &values);
}

fn assert_statuses(name: &str, remover: &ShapeUpgradeRemoveInternalWires, values: &[String]) {
    for (column, status) in [
        ShapeExtendStatus::Done1,
        ShapeExtendStatus::Done2,
        ShapeExtendStatus::Fail1,
        ShapeExtendStatus::Fail2,
    ]
    .iter()
    .enumerate()
    {
        assert_eq!(
            remover.status(*status),
            values[column + 2] == "1",
            "{name} {status:?}"
        );
    }
}

fn assert_result_counts(remover: &ShapeUpgradeRemoveInternalWires, values: &[String]) {
    assert_eq!(
        remover.removed_wires().sequence(|seq| seq.len()),
        values[6].parse().unwrap()
    );
    assert_eq!(
        remover.removed_faces().sequence(|seq| seq.len()),
        values[7].parse().unwrap()
    );
    assert_eq!(
        count(&remover.get_result(), ShapeEnum::Face),
        values[8].parse().unwrap()
    );
    assert_eq!(
        count(&remover.get_result(), ShapeEnum::Wire),
        values[9].parse().unwrap()
    );
}

fn fixture() -> (Shape, Wire) {
    let outer = rectangle(0.0, 10.0);
    let hole = rectangle(2.0, 3.0);
    let mut face = BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &outer, true)
        .unwrap()
        .face()
        .unwrap();
    let mut internal = hole.clone();
    internal.set_orientation(Orientation::Internal);
    BRepBuilder::default()
        .add(&mut face, &internal.into_shape())
        .unwrap();
    (face.into_shape(), hole)
}

fn partition_fixture(adjacent_faces: usize) -> Shape {
    let (shape, hole) = fixture();
    let mut compound = Compound::default();
    BRepBuilder::default().make_compound(&mut compound);
    BRepBuilder::default().add(&mut compound, &shape).unwrap();
    for _ in 0..adjacent_faces {
        let filler = filler_face(&hole);
        BRepBuilder::default()
            .add(&mut compound, &filler.into_shape())
            .unwrap();
    }
    compound.into_shape()
}

fn filler_face(hole: &Wire) -> rustycad_tkbrep::topods::Face {
    let mut shared_edges = BRepBuilderAPIMakeWire::new();
    for edge in ShapeIterator::new(&hole.clone().into_shape(), false, false) {
        shared_edges
            .add_edge(&rustycad_tkbrep::topods::edge(&edge).unwrap())
            .unwrap();
    }
    let filler_wire = shared_edges.wire().unwrap();
    BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &filler_wire, true)
        .unwrap()
        .face()
        .unwrap()
}

fn rectangle(low: f64, high: f64) -> Wire {
    let points = [
        Point3::new(low, low, 0.0),
        Point3::new(high, low, 0.0),
        Point3::new(high, high, 0.0),
        Point3::new(low, high, 0.0),
    ];
    let mut wire = BRepBuilderAPIMakeWire::new();
    for index in 0..points.len() {
        let next = (index + 1) % points.len();
        let edge = BRepBuilderAPIMakeEdge::from_points(points[index], points[next])
            .unwrap()
            .edge()
            .unwrap();
        wire.add_edge(&edge).unwrap();
    }
    wire.wire().unwrap()
}

fn count(shape: &Shape, kind: ShapeEnum) -> usize {
    Explorer::new(shape, kind, ShapeEnum::Shape)
        .unwrap()
        .count()
}

fn expected_row(name: &str) -> Vec<String> {
    include_str!("../../../tools/oracle/tkshhealing_remove_internal_wires.expected")
        .lines()
        .find(|line| line.starts_with(&format!("{name} ")))
        .expect("matching native oracle row")
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}

#[path = "support/remove_internal_wires/observed_curve.rs"]
mod observed_curve;

#[test]
fn geometry_callback_reads_incremental_removal_state_without_deadlock() {
    let (sender, receiver) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || sender.send(observe_second_hole()).unwrap());
    assert_eq!(
        receiver
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap(),
        2
    );
    worker.join().unwrap();
}

fn observe_second_hole() -> usize {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let (mut shape, _) = fixture();
    let mut second = rectangle(4., 5.);
    second.set_orientation(Orientation::Internal);
    BRepBuilder::default()
        .add(&mut shape, &second.clone().into_shape())
        .unwrap();
    let remover = Arc::new(ShapeUpgradeRemoveInternalWires::with_shape(&shape).unwrap());
    remover.set_min_area(2.0);
    remover.set_remove_face_mode(false);
    let observed = Arc::new(AtomicUsize::new(0));
    observe_wire_geometry(&second, &remover, &observed);
    assert!(remover.perform().unwrap());
    assert_eq!(remover.removed_wires().sequence(|seq| seq.len()), 2);
    observed.load(Ordering::SeqCst)
}

fn observe_wire_geometry(
    wire: &Wire,
    remover: &std::sync::Arc<ShapeUpgradeRemoveInternalWires>,
    observed: &std::sync::Arc<std::sync::atomic::AtomicUsize>,
) {
    use rustycad_tkbrep::{brep::BRepTool, topods};
    use rustycad_tkernel::standard::Handle;
    for child in ShapeIterator::new(wire, false, true) {
        let edge = topods::edge(&child).unwrap();
        let (basis, _, _) = BRepTool::curve(&edge).unwrap();
        let observer = Handle::new(observed_curve::RemovalObservingCurve::new(
            basis, remover, observed,
        ));
        BRepBuilder::default()
            .update_edge_curve(
                &edge,
                &observer.cast(),
                BRepTool::edge_tolerance(&edge).unwrap(),
            )
            .unwrap();
    }
}

#[test]
fn protected_clear_keeps_result_context_and_configuration() {
    let remover = configure(2.0, false);
    assert!(remover.perform().unwrap());
    let result = remover.get_result();
    let removed = remover.removed_wires();
    assert_eq!(removed.sequence(|seq| seq.len()), 1);
    let context = remover.context();
    remover.clear();
    assert!(remover.removed_wires().sequence(|seq| seq.is_empty()));
    assert!(remover.removed_faces().sequence(|seq| seq.is_empty()));
    assert!(remover.status(ShapeExtendStatus::Ok));
    assert_eq!(remover.min_area(), 2.0);
    assert!(!remover.remove_face_mode());
    assert!(result.is_same(&remover.get_result()));
    assert!(context == remover.context());
    assert!(removed == remover.removed_wires());
    assert!(removed.sequence(|seq| seq.is_empty()));
}

#[test]
fn protected_status_storage_matches_native_subclass_access() {
    let remover = ShapeUpgradeRemoveInternalWires::new();
    remover.set_status_bits(0x101);
    let row = format!(
        "protected_status {} {} {}",
        remover.status_bits(),
        i32::from(remover.status(ShapeExtendStatus::Done1)),
        i32::from(remover.status(ShapeExtendStatus::Fail1))
    );
    assert_eq!(row, expected_row("protected_status").join(" "));
    remover.clear();
    assert_eq!(remover.status_bits(), 0);
}
