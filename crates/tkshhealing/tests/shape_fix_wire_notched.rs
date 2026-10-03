//! Native-backed ShapeFix_Wire::FixNotchedEdges edge split and status.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Face, Wire};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis3, Direction2, Point2};
use rustycad_tkshhealing::{shape_extend::ShapeExtendStatus, shape_fix::ShapeFixWire};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeWire};

const NATIVE: &str = include_str!("../../../tools/oracle/tkshhealing_fix_wire_notched.expected");

#[test]
fn aggregate_mode_accessors_and_removed_segment_match_native() {
    let fixer = ShapeFixWire::new();
    fixer.set_fix_small_mode(1);
    fixer.set_fix_connected_mode(2);
    fixer.set_fix_edge_curves_mode(3);
    fixer.set_fix_degenerated_mode(4);
    fixer.set_fix_self_intersection_mode(5);
    fixer.set_fix_lacking_mode(6);
    let expected = native_named_values(
        "modes",
        &[
            "small",
            "connected",
            "edge_curves",
            "degenerated",
            "self_intersection",
            "lacking",
            "removed",
        ],
    );
    assert_eq!(
        [
            fixer.fix_small_mode(),
            fixer.fix_connected_mode(),
            fixer.fix_edge_curves_mode(),
            fixer.fix_degenerated_mode(),
            fixer.fix_self_intersection_mode(),
            fixer.fix_lacking_mode(),
            i32::from(fixer.status_removed_segment()),
        ],
        expected
    );
}

#[test]
fn overlapping_face_pcurves_split_and_remove_the_short_edge_like_native() {
    let (wire, face) = overlapping_wire();
    let fixer = ShapeFixWire::from_wire_face(&wire, &face, 1e-7).unwrap();
    let fixed = fixer.fix_notched_edges().unwrap();
    let actual = [
        i32::from(fixed),
        fixer.nb_edges(),
        i32::from(fixer.last_fix_status(ShapeExtendStatus::Done1)),
        i32::from(fixer.last_fix_status(ShapeExtendStatus::Done2)),
        i32::from(fixer.status_notches(ShapeExtendStatus::Done)),
        i32::from(fixer.last_fix_status(ShapeExtendStatus::Ok)),
    ];
    assert_eq!(actual, native_values());
}

fn overlapping_wire() -> (Wire, Face) {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    let mut overlap = pcurve_edge(&surface, 1., 1., 1.);
    overlap.reverse();
    let first = pcurve_edge(&surface, 0., 1., 2.);
    let close = pcurve_edge(&surface, 1., -1., 1.);
    let wire = BRepBuilderAPIMakeWire::from_three_edges(&first, &overlap, &close)
        .unwrap()
        .wire()
        .unwrap();
    (wire, face)
}

fn pcurve_edge(
    surface: &Handle<dyn GeomSurface>,
    x: f64,
    dx: f64,
    length: f64,
) -> rustycad_tkbrep::topods::Edge {
    let curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(x, 0.),
        Direction2::new(dx, 0.).unwrap(),
    ))
    .cast();
    BRepBuilderAPIMakeEdge::from_pcurve_params(&curve, surface, 0., length)
        .unwrap()
        .edge()
        .unwrap()
}

fn native_values() -> [i32; 6] {
    native_named_values(
        "notch",
        &[
            "fixed",
            "nb_edges",
            "done1",
            "done2",
            "notches_done",
            "last_ok",
        ],
    )
}

fn native_named_values<const N: usize>(name: &str, selected: &[&str; N]) -> [i32; N] {
    let row = NATIVE
        .lines()
        .find(|line| line.starts_with(&format!("{name} ")))
        .expect("native ShapeFix_Wire row");
    let tokens: Vec<_> = row.split_whitespace().collect();
    let values: Vec<i32> = tokens
        .windows(2)
        .filter(|pair| selected.contains(&pair[0]))
        .map(|pair| pair[1].parse().unwrap())
        .collect();
    values.try_into().unwrap()
}
