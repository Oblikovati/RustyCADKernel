//! Direct translation of src/ModelingAlgorithms/TKBO/GTests/BOPAlgo_OpenSolid_Test.cxx (3d097a0).
#![allow(non_snake_case)]

#[path = "bop_test_utilities/mod.rs"]
mod bop_test_utilities;
use bop_test_utilities::{GetSurfaceArea, GetVolume};
use rustycad_tkbo::bop_algo::{
    BOPAlgoBOP, BOPAlgoBuilder, BOPAlgoCheckerSI, BOPAlgoPaveFiller, Operation,
};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Face, Shape, Shell, Solid};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape,
};

/// The expected topology and properties of a result (`ShapeExpectation`).
struct ShapeExpectation {
    vertices: usize,
    edges: usize,
    wires: usize,
    faces: usize,
    shells: usize,
    solids: usize,
    surface: f64,
    volume: f64,
}

fn expectation(counts: [usize; 6], surface: f64, volume: f64) -> ShapeExpectation {
    let [vertices, edges, wires, faces, shells, solids] = counts;
    ShapeExpectation {
        vertices,
        edges,
        wires,
        faces,
        shells,
        solids,
        surface,
        volume,
    }
}

/// `OpenResult`: an expectation without volume check.
fn OpenResult(counts: [usize; 6], surface: f64) -> ShapeExpectation {
    expectation(counts, surface, -1.0)
}

fn CountShapes(shape: &Shape, kind: ShapeEnum) -> usize {
    let mut map = IndexedMapOfShape::default();
    topexp::map_shapes(shape, &mut map, true, true);
    (1..=map.extent())
        .filter(|&index| map.find_key(index).unwrap().shape_type().unwrap() == kind)
        .count()
}

fn ExpectNoSelfInterference(shape: &Shape) {
    if shape.is_null() {
        return;
    }
    let mut checker = BOPAlgoCheckerSI::new();
    checker.set_arguments(&list(std::slice::from_ref(shape)));
    checker.set_level_of_check(9);
    checker
        .options_mut()
        .set_fuzzy_value(precision::confusion());
    checker.options_mut().set_run_parallel(false);
    checker.perform();
    assert!(!checker.options().has_errors());
}

fn expect_near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} differs from {expected} by more than {tolerance}"
    );
}

fn ExpectShape(shape: &Shape, expected: &ShapeExpectation) {
    // Open-solid results are intentionally not closed. DRAW checkshape reports
    // BRepCheck_NotClosed for these results, while bopcheck is the actual
    // self-interference validation used by the test.
    ExpectNoSelfInterference(shape);
    assert_eq!(CountShapes(shape, ShapeEnum::Vertex), expected.vertices);
    assert_eq!(CountShapes(shape, ShapeEnum::Edge), expected.edges);
    assert_eq!(CountShapes(shape, ShapeEnum::Wire), expected.wires);
    assert_eq!(CountShapes(shape, ShapeEnum::Face), expected.faces);
    assert_eq!(CountShapes(shape, ShapeEnum::Shell), expected.shells);
    assert_eq!(CountShapes(shape, ShapeEnum::Solid), expected.solids);
    expect_near(GetSurfaceArea(shape), expected.surface, 1.0e-6);
    if expected.volume >= 0.0 {
        expect_near(GetVolume(shape), expected.volume, 1.0e-6);
    }
}

fn list(shapes: &[Shape]) -> List<Shape> {
    shapes.iter().cloned().collect()
}

/// The faces `first..=last` (explorer order) of `solid` in one shell of a new solid, with
/// `extra` in a second shell when it is not null.
fn MakeOpenSolid(solid: &Shape, first: usize, last: usize, extra: &Face) -> Shape {
    let builder = BRepBuilder::default();
    let mut shell = Shell::default();
    builder.make_shell(&mut shell);
    let mut explorer = Explorer::new(solid, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let mut index = 0;
    while explorer.more() {
        if (first..=last).contains(&index) {
            builder.add(&mut shell, explorer.current()).unwrap();
        }
        index += 1;
        explorer.next().unwrap();
    }
    let mut result = Solid::default();
    builder.make_solid(&mut result);
    builder.add(&mut result, &shell).unwrap();
    if !extra.is_null() {
        let mut internal_shell = Shell::default();
        builder.make_shell(&mut internal_shell);
        builder.add(&mut internal_shell, extra).unwrap();
        builder.add(&mut result, &internal_shell).unwrap();
    }
    result.into_shape()
}

fn new_bop(objects: &List<Shape>, tools: &List<Shape>, operation: Operation) -> BOPAlgoBOP {
    let mut algorithm = BOPAlgoBOP::new();
    for object in objects.iter() {
        algorithm.add_argument(object);
    }
    for tool in tools.iter() {
        algorithm.add_tool(tool);
    }
    algorithm.set_operation(operation);
    algorithm.options_mut().set_run_parallel(false);
    algorithm
}

fn CheckBOP(
    objects: &List<Shape>,
    tools: &List<Shape>,
    operation: Operation,
    expected: &ShapeExpectation,
) {
    let mut algorithm = new_bop(objects, tools, operation);
    algorithm.perform();
    assert!(!algorithm.options().has_errors());
    ExpectShape(algorithm.shape(), expected);
}

/// The performed filler over `arguments`, with the open-solid test options.
fn performed_filler(arguments: &List<Shape>) -> BOPAlgoPaveFiller {
    let mut filler = BOPAlgoPaveFiller::new();
    filler.set_arguments(arguments);
    filler.options_mut().set_fuzzy_value(precision::confusion());
    filler.options_mut().set_run_parallel(false);
    filler.set_non_destructive(false);
    filler.perform();
    assert!(!filler.options().has_errors());
    filler
}

fn CheckBOPWithFiller(
    objects: &List<Shape>,
    tools: &List<Shape>,
    operation: Operation,
    expected: &ShapeExpectation,
) {
    let mut arguments = objects.clone();
    for tool in tools.iter() {
        arguments.append(tool.clone());
    }
    let mut filler = performed_filler(&arguments);
    let mut algorithm = new_bop(objects, tools, operation);
    algorithm.set_check_inverted(true);
    algorithm.perform_with_filler(&mut filler);
    assert!(!algorithm.options().has_errors());
    ExpectShape(algorithm.shape(), expected);
}

fn CheckBuildBOP(
    arguments: &List<Shape>,
    objects: &List<Shape>,
    tools: &List<Shape>,
    operation: Operation,
    expected: &ShapeExpectation,
    label: &str,
) {
    let mut filler = performed_filler(arguments);
    let mut builder = BOPAlgoBuilder::new();
    for argument in arguments.iter() {
        builder.add_argument(argument);
    }
    builder.options_mut().set_run_parallel(false);
    builder.set_check_inverted(true);
    builder.perform_with_filler(&mut filler);
    assert!(!builder.options().has_errors(), "{label}");
    builder
        .build_bop_operation_with_filler(&filler, objects, tools, operation, None)
        .unwrap();
    assert!(!builder.options().has_errors(), "{label}");
    ExpectShape(builder.shape(), expected);
}

fn CheckTwoShapeBOP(object: &Shape, tool: &Shape, expected: [ShapeExpectation; 4]) {
    let [common, fuse, cut, tuc] = expected;
    let (object, tool) = (
        list(std::slice::from_ref(object)),
        list(std::slice::from_ref(tool)),
    );
    CheckBOP(&object, &tool, Operation::Common, &common);
    CheckBOP(&object, &tool, Operation::Fuse, &fuse);
    CheckBOP(&object, &tool, Operation::Cut, &cut);
    CheckBOP(&tool, &object, Operation::Cut, &tuc);
}

fn make_box(origin: Point3, dx: f64, dy: f64, dz: f64) -> Shape {
    BRepPrimAPIMakeBox::at_point(origin, dx, dy, dz)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

fn MakeOpenBox(origin: Point3, dx: f64, dy: f64, dz: f64, first: usize, last: usize) -> Shape {
    MakeOpenSolid(&make_box(origin, dx, dy, dz), first, last, &Face::default())
}

fn MakeInternalFace(x: f64) -> Face {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for (y, z) in [(1.0, 1.0), (9.0, 1.0), (9.0, 9.0), (1.0, 9.0)] {
        polygon.add_point(Point3::new(x, y, z)).unwrap();
    }
    polygon.close().unwrap();
    let face = BRepBuilderAPIMakeFace::from_wire(&polygon.wire().unwrap(), false)
        .unwrap()
        .face()
        .unwrap();
    topods_face(face.into_shape().oriented(Orientation::Internal))
}

fn topods_face(shape: Shape) -> Face {
    rustycad_tkbrep::topods::face(&shape).unwrap()
}

fn origin(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

#[path = "bop_algo_open_solid_legacy/cases.rs"]
mod cases;
