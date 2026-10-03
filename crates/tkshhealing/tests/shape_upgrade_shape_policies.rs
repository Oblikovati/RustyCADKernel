//! Policy wrappers compared with pinned OCCT state and topology.
use rustycad_tkbrep::{topexp::Explorer, topods::Shape};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    geom::{GeomCircle, GeomCurve, GeomCylindricalSurface, GeomSurface},
    top_abs::ShapeEnum,
};
use rustycad_tkmath::gp::{Axis2, Axis3};
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus,
    shape_upgrade::{
        FaceDivider, ShapeDivider, ShapeUpgradeFaceDivide, ShapeUpgradeShapeDivideAngle,
        ShapeUpgradeShapeDivideClosedEdges, ShapeUpgradeSplitSurface,
    },
};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};

fn cylinder_face() -> Shape {
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.).unwrap()).cast();
    BRepBuilderAPIMakeFace::from_surface_bounds(&surface, (0., 3., 0., 2.), 1e-7)
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}
fn circle_edge() -> Shape {
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::new(Axis2::default(), 1.).unwrap()).cast();
    BRepBuilderAPIMakeEdge::from_curve(&curve)
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
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
fn configure_angle(
    divider: &ShapeUpgradeShapeDivideAngle,
    initial: &Handle<ShapeUpgradeFaceDivide>,
    mode: usize,
) {
    if mode == 1 {
        divider.set_max_angle(1.).unwrap();
    }
    if mode == 2 {
        initial.set_split_surface_tool(&Handle::new(ShapeUpgradeSplitSurface::new()));
    }
    if mode == 3 {
        divider.set_split_face_tool(&Handle::default());
    }
}
fn angle_row(mode: usize) -> Vec<f64> {
    let divider = ShapeUpgradeShapeDivideAngle::with_shape(1.5, cylinder_face());
    let initial = divider.split_face_tool();
    let surface = initial.cast::<dyn FaceDivider>().split_surface_tool();
    configure_angle(&divider, &initial, mode);
    let maximum = divider.max_angle();
    let changed = mode != 3 && divider.perform(true).unwrap();
    let current = divider.split_face_tool();
    vec![
        mode as f64,
        maximum,
        f64::from(changed),
        f64::from(count(&divider.result(), ShapeEnum::Face)),
        f64::from(current == initial),
        f64::from(
            !current.is_null() && current.cast::<dyn FaceDivider>().split_surface_tool() == surface,
        ),
        f64::from(
            current.is_null()
                || current
                    .cast::<dyn FaceDivider>()
                    .wire_divide_tool()
                    .is_null(),
        ),
    ]
}
fn edge_row(requested: i32) -> Vec<f64> {
    let divider = ShapeUpgradeShapeDivideClosedEdges::new(circle_edge());
    let initial = divider.split_face_tool();
    divider.set_nb_split_points(requested);
    let current = divider.split_face_tool();
    let changed = (&divider as &dyn ShapeDivider).perform(true).unwrap();
    vec![
        f64::from(requested),
        f64::from(changed),
        f64::from(count(&divider.result(), ShapeEnum::Edge)),
        f64::from(current == initial),
        f64::from(
            current
                .cast::<dyn FaceDivider>()
                .split_surface_tool()
                .is_null(),
        ),
        f64::from(divider.status_flag(ShapeExtendStatus::Done1)),
        f64::from(divider.status_flag(ShapeExtendStatus::Fail)),
    ]
}
#[test]
fn angle_and_closed_edge_wrappers_match_native() {
    for line in
        include_str!("../../../tools/oracle/tkshhealing_shape_divide_policies.expected").lines()
    {
        let mut words = line.split_whitespace();
        let kind = words.next().unwrap();
        let expected: Vec<f64> = words.map(|word| word.parse().unwrap()).collect();
        let actual = if kind == "angle" {
            angle_row(expected[0] as usize)
        } else {
            edge_row(expected[0] as i32)
        };
        assert_eq!(actual, expected, "{line}");
    }
}
#[test]
fn angle_reset_preserves_face_identity_but_discards_wire_policy() {
    let divider = ShapeUpgradeShapeDivideAngle::new(1.);
    let face = divider.split_face_tool();
    face.set_wire_divide_tool(&Handle::new(
        rustycad_tkshhealing::shape_upgrade::ShapeUpgradeWireDivide::new(),
    ));
    divider.init_tool(2.).unwrap();
    assert!(divider.split_face_tool() == face);
    assert!(face.cast::<dyn FaceDivider>().wire_divide_tool().is_null());
    assert_eq!(divider.max_angle(), 2.);
    assert!(!divider.perform(true).unwrap());
}
