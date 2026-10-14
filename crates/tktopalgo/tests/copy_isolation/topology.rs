use super::fixture::{SharedCopyFixture, copied_member, copy_fixture, shared_copy_fixture};
use super::geometry::analytic_snapshot;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Transform3;
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};

fn face_edge(face: &Shape) -> Shape {
    let wire = ShapeIterator::new(face, false, false)
        .value()
        .unwrap()
        .clone();
    ShapeIterator::new(&wire, false, false)
        .value()
        .unwrap()
        .clone()
}

#[test]
fn aggregate_copy_preserves_shared_edge_incidence_and_reversed_occurrence() {
    let source = shared_copy_fixture();
    let (_, copied) = copy_fixture(&source);
    let first = face_edge(&copied.faces[0]);
    let second = face_edge(&copied.faces[1]);
    assert!(first.is_same(&second));
    assert!(first.is_same(&copied.edge));
    let last = ShapeIterator::new(&copied.root, false, false)
        .last()
        .unwrap();
    assert!(last.is_same(&first));
    assert_eq!(last.orientation(), Orientation::Reversed);
    copied.edge.set_closed(true).unwrap();
    assert!(first.closed().unwrap());
    assert!(second.closed().unwrap());
    assert!(!source.edge.closed().unwrap());
}

fn translate_occurrence(shape: &mut Shape, x: f64) {
    let translation = Location::new(Transform3::translation([x, 4.0, 5.0]));
    shape.set_location(&translation, false).unwrap();
    assert_eq!(shape.location(), &translation);
}

#[test]
fn source_and_copy_location_and_orientation_values_mutate_independently() {
    let source = shared_copy_fixture();
    let mut positioned = source.root.reversed();
    translate_occurrence(&mut positioned, 3.0);
    let mut copier = BRepBuilderAPICopy::with_shape(&positioned, true, true).unwrap();
    let mut copied = copier.shape().unwrap().clone();
    assert_eq!(copied.location(), positioned.location());
    assert_eq!(copied.orientation(), Orientation::Reversed);
    translate_occurrence(&mut positioned, 9.0);
    assert_ne!(copied.location(), positioned.location());
    translate_occurrence(&mut copied, 17.0);
    copied.reverse();
    assert_eq!(positioned.orientation(), Orientation::Reversed);
    assert_ne!(copied.location(), positioned.location());
}

fn null_copy_failure(copier: &mut BRepBuilderAPICopy) -> bool {
    let failure = copier.perform(&Shape::default(), true, true).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NullObject");
    assert!(!copier.is_done());
    copier.is_done()
}

fn reuse_trace(
    copier: &mut BRepBuilderAPICopy,
    source: &SharedCopyFixture,
    old: &SharedCopyFixture,
    failed_done: bool,
) -> String {
    super::geometry::mutate_analytic_storage(source, 9.0);
    copier.perform(&source.root, true, true).unwrap();
    let new_edge = copied_member(copier, &source.edge);
    let origin = analytic_snapshot(old).curve_origin.x();
    // OCCT Modifier retains its existing map result, even after failed null input.
    format!(
        "null {}\nreuse {} {} {}\n",
        failed_done as u8,
        copier.is_done() as u8,
        new_edge.is_same(&old.edge) as u8,
        origin
    )
}

#[test]
fn retained_successful_copy_survives_same_builder_failure_then_success() {
    let source = shared_copy_fixture();
    let (mut copier, old) = copy_fixture(&source);
    let old_before = analytic_snapshot(&old);
    let failed_done = null_copy_failure(&mut copier);
    assert_eq!(analytic_snapshot(&old), old_before);
    let replay = reuse_trace(&mut copier, &source, &old, failed_done);
    assert_eq!(replay, include_str!("native_reuse.expected"));
    assert_eq!(analytic_snapshot(&old), old_before);
    let (_, fresh) = copy_fixture(&source);
    assert_eq!(analytic_snapshot(&fresh).curve_origin.x(), 9.0);
    assert_eq!(analytic_snapshot(&old).curve_origin.x(), 0.0);
}

fn change_representation_positions(fixture: &SharedCopyFixture, offset: f64) {
    use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
    let builder = BRepBuilder::default();
    let location = Location::new(Transform3::translation([offset, 0.0, 0.0]));
    let curve = BRepTool::curve_with_location(&fixture.edge).unwrap().curve;
    builder
        .update_edge_located_curve(&fixture.edge, &curve, &location, 0.02)
        .unwrap();
    let surface = BRepTool::surface_with_location(&fixture.faces[0])
        .unwrap()
        .0;
    builder
        .update_face(&fixture.faces[0], &surface, &location, 0.01)
        .unwrap();
}

fn representation_positions(fixture: &SharedCopyFixture) -> [f64; 2] {
    use rustycad_tkbrep::brep::BRepTool;
    let curve = BRepTool::curve_with_location(&fixture.edge).unwrap();
    let (_, face_location) = BRepTool::surface_with_location(&fixture.faces[0]).unwrap();
    [
        curve.location.transformation().translation_part().x(),
        face_location.transformation().translation_part().x(),
    ]
}

#[test]
fn geometric_representation_locations_isolate_bidirectional_updates() {
    let source = shared_copy_fixture();
    let (_, copied) = copy_fixture(&source);
    assert_eq!(representation_positions(&source), [0.0, 0.0]);
    assert_eq!(representation_positions(&copied), [0.0, 0.0]);
    change_representation_positions(&source, 9.0);
    assert_eq!(representation_positions(&source), [9.0, 9.0]);
    assert_eq!(representation_positions(&copied), [0.0, 0.0]);
    change_representation_positions(&copied, 17.0);
    assert_eq!(representation_positions(&source), [9.0, 9.0]);
    assert_eq!(representation_positions(&copied), [17.0, 17.0]);
}
