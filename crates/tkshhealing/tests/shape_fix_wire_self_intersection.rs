//! End-to-end native parity for the pinned ShapeFix_Wire bowtie self-intersection fixture.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Edge, ShapeIterator, Wire};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisWire;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tkshhealing::shape_fix::{FixRoot, ShapeFixWire};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon};

#[test]
fn bowtie_wire_splits_both_diagonals_at_one_shared_vertex() {
    let wire = bowtie_wire();
    let face = BRepBuilderAPIMakeFace::from_wire(&wire, true)
        .unwrap()
        .face()
        .unwrap();
    let face_wire = rustycad_tkbrep::topods::wire(
        ShapeIterator::new(&face.clone().into_shape(), false, true)
            .value()
            .unwrap(),
    )
    .unwrap();
    let analyzer = Handle::new(ShapeAnalysisWire::from_wire(&face_wire, &face, 1e-7).unwrap());
    let source_edges = ShapeIterator::new(&face_wire.clone().into_shape(), false, true)
        .map(|shape| rustycad_tkbrep::topods::edge(&shape).unwrap())
        .collect::<Vec<Edge>>();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);

    assert!(fixer.fix_line_self_intersections().unwrap());
    assert_eq!(analyzer.wire_data().nb_edges(), 6);
    assert!(fixer.status_self_intersection(ShapeExtendStatus::Done1));
    assert!(fixer.status_self_intersection(ShapeExtendStatus::Done5));
    assert_native_order_and_shared_vertex(&analyzer.wire_data().wire().unwrap(), &face);
    let context = fixer.context();
    let context = context.get().unwrap();
    assert_eq!(
        source_edges
            .iter()
            .filter(|edge| context.is_recorded(&(*edge).clone().into_shape()))
            .count(),
        2
    );
}

fn bowtie_wire() -> Wire {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    polygon.add_point(Point3::new(0.0, 0.0, 0.0)).unwrap();
    polygon.add_point(Point3::new(1.0, 1.0, 0.0)).unwrap();
    polygon.add_point(Point3::new(1.0, 0.0, 0.0)).unwrap();
    polygon.add_point(Point3::new(0.0, 1.0, 0.0)).unwrap();
    polygon.close().unwrap();
    polygon.wire().unwrap()
}

fn assert_native_order_and_shared_vertex(wire: &Wire, face: &rustycad_tkbrep::topods::Face) {
    let mut crossing = Vec::new();
    let mut endpoints = Vec::new();
    for edge_shape in ShapeIterator::new(wire, false, true) {
        let edge = rustycad_tkbrep::topods::edge(&edge_shape).unwrap();
        let (first, last) = rustycad_tkbrep::topexp::vertices(&edge, false);
        let pc = BRepTool::curve_on_face(&edge, face).unwrap();
        endpoints.push((
            pc.pcurve.value(pc.first).unwrap(),
            pc.pcurve.value(pc.last).unwrap(),
            (pc.first, pc.last),
        ));
        for vertex in [first, last] {
            if BRepTool::pnt(&vertex)
                .is_ok_and(|point| point.distance(&Point3::new(0.5, 0.5, 0.0)) < 1e-7)
            {
                crossing.push(vertex);
            }
        }
    }
    assert_eq!(crossing.len(), 4);
    assert!(crossing.iter().all(|vertex| vertex.is_same(&crossing[0])));
    assert!(
        (BRepTool::vertex_tolerance(&crossing[0]).unwrap()
            - rustycad_tkernel::precision::confusion())
        .abs()
            < 1e-12
    );
    let expected = [
        ((0.5, 0.5), (0.5, -0.5), (0.0, 1.0)),
        (
            (0.0, 0.0),
            (0.5, 0.5),
            (2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt()),
        ),
        ((-0.5, -0.5), (0.0, 0.0), (0.0, 2.0_f64.sqrt() / 2.0)),
        ((-0.5, 0.5), (-0.5, -0.5), (0.0, 1.0)),
        (
            (0.0, 0.0),
            (-0.5, 0.5),
            (2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt()),
        ),
        ((0.5, -0.5), (0.0, 0.0), (0.0, 2.0_f64.sqrt() / 2.0)),
    ];
    for (actual, (first, last, range)) in endpoints.into_iter().zip(expected) {
        assert!(
            actual
                .0
                .distance(rustycad_tkmath::gp::Point2::new(first.0, first.1))
                < 1e-7
        );
        assert!(
            actual
                .1
                .distance(rustycad_tkmath::gp::Point2::new(last.0, last.1))
                < 1e-7
        );
        assert!((actual.2.0 - range.0).abs() < 1e-7);
        assert!((actual.2.1 - range.1).abs() < 1e-7);
    }
}
