//! Behavior tests of BRepClass after the cases of the upstream suite
//! `BRepClass_FaceExplorer_Test.cxx`, which is not mapped: its faces come from
//! BRepBuilderAPI_MakeEdge/MakeWire/MakeFace and BRepPrimAPI_MakeSphere (TKTopAlgo
//! BRepBuilderAPI and TKPrim), which the port does not provide yet. Here the same faces
//! are built with BRep_Builder by the probe's face reader (the records of
//! tools/tktopalgo_corpus_faces.py), and each case checks the same properties.
#[allow(dead_code)]
#[path = "../examples/tp_probe/face.rs"]
mod face;
#[allow(dead_code)]
#[path = "../../tkg2d/examples/g2_probe/print.rs"]
mod print;

use face::{TpFace, tp_face};
use print::Reader;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Wire};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::{Direction2, Line2, Point2};
use rustycad_tktopalgo::brep_class::{
    BRepClassEdge, BRepClassFaceExplorer, BRepClassIntersector, BndBoxState, FClassifier,
};
use std::f64::consts::{FRAC_PI_2, PI, TAU};

/// `holed_face()`: the square [0, 10]^2 with the hole [3, 7]^2 as a reversed wire.
const HOLED: [f64; 128] = [
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1e-07, 8.0, 0.0, 0.0, 1e-07, 10.0, 0.0, 1e-07, 10.0, 10.0,
    1e-07, 0.0, 10.0, 1e-07, 3.0, 3.0, 1e-07, 7.0, 3.0, 1e-07, 7.0, 7.0, 1e-07, 3.0, 7.0, 1e-07,
    8.0, 0.0, 0.0, 1.0, 0.0, 1e-07, 0.0, 0.0, 10.0, 0.0, 0.0, 1.0, 2.0, 0.0, 1e-07, 10.0, 0.0,
    10.0, 10.0, 0.0, 2.0, 3.0, 0.0, 1e-07, 10.0, 10.0, 0.0, 10.0, 0.0, 3.0, 0.0, 0.0, 1e-07, 0.0,
    10.0, 0.0, 0.0, 0.0, 4.0, 5.0, 0.0, 1e-07, 3.0, 3.0, 7.0, 3.0, 0.0, 5.0, 6.0, 0.0, 1e-07, 7.0,
    3.0, 7.0, 7.0, 0.0, 6.0, 7.0, 0.0, 1e-07, 7.0, 7.0, 3.0, 7.0, 0.0, 7.0, 4.0, 0.0, 1e-07, 3.0,
    7.0, 3.0, 3.0, 2.0, 0.0, 4.0, 0.0, 0.0, 1.0, 0.0, 2.0, 0.0, 3.0, 0.0, 1.0, 4.0, 4.0, 0.0, 5.0,
    0.0, 6.0, 0.0, 7.0, 0.0, 0.0,
];

/// `cylinder_face()`: the lateral face of the cylinder of radius 2, u in [0, 2pi], v in
/// [0, 5], with its seam used FORWARD and REVERSED.
const CYLINDER: [f64; 62] = [
    1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1e-07, 2.0, 0.0, 0.0, 1e-07, 0.0, 5.0, 1e-07, 3.0, 4.0,
    0.0, 0.0, 0.0, 1e-07, 0.0, 0.0, 1.0, 0.0, 0.0, TAU, 3.0, 0.0, 1.0, 0.0, 1e-07, TAU, 0.0, TAU,
    5.0, -TAU, 0.0, 4.0, 1.0, 1.0, 0.0, 1e-07, 0.0, 5.0, 1.0, 0.0, 0.0, TAU, 1.0, 0.0, 4.0, 0.0,
    0.0, 1.0, 0.0, 2.0, 1.0, 1.0, 1.0, 0.0,
];

/// `square_face()` with its wire REVERSED: the square [0, 2]^2.
const REVERSED_SQUARE: [f64; 70] = [
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1e-07, 4.0, 0.0, 0.0, 1e-07, 2.0, 0.0, 1e-07, 2.0, 2.0,
    1e-07, 0.0, 2.0, 1e-07, 4.0, 0.0, 0.0, 1.0, 0.0, 1e-07, 0.0, 0.0, 2.0, 0.0, 0.0, 1.0, 2.0, 0.0,
    1e-07, 2.0, 0.0, 2.0, 2.0, 0.0, 2.0, 3.0, 0.0, 1e-07, 2.0, 2.0, 0.0, 2.0, 0.0, 3.0, 0.0, 0.0,
    1e-07, 0.0, 2.0, 0.0, 0.0, 1.0, 1.0, 4.0, 0.0, 0.0, 1.0, 0.0, 2.0, 0.0, 3.0, 0.0, 0.0,
];

/// `square_face()`: the square [0, 2]^2 with a FORWARD wire.
const SQUARE: [f64; 70] = [
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1e-07, 4.0, 0.0, 0.0, 1e-07, 2.0, 0.0, 1e-07, 2.0, 2.0,
    1e-07, 0.0, 2.0, 1e-07, 4.0, 0.0, 0.0, 1.0, 0.0, 1e-07, 0.0, 0.0, 2.0, 0.0, 0.0, 1.0, 2.0, 0.0,
    1e-07, 2.0, 0.0, 2.0, 2.0, 0.0, 2.0, 3.0, 0.0, 1e-07, 2.0, 2.0, 0.0, 2.0, 0.0, 3.0, 0.0, 0.0,
    1e-07, 0.0, 2.0, 0.0, 0.0, 1.0, 0.0, 4.0, 0.0, 0.0, 1.0, 0.0, 2.0, 0.0, 3.0, 0.0, 0.0,
];

/// `disc_face()`: the disc of radius 2, one closed circle edge on one vertex.
const DISC: [f64; 30] = [
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1e-07, 1.0, 2.0, 0.0, 1e-07, 1.0, 1.0, 0.0, 0.0, 0.0, 1e-07,
    0.0, 0.0, 2.0, 0.0, TAU, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0,
];

/// `sphere_face()`: the sphere of radius 2, its seam and its two degenerated poles.
const SPHERE: [f64; 62] = [
    2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1e-07, 2.0, 0.0, -FRAC_PI_2, 1e-07, 0.0, FRAC_PI_2,
    1e-07, 3.0, 4.0, 0.0, 0.0, 1.0, 1e-07, 0.0, -FRAC_PI_2, 1.0, 0.0, 0.0, TAU, 3.0, 0.0, 1.0, 0.0,
    1e-07, TAU, -FRAC_PI_2, TAU, FRAC_PI_2, -TAU, 0.0, 4.0, 1.0, 1.0, 1.0, 1e-07, 0.0, FRAC_PI_2,
    1.0, 0.0, 0.0, TAU, 1.0, 0.0, 4.0, 0.0, 0.0, 1.0, 0.0, 2.0, 1.0, 1.0, 1.0, 0.0,
];

fn face(record: &[f64]) -> TpFace {
    tp_face(&mut Reader::new(record)).expect("face record")
}

/// `classify`: the state of `point` by BRepClass_FClassifier at Precision::Confusion().
fn classify(face: &Face, point: Point2, use_bnd_box: bool) -> State {
    let mut explorer = BRepClassFaceExplorer::new(face).unwrap();
    explorer.set_use_bnd_box(use_bnd_box);
    FClassifier::with_point(&mut explorer, &point, precision::confusion())
        .unwrap()
        .state()
}

fn shapes_of(shape: &Shape, kind: ShapeEnum) -> Vec<Shape> {
    Explorer::new(shape, kind, ShapeEnum::Shape)
        .unwrap()
        .collect()
}

/// `referenceNextEdge`: the other edge at the last vertex of `edge` in `wire` when
/// exactly two edges meet there.
fn reference_next_edge(edge: &Edge, wire: &Shape) -> Edge {
    let (first, last) = topexp::vertices(edge, true);
    if last.is_null() || last.is_same(&first) {
        return Edge::default();
    }
    let mut vertex_edges = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(wire, ShapeEnum::Vertex, ShapeEnum::Edge, &mut vertex_edges)
        .unwrap();
    let Some(edges) = vertex_edges.seek(&last) else {
        return Edge::default();
    };
    if edges.extent() != 2 {
        return Edge::default();
    }
    let mut next = Edge::default();
    for candidate in edges.iter() {
        if !candidate.is_null() && !candidate.is_same(edge) {
            next = topods::edge(candidate).unwrap();
        }
    }
    next
}

/// `expectNextEdgesMatchReference`: the explorer's edges are the wire edges in order and
/// each next edge is the reference one.
fn expect_next_edges_match_reference(face: &Face) {
    let mut explorer = BRepClassFaceExplorer::new(face).unwrap();
    let wires = shapes_of(face, ShapeEnum::Wire);
    let mut wire_index = 0;
    explorer.init_wires();
    while explorer.more_wires() {
        let wire: &Shape = &wires[wire_index];
        let edges = shapes_of(wire, ShapeEnum::Edge);
        let mut edge_index = 0;
        explorer.init_edges();
        while explorer.more_edges() {
            let (data, _) = explorer.current_edge().unwrap();
            let expected = topods::edge(&edges[edge_index]).unwrap();
            assert!(data.edge().is_equal(&expected));
            let expected_next = reference_next_edge(&expected, wire);
            if expected_next.is_null() {
                assert!(data.next_edge().is_null());
            } else {
                assert!(data.next_edge().is_equal(&expected_next));
            }
            explorer.next_edge();
            edge_index += 1;
        }
        assert_eq!(edge_index, edges.len());
        explorer.next_wire();
        wire_index += 1;
    }
    assert_eq!(wire_index, wires.len());
}

#[test]
fn multi_wire_hole_bounding_boxes_preserve_classification() {
    let holed = face(&HOLED);
    let points = [
        Point2::new(1.0, 1.0),
        Point2::new(5.0, 5.0),
        Point2::new(12.0, 5.0),
    ];
    let expected = [State::In, State::Out, State::Out];
    for (point, state) in points.into_iter().zip(expected) {
        assert_eq!(classify(&holed.face, point, false), state);
        assert_eq!(classify(&holed.face, point, true), state);
    }
}

#[test]
fn cylindrical_seam_preserves_wire_occurrence_order() {
    let cylinder = face(&CYLINDER);
    let expected: Vec<Shape> = shapes_of(&cylinder.face, ShapeEnum::Wire)
        .iter()
        .flat_map(|wire| shapes_of(wire, ShapeEnum::Edge))
        .collect();
    let has_seam = (0..expected.len())
        .any(|i| (i + 1..expected.len()).any(|j| expected[i].is_same(&expected[j])));
    assert!(has_seam);
    let mut explorer = BRepClassFaceExplorer::new(&cylinder.face).unwrap();
    let mut index = 0;
    explorer.init_wires();
    while explorer.more_wires() {
        explorer.init_edges();
        while explorer.more_edges() {
            let (data, orientation) = explorer.current_edge().unwrap();
            assert!(data.edge().is_equal(&expected[index]));
            assert_eq!(orientation, expected[index].orientation());
            index += 1;
            explorer.next_edge();
        }
        explorer.next_wire();
    }
    assert_eq!(index, expected.len());
    expect_next_edges_match_reference(&cylinder.face);
    let middle = Point2::new(PI, 2.5);
    assert_eq!(classify(&cylinder.face, middle, false), State::In);
    assert_eq!(classify(&cylinder.face, middle, true), State::In);
    assert_eq!(
        classify(&cylinder.face, Point2::new(PI, 6.0), true),
        State::Out
    );
}

#[test]
fn reversed_wire_next_edges_match_reference() {
    let square = face(&REVERSED_SQUARE);
    let wire = Wire::unchecked(shapes_of(&square.face, ShapeEnum::Wire).remove(0));
    assert_eq!(wire.orientation(), Orientation::Reversed);
    expect_next_edges_match_reference(&square.face);
}

#[test]
fn single_closed_edge_has_no_next_edge() {
    let disc = face(&DISC);
    let mut explorer = BRepClassFaceExplorer::new(&disc.face).unwrap();
    explorer.init_wires();
    assert!(explorer.more_wires());
    explorer.init_edges();
    assert!(explorer.more_edges());
    let (data, _) = explorer.current_edge().unwrap();
    assert!(data.next_edge().is_null());
    explorer.next_edge();
    assert!(!explorer.more_edges());
    expect_next_edges_match_reference(&disc.face);
}

#[test]
fn degenerate_edges_have_no_next_edge() {
    let sphere = face(&SPHERE);
    let mut has_degenerate_edge = false;
    let mut explorer = BRepClassFaceExplorer::new(&sphere.face).unwrap();
    explorer.init_wires();
    while explorer.more_wires() {
        explorer.init_edges();
        while explorer.more_edges() {
            let (data, _) = explorer.current_edge().unwrap();
            if BRepTool::degenerated(data.edge()).unwrap() {
                has_degenerate_edge = true;
                assert!(data.next_edge().is_null());
            }
            explorer.next_edge();
        }
        explorer.next_wire();
    }
    expect_next_edges_match_reference(&sphere.face);
    assert!(has_degenerate_edge);
}

/// The first two edges of the square face, and the pcurve of the first one.
fn square_edges() -> (Face, Edge, Edge) {
    let square = face(&SQUARE);
    let edges = shapes_of(&square.face, ShapeEnum::Edge);
    let (first, second) = (
        topods::edge(&edges[0]).unwrap(),
        topods::edge(&edges[1]).unwrap(),
    );
    (square.face, first, second)
}

#[test]
fn set_topology_invalidates_derived_data() {
    let (square, edge, next_edge) = square_edges();
    let pcurve = BRepTool::curve_on_face(&edge, &square).unwrap();
    assert!(!pcurve.pcurve.is_null());
    let mut data = BRepClassEdge::with_edge_face(edge.clone(), square.clone());
    data.set_geometry(pcurve.pcurve.clone(), pcurve.first, pcurve.last);
    let mut bounds = BoundingBox2::default();
    bounds.update_bounds([0.0, 0.0], [2.0, 2.0]);
    data.set_bounding_box(&bounds);
    data.set_next_edge(next_edge);
    data.set_edge(Edge::unchecked(edge.reversed()));
    assert!(data.curve().is_null());
    assert!(data.next_edge().is_null());
    assert!(data.bounding_box().is_void());
    assert_eq!(data.bounding_box_state(), BndBoxState::NotBuilt);
    assert_eq!(data.first_parameter(), 0.0);
    assert_eq!(data.last_parameter(), 0.0);
    data.set_geometry(pcurve.pcurve, pcurve.first, pcurve.last);
    data.set_bounding_box(&bounds);
    data.set_face(Face::unchecked(square.reversed()));
    assert!(data.curve().is_null());
    assert!(data.bounding_box().is_void());
    assert_eq!(data.bounding_box_state(), BndBoxState::NotBuilt);
}

/// Port deviation (docs/tktopalgo-progress.md): the face bounds of a face without a
/// surface cannot be computed. Natively CheckPoint dereferences the null surface; the
/// port keeps the failure in the explorer and the classification returns it.
#[test]
fn face_without_surface_classification_returns_the_bounds_failure() {
    let mut explorer = BRepClassFaceExplorer::new(&Face::default()).unwrap();
    let classified = FClassifier::with_point(&mut explorer, &Point2::new(1.0, 1.0), 1e-7);
    let failure = classified.err().expect("the null face has no bounds");
    assert!(!failure.exception_type().is_empty());
    let mut point = Point2::new(1.0, 1.0);
    assert!(explorer.check_point(&mut point).is_err());
}

#[test]
fn void_bounding_box_becomes_unavailable() {
    let mut data = BRepClassEdge::new();
    data.set_bounding_box(&BoundingBox2::default());
    assert!(data.bounding_box().is_void());
    assert_eq!(data.bounding_box_state(), BndBoxState::Unavailable);
}

#[test]
fn not_built_and_unavailable_bounding_boxes_intersect_without_cache_writes() {
    let (square, edge, _) = square_edges();
    let pcurve = BRepTool::curve_on_face(&edge, &square).unwrap();
    assert!(!pcurve.pcurve.is_null());
    let mut data = BRepClassEdge::with_edge_face(edge, square);
    data.set_geometry(pcurve.pcurve.clone(), pcurve.first, pcurve.last);
    data.set_use_bnd_box(true);
    let (point, tangent) = pcurve
        .pcurve
        .d1(0.5 * (pcurve.first + pcurve.last))
        .unwrap();
    assert!(tangent.square_magnitude() > precision::square_p_confusion());
    let normal = Direction2::new(-tangent.y(), tangent.x()).unwrap();
    let line = Line2::new(point, normal);
    let mut not_built = BRepClassIntersector::new();
    not_built
        .perform(&line, real_last(), precision::confusion(), &data)
        .unwrap();
    assert!(not_built.is_done());
    assert_eq!(data.bounding_box_state(), BndBoxState::NotBuilt);
    data.set_bounding_box_unavailable();
    let mut unavailable = BRepClassIntersector::new();
    unavailable
        .perform(&line, real_last(), precision::confusion(), &data)
        .unwrap();
    assert!(unavailable.is_done());
    assert_eq!(data.bounding_box_state(), BndBoxState::Unavailable);
}
