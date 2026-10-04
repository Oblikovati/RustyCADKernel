use super::*;
use rustycad_tkbo::bop_tools;
use rustycad_tkbrep::{
    topexp::Explorer,
    topods::{ShapeIterator, Wire},
};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
#[test]
fn native_sampled_wire_hole_detection_matches() {
    for line in include_str!("../../../../tools/oracle/tkbo_wire_hole.expected").lines() {
        let row: Vec<usize> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let (mut face, mut wire) = hole_fixture(row[0], row[1]);
        let orientations = [
            Orientation::Forward,
            Orientation::Reversed,
            Orientation::Internal,
            Orientation::External,
        ];
        wire.set_orientation(orientations[row[2]]);
        face.set_orientation(orientations[row[3]]);
        assert_eq!(
            bop_tools::is_hole(&wire, &face).unwrap(),
            row[4] != 0,
            "{line}"
        );
    }
}
fn hole_fixture(kind: usize, mode: usize) -> (rustycad_tkbrep::topods::Face, Wire) {
    let fixture = fixture(if kind == 5 { 0 } else { kind });
    let wire = if kind == 5 {
        circle_wire()
    } else {
        let mut explorer = Explorer::new(&fixture.face, ShapeEnum::Wire, ShapeEnum::Shape).unwrap();
        Wire::unchecked(Iterator::next(&mut explorer).unwrap())
    };
    (fixture.face, with_missing_pcurve(wire, mode))
}
fn circle_wire() -> Wire {
    let curve: Handle<dyn GeomCurve> =
        Handle::new(super::super::GeomCircle::new(super::super::Axis2::default(), 1.).unwrap())
            .cast();
    let edge = BRepBuilderAPIMakeEdge::from_curve_params(&curve, 0., 2. * std::f64::consts::PI)
        .unwrap()
        .edge()
        .unwrap();
    let mut wire = Wire::default();
    let builder = BRepBuilder::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, &edge).unwrap();
    wire
}
fn with_missing_pcurve(wire: Wire, mode: usize) -> Wire {
    if mode == 0 {
        return wire;
    }
    let builder = BRepBuilder::default();
    let mut composite = Wire::default();
    builder.make_wire(&mut composite);
    let mut empty = Edge::default();
    builder.make_edge(&mut empty).unwrap();
    if mode == 1 {
        builder.add(&mut composite, &empty).unwrap();
    }
    for shape in ShapeIterator::new(&wire, true, true) {
        builder.add(&mut composite, &shape).unwrap();
    }
    if mode == 2 {
        builder.add(&mut composite, &empty).unwrap();
    }
    composite
}
