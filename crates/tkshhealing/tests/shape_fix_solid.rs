//! Native solid-from-shell orientation and retained state.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topexp::Explorer,
    topods::{self, Shape, ShapeIterator, Shell},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus,
    shape_fix::{FixRoot, ShapeFixSolid},
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
fn box_shape() -> Shape {
    BRepPrimAPIMakeBox::from_sizes(2., 3., 4.)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}
fn expected(name: &str) -> Vec<i32> {
    include_str!("../../../tools/oracle/tkshhealing_solid_from_shell.expected")
        .lines()
        .find(|line| line.split_whitespace().next() == Some(name))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|token| token.parse().unwrap())
        .collect()
}
fn assert_shell(name: &str, shell: &Shell) {
    let tool = ShapeFixSolid::new();
    shell.set_free(false).unwrap();
    let solid = tool.solid_from_shell(shell).unwrap();
    let orientation = ShapeIterator::new(&solid, true, true)
        .value()
        .unwrap()
        .orientation();
    let actual = vec![
        i32::from(tool.status_flag(ShapeExtendStatus::Done2)),
        i32::from(shell.free().unwrap()),
        solid.shape_type().unwrap() as i32,
        orientation as i32,
        i32::from(tool.solid().is_null()),
        i32::from(tool.shape().is_null()),
    ];
    assert_eq!(actual, expected(name), "{name}");
    tool.init(&solid);
    assert_eq!(
        vec![
            i32::from(tool.status_flag(ShapeExtendStatus::Done2)),
            i32::from(tool.solid().is_same(&solid)),
            i32::from(tool.shape().is_same(&solid))
        ],
        expected(&format!("{name}_init"))
    );
}
#[test]
fn native_shell_orientation_and_retained_status() {
    let shape = box_shape();
    let mut shell = topods::shell(
        Explorer::new(&shape, ShapeEnum::Shell, ShapeEnum::Shape)
            .unwrap()
            .current(),
    )
    .unwrap();
    assert_shell("box", &shell);
    shell.reverse();
    assert_shell("reversed", &shell);
    let mut empty = Shell::default();
    BRepBuilder::default().make_shell(&mut empty);
    assert_shell("empty", &empty);
    let mut open = Shell::default();
    BRepBuilder::default().make_shell(&mut open);
    let face = Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .current()
        .clone();
    BRepBuilder::default().add(&mut open, &face).unwrap();
    assert_shell("open", &open);
}
#[test]
fn solid_configuration_reaches_shell_and_face_tools() {
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkshhealing::shape_extend::{BasicMsgRegistrator, ShapeExtendMsgRegistrator};
    let solid = topods::solid(&box_shape()).unwrap();
    let tool = ShapeFixSolid::from_solid(&solid);
    assert!(tool.shape().is_same(&solid));
    assert!(tool.solid().is_same(&solid));
    assert_eq!(tool.fix_shell_mode(), -1);
    assert_eq!(tool.fix_shell_orientation_mode(), -1);
    assert!(!tool.create_open_solid_mode());
    tool.set_fix_shell_mode(0);
    tool.set_fix_shell_orientation_mode(1);
    tool.set_create_open_solid_mode(true);
    assert_eq!(tool.fix_shell_mode(), 0);
    assert_eq!(tool.fix_shell_orientation_mode(), 1);
    assert!(tool.create_open_solid_mode());
    let messages = Handle::new(ShapeExtendMsgRegistrator::new()).cast::<dyn BasicMsgRegistrator>();
    tool.set_msg_registrator(&messages);
    tool.set_precision(0.02);
    tool.set_min_tolerance(0.001);
    tool.set_max_tolerance(0.1);
    let face = tool.fix_shell_tool().fix_face_tool();
    assert!(face.msg_registrator() == messages);
    assert_eq!(face.precision(), 0.02);
    assert_eq!(face.min_tolerance(), 0.001);
    assert_eq!(face.max_tolerance(), 0.1);
}
