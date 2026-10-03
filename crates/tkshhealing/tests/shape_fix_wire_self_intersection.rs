//! Native `ShapeFix_Wire::FixSelfIntersection` aggregate control flow.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis2d, Axis3, Direction2, Point2};
use rustycad_tkshhealing::{shape_extend::ShapeExtendStatus, shape_fix::ShapeFixWire};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeWire};

const NATIVE: &str =
    include_str!("../../../tools/oracle/tkshhealing_fix_wire_self_intersection.expected");

#[test]
fn simple_wire_and_unready_statuses_match_native_aggregate() {
    compare_row("simple", observe_simple_wire());
    compare_row("unready", observe_unready_wire());
    compare_row("modes", observe_modes());
}

fn observe_modes() -> Vec<i32> {
    let fixer = ShapeFixWire::new();
    fixer.set_fix_self_intersecting_edge_mode(1);
    fixer.set_fix_intersecting_edges_mode(0);
    fixer.set_fix_nonadjacent_intersecting_edges_mode(-1);
    vec![
        fixer.fix_self_intersecting_edge_mode(),
        fixer.fix_intersecting_edges_mode(),
        fixer.fix_nonadjacent_intersecting_edges_mode(),
    ]
}

fn observe_simple_wire() -> Vec<i32> {
    let (wire, face) = simple_wire();
    let fixer = ShapeFixWire::from_wire_face(&wire, &face, 1e-7).unwrap();
    let fixed = fixer.fix_self_intersection().unwrap();
    vec![
        i32::from(fixed),
        fixer.nb_edges(),
        i32::from(fixer.status_self_intersection(ShapeExtendStatus::Done)),
        i32::from(fixer.status_self_intersection(ShapeExtendStatus::Fail)),
        i32::from(fixer.status_removed_segment()),
    ]
}

fn observe_unready_wire() -> Vec<i32> {
    let fixer = ShapeFixWire::new();
    let fixed = fixer.fix_self_intersection().unwrap();
    vec![
        i32::from(fixed),
        i32::from(fixer.status_self_intersection(ShapeExtendStatus::Done)),
        i32::from(fixer.status_self_intersection(ShapeExtendStatus::Fail)),
    ]
}

fn simple_wire() -> (rustycad_tkbrep::topods::Wire, Face) {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    let first = edge(
        &surface,
        Point2::new(0., 0.),
        Direction2::new(1., 0.).unwrap(),
        2.,
    );
    let second = edge(
        &surface,
        Point2::new(2., 0.),
        Direction2::new(0., 1.).unwrap(),
        2.,
    );
    let third = edge(
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

fn edge(
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

fn compare_row(name: &str, actual: Vec<i32>) {
    let row = NATIVE
        .lines()
        .find(|line| line.starts_with(&format!("{name} ")))
        .expect("native aggregate row");
    let expected: Vec<i32> = row
        .split_whitespace()
        .skip(2)
        .step_by(2)
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(actual, expected, "native FixSelfIntersection case {name}");
}
