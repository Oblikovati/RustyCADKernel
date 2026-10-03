//! Pinned native orchestration, containment, status and volume comparisons.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topexp::Explorer,
    topods::{self, Shape, Shell, Solid},
};
use rustycad_tkernel::message::{ProgressIndicator, ProgressRange, ProgressScope};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus,
    shape_fix::{FixRoot, ShapeFixSolid},
};
use rustycad_tktopalgo::{brep_builder_api::BRepBuilderAPIMakeShape, brep_gprop::BRepGProp};
fn box_shell(x: f64, y: f64, z: f64, size: f64) -> Shell {
    let mut maker = BRepPrimAPIMakeBox::at_point(Point3::new(x, y, z), size, size, size).unwrap();
    topods::shell(
        Explorer::new(maker.shape().unwrap(), ShapeEnum::Shell, ShapeEnum::Shape)
            .unwrap()
            .current(),
    )
    .unwrap()
}
fn count(shape: &Shape, kind: ShapeEnum) -> f64 {
    Explorer::new(shape, kind, ShapeEnum::Shape)
        .unwrap()
        .count() as f64
}
fn expected(name: &str) -> Vec<f64> {
    include_str!("../../../tools/oracle/tkshhealing_solid_perform.expected")
        .lines()
        .find(|line| line.split_whitespace().next() == Some(name))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|token| token.parse().unwrap())
        .collect()
}
struct CancelledProgress;
impl rustycad_tkernel::message::ProgressIndicatorHooks for CancelledProgress {
    fn show(&self, _: &ProgressScope<'_>, _: f64, _: bool) {}
    fn user_break(&self) -> bool {
        true
    }
}
fn run(name: &str, shells: &[Shell], shell_mode: i32, orientation: i32, open: bool) {
    let mut solid = Solid::default();
    let builder = BRepBuilder::default();
    builder.make_solid(&mut solid);
    for shell in shells {
        builder.add(&mut solid, shell).unwrap();
    }
    let tool = ShapeFixSolid::from_solid(&solid);
    tool.set_fix_shell_mode(shell_mode);
    tool.set_fix_shell_orientation_mode(orientation);
    tool.set_create_open_solid_mode(open);
    let progress = ProgressIndicator::new(Box::new(CancelledProgress));
    let range = if name == "cancelled" {
        progress.start()
    } else {
        ProgressRange::default()
    };
    let changed = tool.perform_with_progress(&range).unwrap();
    let actual = observations(&tool, changed);
    let expected = expected(name);
    assert_eq!(actual.len(), expected.len(), "{name}: {actual:?}");
    for (index, (a, b)) in actual.iter().zip(expected).enumerate() {
        assert!(
            (a - b).abs() < 1e-9,
            "{name} field {index}: {a} != {b}; row={actual:?}"
        );
    }
}
fn observations(tool: &ShapeFixSolid, changed: bool) -> Vec<f64> {
    let shape = tool.shape();
    let mut props = GProps::default();
    BRepGProp::volume_properties(&shape, &mut props, false, false, false).unwrap();
    let mut result = vec![
        f64::from(changed),
        f64::from(tool.status_flag(ShapeExtendStatus::Done1)),
        f64::from(tool.status_flag(ShapeExtendStatus::Done2)),
        f64::from(tool.status_flag(ShapeExtendStatus::Done3)),
        shape.shape_type().unwrap() as i32 as f64,
        count(&shape, ShapeEnum::Solid),
        count(&shape, ShapeEnum::Shell),
        count(&shape, ShapeEnum::Face),
        tool.solid().shape_type().unwrap() as i32 as f64,
        f64::from(tool.context().is_null()),
        props.mass(),
    ];
    for shell in Explorer::new(&shape, ShapeEnum::Shell, ShapeEnum::Shape).unwrap() {
        result.extend([
            shell.orientation() as i32 as f64,
            f64::from(shell.closed().unwrap()),
        ]);
    }
    result
}
#[test]
fn native_single_shell_modes_and_cancellation() {
    let outer = box_shell(0., 0., 0., 4.);
    let mut reversed = outer.clone();
    reversed.reverse();
    run("box", std::slice::from_ref(&outer), 0, -1, false);
    run("reversed", &[reversed.clone()], 0, -1, false);
    run("off", &[reversed], 0, 0, false);
    run("full", std::slice::from_ref(&outer), -1, -1, false);
    run("cancelled", &[outer], -1, -1, false);
    run("empty", &[], 0, -1, false);
}
#[test]
fn native_multiple_shell_containment() {
    let outer = box_shell(0., 0., 0., 4.);
    let inner = box_shell(1., 1., 1., 2.);
    run(
        "disconnected",
        &[outer.clone(), box_shell(8., 0., 0., 2.)],
        0,
        -1,
        false,
    );
    run("nested", &[outer.clone(), inner.clone()], 0, -1, false);
    run(
        "island",
        &[outer, inner, box_shell(1.5, 1.5, 1.5, 1.)],
        0,
        -1,
        false,
    );
}
#[test]
fn native_empty_and_open_shells() {
    let mut empty = Shell::default();
    let builder = BRepBuilder::default();
    builder.make_shell(&mut empty);
    run("empty_shell", &[empty], 0, -1, false);
    let outer = box_shell(0., 0., 0., 4.);
    let mut open = Shell::default();
    builder.make_shell(&mut open);
    builder
        .add(
            &mut open,
            Explorer::new(&outer, ShapeEnum::Face, ShapeEnum::Shape)
                .unwrap()
                .current(),
        )
        .unwrap();
    run("open", &[open.clone()], 0, -1, false);
    run("open_allowed", &[open.clone()], 0, -1, true);
    run("mixed", &[outer, open], 0, -1, false);
}
#[test]
fn convenience_perform_matches_empty_native_solid() {
    let mut solid = Solid::default();
    BRepBuilder::default().make_solid(&mut solid);
    assert!(!ShapeFixSolid::from_solid(&solid).perform().unwrap());
}

#[test]
fn shared_face_shells_form_native_compsolid() {
    use rustycad_tkbrep::topods::ShapeIterator;
    let outer = box_shell(0., 0., 0., 4.);
    let source = box_shell(4., 0., 0., 4.);
    let common = Explorer::new(&outer, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .nth(1)
        .unwrap()
        .reversed();
    let mut adjacent = Shell::default();
    let builder = BRepBuilder::default();
    builder.make_shell(&mut adjacent);
    for (index, face) in ShapeIterator::new(&source, true, true).enumerate() {
        builder
            .add(&mut adjacent, if index == 0 { &common } else { &face })
            .unwrap();
    }
    adjacent.set_closed(true).unwrap();
    run("shared_face", &[outer, adjacent], 0, -1, false);
}
