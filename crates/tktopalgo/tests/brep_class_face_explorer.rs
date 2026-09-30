//! Translation of BRepClass_FaceExplorer_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871, using its original builders.
#![allow(non_snake_case)]
#[path = "face_explorer_legacy/builders.rs"]
mod builders;
#[path = "face_explorer_legacy/reference.rs"]
mod reference;
use reference::*;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Wire};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::{Direction2, Line2, Point2};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeSphere;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_class::{
    BRepClassEdge, BRepClassFaceExplorer, BRepClassIntersector, BndBoxState, FClassifier,
};
use std::f64::consts::PI;

#[test]
fn BRepClassFaceExplorerTest__MultiWireHole_BoundingBoxesPreserveClassification() {
    let holed = builders::holed_face();
    let points = [
        Point2::new(1.0, 1.0),
        Point2::new(5.0, 5.0),
        Point2::new(12.0, 5.0),
    ];
    let expected = [State::In, State::Out, State::Out];
    for (point, state) in points.into_iter().zip(expected) {
        assert_eq!(classify(&holed, point, false), state);
        assert_eq!(classify(&holed, point, true), state);
    }
}

#[test]
fn BRepClassFaceExplorerTest__CylindricalSeam_PreservesWireOccurrenceOrder() {
    let cylinder = builders::cylinder_face();
    let expected: Vec<Shape> = shapes_of(&cylinder, ShapeEnum::Wire)
        .iter()
        .flat_map(|wire| shapes_of(wire, ShapeEnum::Edge))
        .collect();
    let has_seam = (0..expected.len())
        .any(|i| (i + 1..expected.len()).any(|j| expected[i].is_same(&expected[j])));
    assert!(has_seam);
    assert_cylinder_occurrences(&cylinder, &expected);
    expect_next_edges_match_reference(&cylinder);
    let middle = Point2::new(PI, 2.5);
    assert_eq!(classify(&cylinder, middle, false), State::In);
    assert_eq!(classify(&cylinder, middle, true), State::In);
    assert_eq!(classify(&cylinder, Point2::new(PI, 6.0), true), State::Out);
}

#[test]
fn BRepClassFaceExplorerTest__ReversedWire_NextEdgesMatchReference() {
    let wire = builders::rectangle(0.0, 0.0, 2.0, 2.0);
    assert!(!wire.is_null());
    let reversed = topods::wire(&wire.reversed()).unwrap();
    expect_next_edges_match_reference(&builders::wire_face(&reversed));
}

#[test]
fn BRepClassFaceExplorerTest__SingleClosedEdge_HasNoNextEdge() {
    let disc = builders::disc_face();
    let mut explorer = BRepClassFaceExplorer::new(&disc).unwrap();
    explorer.init_wires();
    assert!(explorer.more_wires());
    explorer.init_edges();
    assert!(explorer.more_edges());
    let (data, _) = explorer.current_edge().unwrap();
    assert!(data.next_edge().is_null());
    explorer.next_edge();
    assert!(!explorer.more_edges());
    expect_next_edges_match_reference(&disc);
}

#[test]
fn BRepClassFaceExplorerTest__DegenerateEdges_HaveNoNextEdge() {
    let mut maker = BRepPrimAPIMakeSphere::new(2.0).unwrap();
    maker.build(&ProgressRange::default()).unwrap();
    assert!(maker.is_done());
    let sphere = maker.shape().unwrap().clone();
    assert!(!sphere.is_null());
    let mut found = false;
    for shape in shapes_of(&sphere, ShapeEnum::Face) {
        let face = topods::face(&shape).unwrap();
        found |= check_degenerate_face(&face);
        expect_next_edges_match_reference(&face);
    }
    assert!(found);
}

fn check_degenerate_face(face: &Face) -> bool {
    let mut explorer = BRepClassFaceExplorer::new(face).unwrap();
    let mut found = false;
    explorer.init_wires();
    while explorer.more_wires() {
        found |= check_degenerate_wire(&mut explorer);
        explorer.next_wire();
    }
    found
}

fn check_degenerate_wire(explorer: &mut BRepClassFaceExplorer) -> bool {
    let mut found = false;
    explorer.init_edges();
    while explorer.more_edges() {
        let (record, _) = explorer.current_edge().unwrap();
        if BRepTool::degenerated(record.edge()).unwrap() {
            found = true;
            assert!(record.next_edge().is_null());
        }
        explorer.next_edge();
    }
    found
}

/// The first two edges of the square face, and the pcurve of the first one.
fn square_edges() -> (Face, Edge, Edge) {
    let square = builders::square_face();
    let edges = shapes_of(&square, ShapeEnum::Edge);
    assert!(!edges.is_empty());
    assert!(edges.len() > 1);
    let (first, second) = (
        topods::edge(&edges[0]).unwrap(),
        topods::edge(&edges[1]).unwrap(),
    );
    (square, first, second)
}

#[test]
fn BRepClassEdgeTest__SetTopology_InvalidatesDerivedData() {
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
    assert_invalidated_topology(&data);
    data.set_geometry(pcurve.pcurve, pcurve.first, pcurve.last);
    data.set_bounding_box(&bounds);
    data.set_face(Face::unchecked(square.reversed()));
    assert!(data.curve().is_null());
    assert!(data.bounding_box().is_void());
    assert_eq!(data.bounding_box_state(), BndBoxState::NotBuilt);
}

#[test]
fn BRepClassEdgeTest__VoidBoundingBox_BecomesUnavailable() {
    let mut data = BRepClassEdge::new();
    data.set_bounding_box(&BoundingBox2::default());
    assert!(data.bounding_box().is_void());
    assert_eq!(data.bounding_box_state(), BndBoxState::Unavailable);
}

#[test]
fn BRepClassIntersectorTest__NotBuiltAndUnavailableBoundingBoxes_IntersectWithoutCacheWrites() {
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
    assert_intersection_cache(&line, &data, BndBoxState::NotBuilt);
    data.set_bounding_box_unavailable();
    assert_intersection_cache(&line, &data, BndBoxState::Unavailable);
}

fn assert_cylinder_occurrences(cylinder: &Face, expected: &[Shape]) {
    let mut explorer = BRepClassFaceExplorer::new(cylinder).unwrap();
    let mut index = 0;
    explorer.init_wires();
    while explorer.more_wires() {
        explorer.init_edges();
        while explorer.more_edges() {
            assert!(index < expected.len());
            let (data, orientation) = explorer.current_edge().unwrap();
            assert!(data.edge().is_equal(&expected[index]));
            assert_eq!(orientation, expected[index].orientation());
            index += 1;
            explorer.next_edge();
        }
        explorer.next_wire();
    }
    assert_eq!(index, expected.len());
}

fn assert_invalidated_topology(data: &BRepClassEdge) {
    assert!(data.curve().is_null());
    assert!(data.next_edge().is_null());
    assert!(data.bounding_box().is_void());
    assert_eq!(data.bounding_box_state(), BndBoxState::NotBuilt);
    assert_eq!(data.first_parameter(), 0.0);
    assert_eq!(data.last_parameter(), 0.0);
}

fn assert_intersection_cache(line: &Line2, edge: &BRepClassEdge, expected: BndBoxState) {
    let mut intersector = BRepClassIntersector::new();
    intersector
        .perform(line, real_last(), precision::confusion(), edge)
        .unwrap();
    assert!(intersector.is_done());
    assert_eq!(edge.bounding_box_state(), expected);
}
