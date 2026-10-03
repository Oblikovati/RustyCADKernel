//! Native small-edge midpoint, degeneracy and accumulated-status behavior.
mod support;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis1, Axis2d, Axis3, Direction3, Point3};
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisWire,
    shape_extend::{ShapeExtendStatus, ShapeExtendWireData},
};
const EXPECTED: &str = include_str!("../../../tools/oracle/tkshhealing_wire_small.expected");

#[test]
fn small_edge_geometry_and_identity_match_native() {
    inspect("single", 1, 0, false, 1);
    inspect("no_vertices", 0, 0, false, 2);
    inspect("same_no_geometry", 1, 0, false, 2);
    inspect("distinct_no_geometry", 2, 0, false, 2);
    inspect("far_vertices", 3, 0, false, 2);
    inspect("curve_small", 1, 1, false, 2);
    inspect("curve_loop", 1, 2, false, 2);
    inspect("pcurve_small", 1, 3, false, 2);
}

#[test]
fn small_degenerated_edges_preserve_native_status_priority() {
    inspect("degenerated_pcurve", 1, 3, true, 2);
    inspect("degenerated_no_geometry", 1, 0, true, 2);
    inspect("degenerated_no_vertices", 0, 0, true, 2);
}

fn inspect(name: &str, vertices: u8, geometry: u8, degenerated: bool, count: usize) {
    let face = fixture_face(geometry);
    let edge = fixture_edge(vertices);
    attach_geometry(&edge, &face, geometry);
    BRepBuilder::default()
        .degenerated(&edge, degenerated)
        .unwrap();
    let wire = Handle::new(ShapeExtendWireData::new());
    for _ in 0..count {
        wire.add_edge(&edge, 0).unwrap();
    }
    let analyzer = ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7).unwrap();
    let mut observed = vec![u8::from(analyzer.check_small_at(0, 1.).unwrap())];
    for flag in [
        ShapeExtendStatus::Done1,
        ShapeExtendStatus::Done2,
        ShapeExtendStatus::Fail1,
        ShapeExtendStatus::Fail2,
    ] {
        observed.push(u8::from(analyzer.last_check_status(flag)));
    }
    append_aggregate(&analyzer, &mut observed);
    let row = EXPECTED
        .lines()
        .find(|line| line.starts_with(&format!("{name} ")))
        .unwrap();
    let expected: Vec<u8> = row
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(observed, expected, "native small-edge case {name}");
}

fn append_aggregate(analyzer: &ShapeAnalysisWire, observed: &mut Vec<u8>) {
    observed.extend([
        u8::from(analyzer.check_small(1.).unwrap()),
        u8::from(analyzer.status_small(ShapeExtendStatus::Done)),
        u8::from(analyzer.status_small(ShapeExtendStatus::Fail)),
        u8::from(analyzer.check_small(-1.).unwrap()),
        u8::from(analyzer.status_small(ShapeExtendStatus::Done)),
    ]);
    analyzer.clear_statuses();
    observed.push(u8::from(analyzer.status_small(ShapeExtendStatus::Ok)));
}

fn fixture_face(geometry: u8) -> Face {
    let mut face = Face::default();
    if geometry == 3 {
        let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
        BRepBuilder::default()
            .make_face_on_surface(&mut face, &surface, 1e-7)
            .unwrap();
    }
    face
}

fn fixture_edge(vertices: u8) -> Edge {
    if vertices == 0 {
        let mut edge = Edge::default();
        BRepBuilder::default().make_edge(&mut edge).unwrap();
        return edge;
    }
    let first = support::vertex(0., 0.);
    let last = if vertices == 1 {
        first.clone()
    } else {
        support::vertex(if vertices == 3 { 2. } else { 0. }, 0.)
    };
    support::edge(&first, &last)
}

fn attach_geometry(edge: &Edge, face: &Face, geometry: u8) {
    let builder = BRepBuilder::default();
    if geometry == 1 || geometry == 2 {
        let axis = Axis1::new(
            Point3::new(if geometry == 2 { 10. } else { 0. }, 0., 0.),
            Direction3::new(1., 0., 0.).unwrap(),
        );
        let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(axis)).cast();
        builder.update_edge_curve(edge, &curve, 1e-7).unwrap();
        builder.range(edge, 0., 1., true).unwrap();
    }
    if geometry == 3 {
        let curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::default())).cast();
        builder
            .update_edge_pcurve_on_face(edge, &curve, face, 1e-7)
            .unwrap();
        builder.range_on_face(edge, face, 0., 1.).unwrap();
    }
}
