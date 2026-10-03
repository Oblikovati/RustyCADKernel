//! Shell face ordering and state compared with pinned OCCT.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topexp::Explorer,
    topods::{self, Face, Shape, Shell},
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkshhealing::{shape_extend::ShapeExtendStatus, shape_fix::ShapeFixShell};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
fn faces(shape: &Shape) -> Vec<Face> {
    Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .map(|shape| topods::face(&shape).unwrap())
        .collect()
}
fn box_faces() -> Vec<Face> {
    faces(
        BRepPrimAPIMakeBox::from_sizes(2., 3., 4.)
            .unwrap()
            .shape()
            .unwrap(),
    )
}
fn shell(source: &[Face]) -> Shell {
    let mut shell = Shell::default();
    let builder = BRepBuilder::default();
    builder.make_shell(&mut shell);
    for face in source {
        builder.add(&mut shell, face).unwrap();
    }
    shell
}
fn expected(name: &str) -> Vec<i32> {
    include_str!("../../../tools/oracle/tkshhealing_shell_orientation.expected")
        .lines()
        .find(|line| line.split_whitespace().next() == Some(name))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|x| x.parse().unwrap())
        .collect()
}
fn assert_orientation(name: &str, source: &[Face]) {
    let input = shell(source);
    let tool = ShapeFixShell::new();
    let changed = tool
        .fix_face_orientation(&input, true, name.ends_with("nonmanifold"))
        .unwrap();
    let result = tool.shape();
    let output = faces(&result);
    let mut actual = vec![
        i32::from(changed),
        tool.nb_shells(),
        i32::from(tool.status_flag(ShapeExtendStatus::Done2)),
        i32::from(tool.status_flag(ShapeExtendStatus::Fail)),
        result.shape_type().unwrap() as i32,
        output.len() as i32,
        i32::from(tool.shell().closed().unwrap()),
    ];
    for face in output {
        actual.push(
            source
                .iter()
                .position(|other| other.is_same(&face))
                .map_or(0, |i| i as i32 + 1),
        );
        actual.push(face.orientation() as i32);
    }
    assert_eq!(actual, expected(name), "{name}");
}
#[test]
fn native_empty_single_and_disconnected_shells() {
    let source = box_faces();
    assert_orientation("empty", &[]);
    assert_orientation("single", &source[..1]);
    assert_orientation("disconnected", &source[..2]);
    assert_orientation("adjacent", &[source[0].clone(), source[2].clone()]);
}
#[test]
fn native_box_face_flip_and_duplicate_face_order() {
    let source = box_faces();
    assert_orientation("box", &source);
    let mut flipped = source.clone();
    flipped[2].reverse();
    assert_orientation("flipped", &flipped);
    let mut duplicate = source.clone();
    duplicate.push(source[0].clone());
    assert_orientation("duplicate", &duplicate);
}
#[test]
fn shell_configuration_propagates_to_shared_face_tool() {
    use rustycad_tkshhealing::shape_fix::FixRoot;
    let tool = ShapeFixShell::from_shell(&shell(&box_faces()));
    assert_eq!(tool.fix_face_mode(), -1);
    assert_eq!(tool.fix_orientation_mode(), -1);
    tool.set_fix_face_mode(0);
    tool.set_fix_orientation_mode(1);
    tool.set_non_manifold_flag(true);
    tool.set_precision(0.02);
    tool.set_min_tolerance(0.001);
    tool.set_max_tolerance(0.1);
    let face = tool.fix_face_tool();
    assert_eq!(face.precision(), 0.02);
    assert_eq!(face.min_tolerance(), 0.001);
    assert_eq!(face.max_tolerance(), 0.1);
    assert_eq!(tool.fix_face_mode(), 0);
    assert_eq!(tool.fix_orientation_mode(), 1);
    assert!(tool.error_faces().is_null());
    assert_eq!(tool.shell().orientation(), Orientation::Forward);
}

fn shared_copy(face: &Face) -> Face {
    let mut copy = topods::face(&face.empty_copied().unwrap()).unwrap();
    for wire in rustycad_tkbrep::topods::ShapeIterator::new(face, false, true) {
        BRepBuilder::default().add(&mut copy, &wire).unwrap();
    }
    copy
}
#[test]
fn multiply_shared_faces_follow_native_shell_grouping() {
    let source = box_faces();
    let copy = shared_copy(&source[0]);
    let other = shared_copy(&source[0]);
    let triple = [source[0].clone(), copy.clone(), other];
    assert_orientation("triple_faces", &triple);
    assert_orientation("triple_nonmanifold", &triple);
    let mut extra = source;
    extra.push(copy);
    assert_orientation("extra_face", &extra);
    assert_orientation("extra_nonmanifold", &extra);
}

struct CancelledProgress;
impl rustycad_tkernel::message::ProgressIndicatorHooks for CancelledProgress {
    fn show(&self, _: &rustycad_tkernel::message::ProgressScope<'_>, _: f64, _: bool) {}
    fn user_break(&self) -> bool {
        true
    }
}
fn assert_perform(
    name: &str,
    source: &[Face],
    face_mode: i32,
    orientation_mode: i32,
    retain: bool,
) {
    use rustycad_tkernel::message::{ProgressIndicator, ProgressRange};
    use rustycad_tkshhealing::shape_fix::FixRoot;
    let input = shell(source);
    let tool = ShapeFixShell::from_shell(&input);
    if retain {
        tool.fix_face_orientation(&input, true, false).unwrap();
        tool.init(&input);
    }
    tool.set_fix_face_mode(face_mode);
    tool.set_fix_orientation_mode(orientation_mode);
    let progress = ProgressIndicator::new(Box::new(CancelledProgress));
    let range = if name == "perform_cancelled" {
        progress.start()
    } else {
        ProgressRange::default()
    };
    let changed = tool.perform_with_progress(&range).unwrap();
    let actual = vec![
        i32::from(changed),
        tool.nb_shells(),
        i32::from(tool.status_flag(ShapeExtendStatus::Done1)),
        i32::from(tool.status_flag(ShapeExtendStatus::Done2)),
        i32::from(tool.context().is_null()),
        faces(&tool.shape()).len() as i32,
        i32::from(tool.shell().closed().unwrap()),
    ];
    assert_eq!(actual, expected(name), "{name}");
}
#[test]
fn perform_matches_native_modes_retained_status_and_cancellation() {
    let source = box_faces();
    assert_perform("perform_box", &source, -1, -1, false);
    assert_perform("perform_cancelled", &source, -1, -1, false);
    assert_perform("perform_empty", &[], -1, -1, false);
    let mut flipped = source;
    flipped[2].reverse();
    assert_perform("perform_flipped", &flipped, 0, -1, false);
    assert_perform("perform_off", &flipped, 0, 0, false);
    assert_perform("perform_retained", &flipped, 0, 0, true);
}
#[test]
fn no_progress_convenience_uses_the_native_perform_path() {
    let tool = ShapeFixShell::from_shell(&shell(&[]));
    assert!(!tool.perform().unwrap());
    assert_eq!(tool.nb_shells(), 0);
}

fn inconsistent_copy(face: &Face) -> Face {
    use rustycad_tkbrep::topods::ShapeIterator;
    let mut copy = topods::face(&face.empty_copied().unwrap()).unwrap();
    let mut first = true;
    for source_wire in ShapeIterator::new(face, false, true) {
        let mut wire = source_wire.empty_copied().unwrap();
        for mut edge in ShapeIterator::new(&source_wire, false, true) {
            if first {
                edge.reverse();
                first = false;
            }
            BRepBuilder::default().add(&mut wire, &edge).unwrap();
        }
        BRepBuilder::default().add(&mut copy, &wire).unwrap();
    }
    copy
}
#[test]
fn inconsistent_orientation_reports_native_failure() {
    let source = box_faces();
    assert_orientation(
        "inconsistent",
        &[source[0].clone(), inconsistent_copy(&source[0])],
    );
}
