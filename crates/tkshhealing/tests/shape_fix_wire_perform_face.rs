//! Face-ready `ShapeFix_Wire::Perform` native pass-order regression.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis2d, Axis3, Direction2, Point2};
use rustycad_tkshhealing::{shape_extend::ShapeExtendStatus, shape_fix::ShapeFixWire};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeWire};

const NATIVE: &str = include_str!("../../../tools/oracle/tkshhealing_wire_perform_face.expected");

#[test]
fn face_ready_perform_runs_native_repair_order_and_statuses() {
    let (wire, face) = simple_wire();
    let fixer = ShapeFixWire::from_wire_face(&wire, &face, 1e-7).unwrap();
    let fixed = fixer.perform().unwrap();
    let actual = [
        i32::from(fixed),
        fixer.nb_edges(),
        i32::from(fixer.status_reorder(ShapeExtendStatus::Done)),
        i32::from(fixer.status_small(ShapeExtendStatus::Done)),
        i32::from(fixer.status_connected(ShapeExtendStatus::Done)),
        i32::from(fixer.status_edge_curves(ShapeExtendStatus::Done)),
        i32::from(fixer.status_degenerated(ShapeExtendStatus::Done)),
        i32::from(fixer.status_notches(ShapeExtendStatus::Done)),
        i32::from(fixer.status_fix_tails(ShapeExtendStatus::Done)),
        i32::from(fixer.status_self_intersection(ShapeExtendStatus::Done)),
        i32::from(fixer.status_lacking(ShapeExtendStatus::Done)),
        i32::from(fixer.status_closed(ShapeExtendStatus::Done)),
    ];
    assert_eq!(actual, native_values());
}

fn simple_wire() -> (rustycad_tkbrep::topods::Wire, Face) {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    let first = pcurve(
        &surface,
        Point2::new(0., 0.),
        Direction2::new(1., 0.).unwrap(),
        2.,
    );
    let second = pcurve(
        &surface,
        Point2::new(2., 0.),
        Direction2::new(0., 1.).unwrap(),
        2.,
    );
    let third = pcurve(
        &surface,
        Point2::new(0., 2.),
        Direction2::new(1., -1.).unwrap(),
        2.8284271247461903,
    );
    let wire = BRepBuilderAPIMakeWire::from_three_edges(&first, &second, &third)
        .unwrap()
        .wire()
        .unwrap();
    (wire, face)
}

fn pcurve(
    surface: &Handle<dyn GeomSurface>,
    origin: Point2,
    direction: Direction2,
    length: f64,
) -> rustycad_tkbrep::topods::Edge {
    let curve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::new(Axis2d::new(origin, direction))).cast();
    BRepBuilderAPIMakeEdge::from_pcurve_params(&curve, surface, 0., length)
        .unwrap()
        .edge()
        .unwrap()
}

fn native_values() -> [i32; 12] {
    let row = NATIVE
        .lines()
        .find(|line| line.starts_with("face "))
        .expect("native face Perform row");
    row.split_whitespace()
        .skip(2)
        .step_by(2)
        .map(|value| value.parse().unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}
