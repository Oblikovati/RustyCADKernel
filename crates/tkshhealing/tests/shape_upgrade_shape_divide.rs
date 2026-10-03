//! Whole-shape subdivision against the pinned native oracle.
use rustycad_tkbrep::{
    brep_tools::ReShape,
    topexp::Explorer,
    topods::{Builder, Compound, Shape},
};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::{
    gp::{Plane3, Point3, Transform3, Vector3},
    toploc::Location,
};
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus,
    shape_upgrade::{
        FaceDivideData, FaceDivider, ShapeDivider, ShapeUpgradeFaceDivide,
        ShapeUpgradeFaceDivideArea, ShapeUpgradeShapeDivide, ShapeUpgradeTool,
    },
};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};

fn fixture(mode: usize) -> Shape {
    if mode == 12 {
        return vertexless_edge();
    }
    if mode == 0 {
        return Shape::default();
    }
    if [4, 8, 9].contains(&mode) {
        return boundary_fixture(mode == 8);
    }
    let face = BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0., 2., 0., 2.))
        .unwrap()
        .face()
        .unwrap()
        .into_shape();
    if [3, 7, 10, 11].contains(&mode) {
        return assembly(face);
    }
    face
}
fn vertexless_edge() -> Shape {
    let mut edge = rustycad_tkbrep::topods::Edge::default();
    rustycad_tkbrep::brep::BRepBuilder::default()
        .make_edge(&mut edge)
        .unwrap();
    edge.into_shape()
}
fn boundary_fixture(as_wire: bool) -> Shape {
    let edge =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(0., 0., 0.), Point3::new(2., 0., 0.))
            .unwrap()
            .edge()
            .unwrap()
            .into_shape();
    if !as_wire {
        return edge;
    }
    let mut wire = rustycad_tkbrep::topods::Wire::default();
    Builder.make_wire(&mut wire);
    Builder.add(&mut wire, &edge).unwrap();
    wire.into_shape()
}
fn assembly(mut face: Shape) -> Shape {
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    Builder.add(&mut compound, &face).unwrap();
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(0., 0., 5.));
    face.set_location(&Location::new(transform), true).unwrap();
    Builder.add(&mut compound, &face).unwrap();
    let mut shape = compound.into_shape();
    shape.reverse();
    shape
}
fn configure(divider: &ShapeUpgradeShapeDivide, source: &Shape, mode: usize) {
    if [2, 3, 7].contains(&mode) {
        let area = Handle::new(ShapeUpgradeFaceDivideArea::empty());
        area.set_max_area(2.);
        divider.set_split_face_tool(&area.cast());
    }
    configure_wire_policy(divider, mode);
    if mode == 6 {
        divider.context().remove(source).unwrap();
    }
    if mode == 7 {
        divider.context().set_mode_consider_location(true);
    }
}
fn configure_wire_policy(divider: &ShapeUpgradeShapeDivide, mode: usize) {
    if [8, 9, 14].contains(&mode) {
        let face = Handle::new(ShapeUpgradeFaceDivide::empty());
        face.set_wire_divide_tool(&Handle::new(CopyWire::default()).cast());
        divider.set_split_face_tool(&face);
    }
    if mode == 5 || mode == 11 {
        divider.set_split_face_tool(&Handle::new(FailingFace::default()).cast());
    }
    if mode == 13 {
        let face = Handle::new(ShapeUpgradeFaceDivide::empty());
        face.set_wire_divide_tool(&Handle::default());
        divider.set_split_face_tool(&face);
    }
}
fn count(shape: &Shape, kind: ShapeEnum) -> i32 {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut result = 0;
    while explorer.more() {
        result += 1;
        explorer.next().unwrap();
    }
    result
}
fn observe(mode: usize) -> Vec<i32> {
    let source = fixture(mode);
    let divider = ShapeUpgradeShapeDivide::new(source.clone());
    let original_context = divider.context();
    let messages = Handle::new(CountMessages::default());
    divider.set_msg_registrator(&messages.cast());
    assert!(divider.msg_registrator() == messages.cast::<dyn BasicMsgRegistrator>());
    configure(&divider, &source, mode);
    let changed = (&divider as &dyn ShapeDivider)
        .perform(mode != 6 && mode != 7)
        .unwrap();
    observation_row(
        &divider,
        &source,
        mode,
        changed,
        &original_context,
        *messages.count.read(),
    )
}
fn observation_row(
    divider: &ShapeUpgradeShapeDivide,
    source: &Shape,
    mode: usize,
    changed: bool,
    context: &Handle<rustycad_tkshhealing::shape_build::ShapeBuildReShape>,
    message_count: i32,
) -> Vec<i32> {
    let result = divider.result();
    let mut row = vec![
        mode as i32,
        i32::from(changed),
        i32::from(result.is_null()),
        i32::from(result.is_same(source)),
        count(&result, ShapeEnum::Face),
        count(&result, ShapeEnum::Edge),
        i32::from(divider.context() == *context),
        i32::from(divider.with(|f| f.shape.is_same(source))),
    ];
    append_statuses(divider, &mut row);
    row.push(message_count);
    row.extend(placement_counts(&result));
    row
}
fn append_statuses(divider: &ShapeUpgradeShapeDivide, row: &mut Vec<i32>) {
    for flag in [
        ShapeExtendStatus::Done1,
        ShapeExtendStatus::Done2,
        ShapeExtendStatus::Fail1,
        ShapeExtendStatus::Fail2,
        ShapeExtendStatus::Fail3,
    ] {
        row.push(i32::from(divider.status_flag(flag)));
    }
}
#[test]
fn shape_subdivision_matches_native_rows() {
    for (mode, line) in include_str!("../../../tools/oracle/tkshhealing_shape_divide.expected")
        .lines()
        .enumerate()
    {
        let expected: Vec<i32> = line
            .split_whitespace()
            .map(|word| word.parse().unwrap())
            .collect();
        assert_eq!(observe(mode), expected, "mode {mode}");
    }
}
#[derive(Default)]
struct FailingFace {
    parent: ShapeUpgradeFaceDivide,
}
impl_transient!(FailingFace,"ShapeDivideTest_FailingFace",ShapeUpgradeFaceDivide,parent,interfaces:[dyn FaceDivider]);
impl FaceDivider for FailingFace {
    fn face_divide_data(&self) -> &FaceDivideData {
        self.parent.face_divide_data()
    }
    fn face_divide_base(&self) -> &ShapeUpgradeTool {
        self.parent.face_divide_base()
    }
    fn perform(&self, _area: f64) -> Result<bool, Failure> {
        Err(Failure::with_type(
            "Standard_Failure",
            "fixture face failure",
            "",
        ))
    }
}

use rustycad_tkernel::{
    message::{Gravity, Msg},
    standard::SharedState,
};
use rustycad_tkshhealing::{
    shape_extend::{BasicMsgRegistrator, ShapeExtend, ShapeExtendBasicMsgRegistrator},
    shape_upgrade::{ShapeUpgradeWireDivide, WireDivideData, WireDivider},
};
#[derive(Default)]
struct CopyWire {
    parent: ShapeUpgradeWireDivide,
}
impl_transient!(CopyWire,"ShapeDivideTest_CopyWire",ShapeUpgradeWireDivide,parent,interfaces:[dyn WireDivider]);
impl WireDivider for CopyWire {
    fn wire_divide_data(&self) -> &WireDivideData {
        self.parent.wire_divide_data()
    }
    fn wire_divide_base(&self) -> &ShapeUpgradeTool {
        self.parent.wire_divide_base()
    }
    fn perform(&self) -> Result<(), Failure> {
        let wire = self.parent.wire().into_shape();
        let mut copy = wire.empty_copied()?;
        for child in rustycad_tkbrep::topods::ShapeIterator::new(&wire, true, true) {
            Builder.add(&mut copy, &child)?;
        }
        self.wire_divide_data().with_mut(|f| {
            f.wire = rustycad_tkbrep::topods::wire(&copy).unwrap();
            f.status = ShapeExtend::encode_status(ShapeExtendStatus::Done1)
                | ShapeExtend::encode_status(ShapeExtendStatus::Fail1);
        });
        Ok(())
    }
}
#[derive(Default)]
struct CountMessages {
    parent: ShapeExtendBasicMsgRegistrator,
    count: SharedState<i32>,
}
impl_transient!(CountMessages,"ShapeDivideTest_CountMessages",ShapeExtendBasicMsgRegistrator,parent,interfaces:[dyn BasicMsgRegistrator]);
impl BasicMsgRegistrator for CountMessages {
    fn send_shape(&self, _shape: &Shape, _message: &Msg, gravity: Gravity) {
        assert_eq!(gravity, Gravity::Info);
        *self.count.write() += 1;
    }
}
#[test]
fn setters_retain_raw_native_values() {
    let divider = ShapeUpgradeShapeDivide::empty();
    divider.set_precision(-1.);
    divider.set_min_tolerance(-2.);
    divider.set_max_tolerance(-3.);
    divider.set_surface_segment_mode(false);
    divider.set_edge_mode(7);
    assert_eq!(
        divider.with(|f| (
            f.precision,
            f.min_tolerance,
            f.max_tolerance,
            f.segment_mode,
            f.edge_mode
        )),
        (-1., -2., -3., false, 7)
    );
}
#[test]
fn init_preserves_result_and_missing_context_is_recreated() {
    let divider = ShapeUpgradeShapeDivide::empty();
    let shape = fixture(1);
    divider.with_mut(|f| f.result = shape.clone());
    divider.init(&Shape::default());
    assert!(!divider.perform(true).unwrap());
    assert!(divider.result().is_same(&shape));
    divider.set_context(&Handle::default());
    assert!(divider.context().is_null());
    divider.init(&shape);
    divider.perform(false).unwrap();
    assert!(!divider.context().is_null());
}

fn placement_counts(result: &Shape) -> [i32; 2] {
    let mut faces = Explorer::new(result, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let (mut reversed, mut z) = (0, 0);
    while faces.more() {
        let shape = faces.current();
        reversed +=
            i32::from(shape.orientation() == rustycad_tkg3d::top_abs::Orientation::Reversed);
        z += shape.location().transformation().translation_part().z() as i32;
        faces.next().unwrap();
    }
    [reversed, z]
}
