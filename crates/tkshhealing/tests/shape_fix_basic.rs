//! Behavior tests of ShapeFix_Root, ShapeFix_ShapeTolerance and ShapeFix_EdgeConnect.
mod healing_support;

use healing_support::{edges_of, square_face};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::message::Msg;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_extend::{BasicMsgRegistrator, ShapeExtendMsgRegistrator};
use rustycad_tkshhealing::shape_fix::{
    FixRoot, ShapeFixEdgeConnect, ShapeFixRoot, ShapeFixShapeTolerance, need_fix,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

#[test]
fn root_precision_widens_limits_and_set_copies_them() {
    let root = ShapeFixRoot::new();
    root.set_precision(0.5);
    assert_eq!((root.min_tolerance(), root.max_tolerance()), (1e-7, 0.5));
    root.set_precision(1e-9);
    assert_eq!(root.min_tolerance(), 1e-9);
    let other = ShapeFixRoot::new();
    other.set_context(&Handle::new(ShapeBuildReShape::new()));
    root.set(&other);
    assert_eq!(root.precision(), 1e-7);
    assert!(!root.context().is_null());
    assert!(need_fix(-1, true) && !need_fix(0, true) && need_fix(2, false));
}

#[test]
fn root_sends_messages_to_its_registrator() {
    let root = ShapeFixRoot::new();
    let keeper = Handle::new(ShapeExtendMsgRegistrator::new());
    root.set_msg_registrator(&keeper.cast::<dyn BasicMsgRegistrator>());
    let face = square_face(0.0);
    root.send_warning(&face, &Msg::new());
    root.send_fail(&face, &Msg::new());
    keeper.map_shape(|map| assert_eq!(map.find(&face).unwrap().extent(), 2));
    root.set_msg_registrator(&Handle::default());
    root.send_fail_on_shape(&Msg::new());
}

#[test]
fn shape_tolerance_limits_and_forces_tolerances() {
    let face = square_face(0.0);
    let tool = ShapeFixShapeTolerance::new();
    assert!(
        tool.limit_tolerance(&face, 1e-5, 1e-3, ShapeEnum::Edge)
            .unwrap()
    );
    assert_eq!(BRepTool::edge_tolerance(&edges_of(&face)[0]).unwrap(), 1e-5);
    assert!(
        !tool
            .limit_tolerance(&face, 1e-5, 1e-3, ShapeEnum::Edge)
            .unwrap()
    );
    tool.set_tolerance(&face, 1e-6, ShapeEnum::Wire).unwrap();
    let (v, _) = topexp::vertices(&edges_of(&face)[0], false);
    assert_eq!(BRepTool::vertex_tolerance(&v).unwrap(), 1e-6);
    assert!(
        !tool
            .limit_tolerance(&face, -1.0, 1.0, ShapeEnum::Shape)
            .unwrap()
    );
}

fn line(a: (f64, f64, f64), b: (f64, f64, f64)) -> Edge {
    BRepBuilderAPIMakeEdge::from_points(Point3::new(a.0, a.1, a.2), Point3::new(b.0, b.1, b.2))
        .unwrap()
        .edge()
        .unwrap()
}

#[test]
fn edge_connect_shares_a_vertex_between_the_curve_ends() {
    let e1 = line((0., 0., 0.), (1., 0., 0.));
    let e2 = line((1.002, 0., 0.), (2., 0., 0.));
    let mut tool = ShapeFixEdgeConnect::new();
    tool.add_edges(&e1, &e2);
    tool.build().unwrap();
    let (last, first) = (
        topexp::last_vertex(&e1, true),
        topexp::first_vertex(&e2, true),
    );
    assert!(last.is_same(&first));
    let p = BRepTool::pnt(&last).unwrap();
    assert!((p.x() - 1.001).abs() < 1e-12);
    assert!(BRepTool::vertex_tolerance(&last).unwrap() > 1e-3);
}
