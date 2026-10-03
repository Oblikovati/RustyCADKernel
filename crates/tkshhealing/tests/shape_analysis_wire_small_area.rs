//! Native area screening, exact property fallback and distinct argument-wire behavior.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face, Wire},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisWire,
    shape_extend::{ShapeExtendStatus, ShapeExtendWireData},
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakePolygon;
const EXPECTED: &str = include_str!("../../../tools/oracle/tkshhealing_wire_small_area.expected");

#[test]
fn sampled_and_exact_area_thresholds_match_native() {
    inspect("normal", 1e-7, false, false);
    inspect("below", 0.49, false, false);
    inspect("boundary", 0.5, false, false);
    inspect("above", 0.51, false, false);
    inspect("reversed", 1., true, false);
    inspect("different_argument", 1., false, true);
}

#[test]
fn small_area_failure_statuses_match_native() {
    missing("empty", false, true);
    missing("no_geometry", true, true);
    missing("no_face", true, false);
}

fn inspect(name: &str, precision: f64, reversed: bool, different: bool) {
    let face = planar_face();
    let mut wire = square(1.);
    if reversed {
        wire.reverse();
    }
    let analyzer = ShapeAnalysisWire::from_wire(&wire, &face, precision).unwrap();
    let supplied = if different { square(10.) } else { wire };
    assert_native(name, &analyzer, &supplied);
}

fn missing(name: &str, edge_present: bool, face_present: bool) {
    let face = if face_present {
        planar_face()
    } else {
        Face::default()
    };
    let wire = Handle::new(ShapeExtendWireData::new());
    if edge_present {
        let mut edge = Edge::default();
        BRepBuilder::default().make_edge(&mut edge).unwrap();
        wire.add_edge(&edge, 0).unwrap();
    }
    let analyzer = ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7).unwrap();
    assert_native(name, &analyzer, &square(1.));
}

fn assert_native(name: &str, analyzer: &ShapeAnalysisWire, wire: &Wire) {
    let observed = [
        u8::from(analyzer.check_small_area(wire).unwrap()),
        u8::from(analyzer.last_check_status(ShapeExtendStatus::Done1)),
        u8::from(analyzer.last_check_status(ShapeExtendStatus::Fail1)),
        u8::from(analyzer.last_check_status(ShapeExtendStatus::Fail2)),
    ];
    let row = EXPECTED
        .lines()
        .find(|line| line.starts_with(&format!("{name} ")))
        .unwrap();
    let expected: Vec<u8> = row
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(
        observed.as_slice(),
        expected,
        "native small-area case {name}"
    );
}

fn planar_face() -> Face {
    let mut face = Face::default();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}

fn square(side: f64) -> Wire {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for (x, y) in [(0., 0.), (side, 0.), (side, side), (0., side)] {
        polygon.add_point(Point3::new(x, y, 0.)).unwrap();
    }
    polygon.close().unwrap();
    polygon.wire().unwrap()
}
