//! Exact native endpoint-pairing flags and output-reference preservation.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Builder, Compound, Edge, Shape, Vertex, Wire},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisWire,
    shape_extend::{ShapeExtendStatus, ShapeExtendWireData},
};

const EXPECTED: &str =
    include_str!("../../../tools/oracle/tkshhealing_wire_shape_connect.expected");

#[test]
fn endpoint_pairings_ties_and_precision_match_native() {
    for (name, first, last, precision) in [
        ("tail_head", 20., 30., 0.),
        ("tail_tail", 30., 20., 0.),
        ("head_tail", 0., 10., 0.),
        ("head_head", 10., 0., 0.),
        ("equidistant", 15., 15., 5.),
        ("too_far", 0., 1., 0.),
    ] {
        inspect(name, &edge_at(first, last).into_shape(), precision);
    }
}

#[test]
fn unsupported_shapes_preserve_distance_outputs() {
    inspect("null", &Shape::default(), 0.);
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    inspect("compound", &compound.into_shape(), 0.);
}

#[test]
fn wire_endpoints_include_reversed_orientation() {
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    let mut shape = wire.clone().into_shape();
    Builder
        .add(&mut shape, &edge_at(20., 30.).into_shape())
        .unwrap();
    inspect("wire", &shape, 0.);
    shape.reverse();
    inspect("reversed_wire", &shape, 0.);
}

fn inspect(name: &str, shape: &Shape, precision: f64) {
    let wire = Handle::new(ShapeExtendWireData::new());
    wire.add_edge(&edge_at(10., 20.), 0).unwrap();
    let analyzer = ShapeAnalysisWire::new();
    analyzer.load_wire_data(&wire);
    let mut distances = [42., 43., 44., 45.];
    let found = analyzer
        .check_shape_connect_into(&mut distances, shape, precision)
        .unwrap();
    let mut observed = connection_flags(&analyzer, found);
    observed.extend(distances);
    observed.extend([analyzer.min_distance_3d(), analyzer.max_distance_3d()]);
    let row = EXPECTED
        .lines()
        .find(|row| row.starts_with(&format!("{name} ")))
        .unwrap();
    let expected: Vec<f64> = row
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(observed, expected, "native connection {name}");
    assert_eq!(
        analyzer.check_shape_connect(shape, precision).unwrap(),
        found
    );
}

fn connection_flags(analyzer: &ShapeAnalysisWire, found: bool) -> Vec<f64> {
    let statuses = [
        ShapeExtendStatus::Done1,
        ShapeExtendStatus::Done2,
        ShapeExtendStatus::Done3,
        ShapeExtendStatus::Done4,
        ShapeExtendStatus::Done5,
        ShapeExtendStatus::Done6,
        ShapeExtendStatus::Fail1,
        ShapeExtendStatus::Fail2,
    ];
    let mut observed = vec![f64::from(found)];
    observed.extend(statuses.map(|status| f64::from(analyzer.last_check_status(status))));
    observed
}

fn edge_at(first: f64, last: f64) -> Edge {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    let mut shape = edge.clone().into_shape();
    for (x, orientation) in [(first, Orientation::Forward), (last, Orientation::Reversed)] {
        let mut vertex = Vertex::default();
        builder
            .make_vertex_at(&mut vertex, Point3::new(x, 0., 0.), 1e-7)
            .unwrap();
        Builder
            .add(&mut shape, &vertex.oriented(orientation))
            .unwrap();
    }
    edge
}

#[test]
fn disconnected_wire_error_preserves_outputs_after_done_status() {
    let mut bad = Wire::default();
    Builder.make_wire(&mut bad);
    let mut shape = bad.into_shape();
    Builder
        .add(&mut shape, &edge_at(20., 30.).into_shape())
        .unwrap();
    Builder
        .add(&mut shape, &edge_at(30., 40.).into_shape())
        .unwrap();
    let wire = Handle::new(ShapeExtendWireData::new());
    wire.add_edge(&edge_at(10., 20.), 0).unwrap();
    let analyzer = ShapeAnalysisWire::new();
    analyzer.load_wire_data(&wire);
    let mut distances = [42., 43., 44., 45.];
    let error = analyzer
        .check_shape_connect_into(&mut distances, &shape, 0.)
        .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NullObject");
    assert!(analyzer.last_check_status(ShapeExtendStatus::Done1));
    assert_eq!(distances, [42., 43., 44., 45.]);
    assert!(
        EXPECTED
            .lines()
            .any(|row| row == "disconnected_wire_error 1 1 42 43 44 45")
    );
}
