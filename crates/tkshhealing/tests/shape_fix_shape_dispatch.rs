//! Native dispatch, status, repeated calls and shared repair configuration.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topexp::Explorer,
    topods::{Compound, Shape},
};
use rustycad_tkernel::message::{ProgressIndicator, ProgressRange, ProgressScope};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus,
    shape_fix::{FixRoot, ShapeFixShape},
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
    include_str!("../../../tools/oracle/tkshhealing_shape_dispatch.expected")
        .lines()
        .find(|line| line.split_whitespace().next() == Some(name))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|token| token.parse().unwrap())
        .collect()
}
fn assert_result(name: &str, tool: &ShapeFixShape, changed: bool) {
    let mut actual = vec![i32::from(changed)];
    for status in [
        ShapeExtendStatus::Done1,
        ShapeExtendStatus::Done2,
        ShapeExtendStatus::Done3,
        ShapeExtendStatus::Done4,
        ShapeExtendStatus::Done5,
        ShapeExtendStatus::Done6,
    ] {
        actual.push(i32::from(tool.status(status)));
    }
    let shape = tool.shape();
    actual.push(shape.shape_type().unwrap() as i32);
    for kind in [
        ShapeEnum::Solid,
        ShapeEnum::Shell,
        ShapeEnum::Face,
        ShapeEnum::Edge,
    ] {
        actual.push(
            Explorer::new(&shape, kind, ShapeEnum::Shape)
                .unwrap()
                .count() as i32,
        );
    }
    actual.push(tool.fix_face_tool().fix_small_area_wire_mode());
    assert_eq!(actual, expected(name), "{name}");
}
struct CancelledProgress;
impl rustycad_tkernel::message::ProgressIndicatorHooks for CancelledProgress {
    fn show(&self, _: &ProgressScope<'_>, _: f64, _: bool) {}
    fn user_break(&self) -> bool {
        true
    }
}
fn run(name: &str, shape: Shape, off: bool) {
    let tool = ShapeFixShape::new(shape);
    if off {
        tool.set_fix_solid_mode(0);
        tool.set_fix_free_shell_mode(0);
        tool.set_fix_free_face_mode(0);
        tool.set_fix_free_wire_mode(0);
    }
    let progress = ProgressIndicator::new(Box::new(CancelledProgress));
    let range = if name == "cancelled" {
        progress.start()
    } else {
        ProgressRange::default()
    };
    assert_result(name, &tool, tool.perform_with_progress(&range).unwrap());
}
#[test]
fn native_repair_dispatch_and_modes() {
    let shape = box_shape();
    for (name, kind) in [
        ("solid", ShapeEnum::Solid),
        ("shell", ShapeEnum::Shell),
        ("face", ShapeEnum::Face),
        ("wire", ShapeEnum::Wire),
        ("edge", ShapeEnum::Edge),
        ("vertex", ShapeEnum::Vertex),
    ] {
        run(
            name,
            Explorer::new(&shape, kind, ShapeEnum::Shape)
                .unwrap()
                .current()
                .clone(),
            false,
        );
    }
    run("solid_off", shape.clone(), true);
    run(
        "wire_off",
        Explorer::new(&shape, ShapeEnum::Wire, ShapeEnum::Shape)
            .unwrap()
            .current()
            .clone(),
        true,
    );
    run("cancelled", shape, false);
}
#[test]
fn repeated_shapes_keep_native_visited_map() {
    let shape = box_shape();
    let tool = ShapeFixShape::new(shape.clone());
    assert_result("repeat_first", &tool, tool.perform().unwrap());
    assert_result("repeat_second", &tool, tool.perform().unwrap());
    tool.init(shape);
    assert_result("after_init", &tool, false);
    assert_result("repeat_init", &tool, tool.perform().unwrap());
}
#[test]
fn compound_dispatch_retains_native_last_child_status() {
    let mut compound = Compound::default();
    let builder = BRepBuilder::default();
    builder.make_compound(&mut compound);
    builder.add(&mut compound, &box_shape()).unwrap();
    builder
        .add(
            &mut compound,
            BRepPrimAPIMakeBox::from_sizes(1., 1., 1.)
                .unwrap()
                .shape()
                .unwrap(),
        )
        .unwrap();
    run("compound", compound.into_shape(), false);
}
#[test]
fn shared_tool_configuration_reaches_wire_fixer() {
    let tool = ShapeFixShape::empty();
    tool.set_precision(0.03);
    tool.set_min_tolerance(0.001);
    tool.set_max_tolerance(0.2);
    let wire = tool.fix_wire_tool();
    assert_eq!(wire.precision(), 0.03);
    assert_eq!(wire.min_tolerance(), 0.001);
    assert_eq!(wire.max_tolerance(), 0.2);
    assert!(tool.fix_edge_tool() == wire.fix_edge_tool());
    assert_eq!(tool.fix_same_parameter_mode(), -1);
    assert_eq!(tool.fix_vertex_position_mode(), 0);
    assert_eq!(tool.fix_vertex_tol_mode(), -1);
    tool.set_fix_same_parameter_mode(0);
    tool.set_fix_vertex_position_mode(1);
    tool.set_fix_vertex_tol_mode(0);
    assert_eq!(tool.fix_same_parameter_mode(), 0);
    assert_eq!(tool.fix_vertex_position_mode(), 1);
    assert_eq!(tool.fix_vertex_tol_mode(), 0);
}

#[test]
fn face_repeat_preserves_native_early_return_mode_mutation() {
    let shape = box_shape();
    let face = Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .current()
        .clone();
    let tool = ShapeFixShape::new(face);
    assert_result("face_first", &tool, tool.perform().unwrap());
    assert_result("face_repeat", &tool, tool.perform().unwrap());
}
#[test]
fn vertex_position_mode_runs_before_native_dispatch() {
    let tool = ShapeFixShape::new(box_shape());
    tool.set_fix_vertex_position_mode(1);
    assert_result("vertex_position", &tool, tool.perform().unwrap());
}
#[test]
fn message_registrator_reaches_shared_wire_tool() {
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkshhealing::shape_extend::{BasicMsgRegistrator, ShapeExtendMsgRegistrator};
    let tool = ShapeFixShape::default();
    let messages = Handle::new(ShapeExtendMsgRegistrator::new()).cast::<dyn BasicMsgRegistrator>();
    tool.set_msg_registrator(&messages);
    assert!(tool.fix_wire_tool().msg_registrator() == messages);
}
