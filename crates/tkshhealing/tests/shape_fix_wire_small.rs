//! Native-backed FixSmall and FixConnected behavior for ShapeFix_Wire.
use rustycad_tkbrep::topods::{Builder, Wire};
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tkshhealing::shape_fix::ShapeFixWire;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

#[test]
fn small_distinct_vertex_edge_requires_topology_mode() {
    let wire = short_edge_wire();
    let fixer = ShapeFixWire::new();
    fixer.load(&wire).unwrap();

    assert!(!fixer.fix_small_at(1, false, 2e-5).unwrap());
    assert!(fixer.last_fix_status(ShapeExtendStatus::Fail2));
    assert_eq!(fixer.wire_data().unwrap().nb_edges(), 3);
}

#[test]
fn topology_mode_removes_small_edge_and_reconnects_neighbors() {
    let wire = short_edge_wire();
    let fixer = ShapeFixWire::new();
    fixer.load(&wire).unwrap();
    fixer.set_topology_mode(true);

    assert!(fixer.fix_small_at(1, false, 2e-5).unwrap());
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done2));
    assert!(!fixer.last_fix_status(ShapeExtendStatus::Fail3));
    assert_eq!(fixer.wire_data().unwrap().nb_edges(), 2);
}

fn short_edge_wire() -> rustycad_tkbrep::topods::Wire {
    let points = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1e-5, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
    ];
    let edges = [
        BRepBuilderAPIMakeEdge::from_points(points[0], points[1])
            .unwrap()
            .edge()
            .unwrap(),
        BRepBuilderAPIMakeEdge::from_points(points[1], points[2])
            .unwrap()
            .edge()
            .unwrap(),
        BRepBuilderAPIMakeEdge::from_points(points[2], points[0])
            .unwrap()
            .edge()
            .unwrap(),
    ];
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    for edge in &edges {
        Builder.add(&mut wire, &edge.clone().into_shape()).unwrap();
    }
    wire
}
