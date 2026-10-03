//! Whole-shape closed-face division against the pinned native oracle.
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeCylinder, BRepPrimAPIMakeOneAxis};
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus,
    shape_upgrade::{
        FaceDivider, ShapeDivider, ShapeUpgradeClosedFaceDivide, ShapeUpgradeShapeDivideClosed,
    },
};

fn cylinder_face() -> rustycad_tkbrep::topods::Shape {
    let mut maker = BRepPrimAPIMakeCylinder::new(2.0, 5.0);
    maker.face().unwrap().into_shape()
}

fn count(shape: &rustycad_tkbrep::topods::Shape, kind: ShapeEnum) -> i32 {
    let mut explorer =
        rustycad_tkbrep::topexp::Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut count = 0;
    while explorer.more() {
        count += 1;
        explorer.next().unwrap();
    }
    count
}

fn observe(cuts: i32) -> Vec<i32> {
    let divider = ShapeUpgradeShapeDivideClosed::new(cylinder_face());
    if cuts != 1 {
        divider.set_nb_split_points(cuts);
    }
    let changed = (&divider as &dyn ShapeDivider).perform(true).unwrap();
    let result = divider.result();
    let child = ShapeDivider::split_face_tool(&divider).cast::<ShapeUpgradeClosedFaceDivide>();
    observation_row(&divider, &result, cuts, changed, child)
}

fn observation_row(
    divider: &ShapeUpgradeShapeDivideClosed,
    result: &rustycad_tkbrep::topods::Shape,
    cuts: i32,
    changed: bool,
    child: rustycad_tkernel::standard::Handle<ShapeUpgradeClosedFaceDivide>,
) -> Vec<i32> {
    let mut row = vec![cuts, i32::from(changed), count(result, ShapeEnum::Face)];
    row.push(count(result, ShapeEnum::Edge));
    row.extend(statuses(divider));
    row.push(child.get().unwrap().nb_split_points());
    row.push(i32::from(child.get().unwrap().wire_divide_tool().is_null()));
    row
}

fn statuses(divider: &ShapeUpgradeShapeDivideClosed) -> [i32; 5] {
    [
        ShapeExtendStatus::Done1,
        ShapeExtendStatus::Done2,
        ShapeExtendStatus::Fail1,
        ShapeExtendStatus::Fail2,
        ShapeExtendStatus::Fail3,
    ]
    .map(|flag| i32::from(divider.status_flag(flag)))
}

#[test]
fn closed_face_shape_division_matches_native_rows() {
    for line in
        include_str!("../../../tools/oracle/shape_upgrade_shape_divide_closed.expected").lines()
    {
        let expected: Vec<i32> = line
            .split_whitespace()
            .map(|word| word.parse().unwrap())
            .collect();
        assert_eq!(observe(expected[0]), expected);
    }
}

#[test]
fn each_setter_replaces_the_child_and_nulls_its_wire_policy() {
    let divider = ShapeUpgradeShapeDivideClosed::new(cylinder_face());
    let old_tool = ShapeDivider::split_face_tool(&divider);
    divider.set_nb_split_points(3);
    let new_tool = ShapeDivider::split_face_tool(&divider);
    let old_closed = old_tool.cast::<ShapeUpgradeClosedFaceDivide>();
    let new_closed = new_tool.cast::<ShapeUpgradeClosedFaceDivide>();
    assert_ne!(old_tool.object_address(), new_tool.object_address());
    assert_eq!(old_closed.get().unwrap().nb_split_points(), 1);
    assert_eq!(new_closed.get().unwrap().nb_split_points(), 3);
    assert!(new_closed.get().unwrap().wire_divide_tool().is_null());
}
