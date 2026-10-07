//! Direct translation of src/ModelingAlgorithms/TKBO/GTests/BRepTools_History_Test.cxx (3d097a0).
#![allow(non_snake_case)]

use rustycad_tkbo::bop_algo::{
    BOPAlgoBOP, BOPAlgoBuilder, BOPAlgoPaveFiller, BOPAlgoSection, Operation,
};
use rustycad_tkbo::brep_algo_api::BRepAlgoAPIFuse;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::{History, uv_bounds};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{self, Axis2, Circle3, Plane3, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon,
    BRepBuilderAPIMakeShape,
};
use rustycad_tktopalgo::brep_gprop::BRepGProp;

fn CountShapes(shapes: &[Shape], kind: ShapeEnum) -> usize {
    let mut map = IndexedMapOfShape::default();
    for shape in shapes {
        topexp::map_shapes(shape, &mut map, true, true);
    }
    map.iter()
        .filter(|sub_shape| sub_shape.shape_type().unwrap() == kind)
        .count()
}

/// The expected removal, and the modified and generated counts of a shape.
struct Expected {
    deleted: bool,
    modified: usize,
    generated: usize,
}

fn ExpectHistory(history: &History, shape: &Shape, expected: &Expected, generated_kind: ShapeEnum) {
    assert_eq!(history.is_removed(shape).unwrap(), expected.deleted);
    let modified_kind = shape.shape_type().unwrap();
    assert_eq!(
        CountShapes(&history.modified(shape).unwrap(), modified_kind),
        expected.modified
    );
    assert_eq!(
        CountShapes(&history.generated(shape).unwrap(), generated_kind),
        expected.generated
    );
}

fn MakeCircleEdge(center: Point3, radius: f64) -> Shape {
    let frame = Axis2::new(center, gp::dz()).unwrap();
    let edge = BRepBuilderAPIMakeEdge::from_circ(Circle3::new(frame, radius).unwrap()).unwrap();
    edge.edge().unwrap().into_shape()
}

fn MakeRectangleFace(first_x: f64, last_x: f64, first_y: f64, last_y: f64) -> Shape {
    let plane = Plane3::new(Point3::new(0.0, 0.0, 0.0), gp::dz()).unwrap();
    let face = BRepBuilderAPIMakeFace::from_plane_bounds(plane, (first_x, last_x, first_y, last_y))
        .unwrap();
    face.face().unwrap().into_shape()
}

fn MakePolygonFace() -> Shape {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    let corners = [
        (-5.0, -5.0),
        (-5.0, 10.0),
        (5.0, 10.0),
        (5.0, -5.0),
        (2.0, -5.0),
        (2.0, 5.0),
        (-2.0, 5.0),
        (-2.0, -5.0),
    ];
    for (x, z) in corners {
        polygon.add_point(Point3::new(x, 0.0, z)).unwrap();
    }
    polygon.close().unwrap();
    let face = BRepBuilderAPIMakeFace::from_wire(&polygon.wire().unwrap(), false).unwrap();
    face.face().unwrap().into_shape()
}

fn MakeExpandedFace(face: &Face) -> Shape {
    let surface = BRepTool::surface(face).unwrap();
    let (first_u, last_u, first_v, last_v) = uv_bounds(face).unwrap();
    let bounds = (
        first_u - 100.0,
        last_u + 100.0,
        first_v - 100.0,
        last_v + 100.0,
    );
    let maker =
        BRepBuilderAPIMakeFace::from_surface_bounds(&surface, bounds, precision::confusion())
            .unwrap();
    maker.face().unwrap().into_shape()
}

/// The performed pave filler of the two shapes, without fuzzy value nor safe input.
fn performed_filler(first: &Shape, second: &Shape) -> BOPAlgoPaveFiller {
    let mut filler = BOPAlgoPaveFiller::new();
    filler.set_arguments(&[first.clone(), second.clone()].into_iter().collect());
    filler.options_mut().set_fuzzy_value(0.0);
    filler.options_mut().set_run_parallel(false);
    filler.set_non_destructive(false);
    filler.perform();
    assert!(!filler.options().has_errors());
    filler
}

/// The history of a Boolean operation or of the section on the filler arguments.
fn boolean_history(filler: &mut BOPAlgoPaveFiller, operation: Operation) -> Handle<History> {
    let arguments: Vec<Shape> = filler.arguments().iter().cloned().collect();
    let (first, last) = (&arguments[0], &arguments[arguments.len() - 1]);
    if operation == Operation::Section {
        let mut section = BOPAlgoSection::new();
        section.add_argument(first);
        section.add_argument(last);
        prepare(&mut section);
        section.perform_with_filler(filler);
        return performed_history(&mut section);
    }
    let mut bop = BOPAlgoBOP::new();
    bop.add_argument(first);
    bop.add_tool(last);
    bop.set_operation(operation);
    prepare(&mut bop);
    bop.perform_with_filler(filler);
    performed_history(&mut bop)
}

fn prepare(builder: &mut BOPAlgoBuilder) {
    builder.options_mut().set_run_parallel(false);
    builder.set_to_fill_history(true);
}

/// The history of a performed algorithm; its errors fail the test.
fn performed_history(builder: &mut BOPAlgoBuilder) -> Handle<History> {
    if builder.options().has_errors() {
        let mut errors = Vec::new();
        builder.options().dump_errors(&mut errors).unwrap();
        panic!("{}", String::from_utf8_lossy(&errors));
    }
    builder.history().unwrap()
}

fn generated_kind(shape: &Shape) -> ShapeEnum {
    if shape.shape_type().unwrap() == ShapeEnum::Face {
        ShapeEnum::Edge
    } else {
        ShapeEnum::Vertex
    }
}

fn ExpectBooleanHistory(
    first: &Shape,
    second: &Shape,
    operation: Operation,
    expected: [Expected; 2],
) {
    let mut filler = performed_filler(first, second);
    let history = boolean_history(&mut filler, operation);
    let history = history.get().unwrap();
    ExpectHistory(history, first, &expected[0], generated_kind(first));
    ExpectHistory(history, second, &expected[1], generated_kind(second));
}

fn e(deleted: bool, modified: usize, generated: usize) -> Expected {
    Expected {
        deleted,
        modified,
        generated,
    }
}

// Equivalent to tests/boolean/history/A1.
#[test]
fn BRepTools_HistoryTest__A1_IntersectingCircleEdges() {
    let edge1 = MakeCircleEdge(Point3::new(0.0, 0.0, 0.0), 10.0);
    let edge2 = MakeCircleEdge(Point3::new(10.0, 0.0, 0.0), 10.0);
    ExpectBooleanHistory(
        &edge1,
        &edge2,
        Operation::Common,
        [e(true, 0, 0), e(true, 0, 0)],
    );
    ExpectBooleanHistory(
        &edge1,
        &edge2,
        Operation::Fuse,
        [e(false, 3, 2), e(false, 3, 2)],
    );
    ExpectBooleanHistory(
        &edge1,
        &edge2,
        Operation::Cut,
        [e(false, 3, 2), e(true, 0, 2)],
    );
    ExpectBooleanHistory(
        &edge2,
        &edge1,
        Operation::Cut,
        [e(false, 3, 2), e(true, 0, 2)],
    );
    ExpectBooleanHistory(
        &edge1,
        &edge2,
        Operation::Section,
        [e(true, 0, 2), e(true, 0, 2)],
    );
}

// Equivalent to tests/boolean/history/A2.
#[test]
fn BRepTools_HistoryTest__A2_IntersectingPlanarFaces() {
    let face1 = MakeRectangleFace(-10.0, 10.0, -10.0, 10.0);
    let face2 = MakePolygonFace();
    ExpectBooleanHistory(
        &face1,
        &face2,
        Operation::Common,
        [e(true, 0, 0), e(true, 0, 0)],
    );
    ExpectBooleanHistory(
        &face1,
        &face2,
        Operation::Fuse,
        [e(false, 1, 2), e(false, 3, 2)],
    );
    ExpectBooleanHistory(
        &face1,
        &face2,
        Operation::Cut,
        [e(false, 1, 2), e(true, 0, 2)],
    );
    ExpectBooleanHistory(
        &face2,
        &face1,
        Operation::Cut,
        [e(false, 3, 2), e(true, 0, 2)],
    );
    ExpectBooleanHistory(
        &face1,
        &face2,
        Operation::Section,
        [e(true, 0, 2), e(true, 0, 2)],
    );
}

// Equivalent to tests/boolean/history/A3.
#[test]
fn BRepTools_HistoryTest__A3_OverlappingPlanarFaces() {
    let face1 = MakeRectangleFace(-10.0, 10.0, -10.0, 10.0);
    let face2 = MakeRectangleFace(0.0, 20.0, -10.0, 10.0);
    ExpectBooleanHistory(
        &face1,
        &face2,
        Operation::Common,
        [e(false, 1, 0), e(false, 1, 0)],
    );
    ExpectBooleanHistory(
        &face1,
        &face2,
        Operation::Fuse,
        [e(false, 2, 0), e(false, 2, 0)],
    );
    ExpectBooleanHistory(
        &face1,
        &face2,
        Operation::Cut,
        [e(false, 1, 0), e(true, 0, 0)],
    );
    ExpectBooleanHistory(
        &face2,
        &face1,
        Operation::Cut,
        [e(false, 1, 0), e(true, 0, 0)],
    );
    ExpectBooleanHistory(
        &face1,
        &face2,
        Operation::Section,
        [e(true, 0, 0), e(true, 0, 0)],
    );
}

// Equivalent to tests/boolean/history/A4.
#[test]
fn BRepTools_HistoryTest__A4_GeneralFuseEdgeAndFace() {
    let face = MakeRectangleFace(0.0, 10.0, 0.0, 10.0);
    let edge = BRepBuilderAPIMakeEdge::from_points(
        Point3::new(5.0, 5.0, -5.0),
        Point3::new(5.0, 5.0, 5.0),
    );
    let edge = edge.unwrap().edge().unwrap().into_shape();
    let mut builder = BOPAlgoBuilder::new();
    builder.add_argument(&face);
    builder.add_argument(&edge);
    builder.options_mut().set_run_parallel(false);
    builder.perform();
    assert!(!builder.options().has_errors());
    assert!(!builder.is_deleted(&face).unwrap());
    assert!(!builder.is_deleted(&edge).unwrap());
    assert_eq!(
        CountShapes(&builder.modified(&face).unwrap(), ShapeEnum::Face),
        1
    );
    assert_eq!(
        CountShapes(&builder.modified(&edge).unwrap(), ShapeEnum::Edge),
        2
    );
    assert_eq!(
        CountShapes(&builder.generated(&face).unwrap(), ShapeEnum::Vertex),
        1
    );
    assert_eq!(
        CountShapes(&builder.generated(&edge).unwrap(), ShapeEnum::Vertex),
        1
    );
}

/// The faces of the box after the first two.
fn side_faces(box_shape: &Shape) -> Vec<Shape> {
    let mut explorer = Explorer::new(box_shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let mut faces = Vec::new();
    while explorer.more() {
        faces.push(explorer.current().clone());
        explorer.next().unwrap();
    }
    faces.split_off(2)
}

// Equivalent to tests/boolean/history/A8.
#[test]
fn BRepTools_HistoryTest__A8_FuseAndSimplifyHistory() {
    let box1 = BRepPrimAPIMakeBox::from_sizes(10.0, 10.0, 10.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let box2 = BRepPrimAPIMakeBox::at_point(Point3::new(5.0, 0.0, 0.0), 10.0, 10.0, 10.0);
    let box2 = box2.unwrap().shape().unwrap().clone();
    let mut fuse = BRepAlgoAPIFuse::from_shapes(&box1, &box2).unwrap();
    fuse.set_run_parallel(false);
    fuse.build().unwrap();
    assert!(!fuse.has_errors());
    for face in side_faces(&box1) {
        assert_eq!(
            CountShapes(&fuse.modified(&face).unwrap(), ShapeEnum::Face),
            2
        );
    }

    fuse.simplify_result(true, true, precision::angular())
        .unwrap();
    for face in side_faces(&box1) {
        let modified = fuse.modified(&face).unwrap();
        assert_eq!(CountShapes(&modified, ShapeEnum::Face), 1);
        if let Some(first) = modified.first() {
            let mut properties = GProps::default();
            BRepGProp::surface_properties(first, &mut properties, false, false).unwrap();
            assert!((properties.mass() - 150.0).abs() <= precision::confusion());
        }
    }
}

// Equivalent to tests/boolean/history/A9.
#[test]
fn BRepTools_HistoryTest__A9_SectionExpandedBoxFacesHistory() {
    let box_shape = BRepPrimAPIMakeBox::from_sizes(10.0, 10.0, 10.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let mut explorer = Explorer::new(&box_shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let mut input_faces = Vec::new();
    while explorer.more() {
        input_faces.push(MakeExpandedFace(&topods::face(explorer.current()).unwrap()));
        explorer.next().unwrap();
    }
    let mut filler = BOPAlgoPaveFiller::new();
    filler.set_arguments(&input_faces.iter().cloned().collect::<List<Shape>>());
    filler.options_mut().set_fuzzy_value(0.0);
    filler.options_mut().set_run_parallel(false);
    filler.set_non_destructive(false);
    filler.perform();
    assert!(!filler.options().has_errors());

    let mut section = BOPAlgoSection::new();
    for argument in filler.arguments().iter() {
        section.add_argument(argument);
    }
    section.set_to_fill_history(true);
    section.options_mut().set_run_parallel(false);
    section.perform_with_filler(&mut filler);
    assert!(!section.options().has_errors());
    assert!(!section.shape().is_null());
    let history = section.history().unwrap();
    let history = history.get().unwrap();
    for face in &input_faces {
        assert!(history.is_removed(face).unwrap());
        assert_eq!(
            CountShapes(&history.modified(face).unwrap(), ShapeEnum::Face),
            0
        );
        assert_eq!(
            CountShapes(&history.generated(face).unwrap(), ShapeEnum::Edge),
            12
        );
    }
}
