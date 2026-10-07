//! Direct translation of src/ModelingAlgorithms/TKBO/GTests/BRepAlgoAPI_Splitter_Test.cxx (3d097a0).
#![allow(non_snake_case)]

#[path = "bop_test_utilities/mod.rs"]
mod bop_test_utilities;
use bop_test_utilities::{GetSurfaceArea, GetVolume};
use rustycad_tkbo::bop_algo::BOPAlgoSplitter;
use rustycad_tkbo::brep_algo_api::BRepAlgoAPISplitter;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Compound, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCurve, GeomCylindricalSurface, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{self, Axis2, Axis3, Plane3, Point3};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeSphere,
};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use std::f64::consts::PI;

fn CountShapes(shape: &Shape, kind: ShapeEnum) -> usize {
    let mut map = IndexedMapOfShape::default();
    topexp::map_shapes(shape, &mut map, true, true);
    (1..=map.extent())
        .filter(|&index| map.find_key(index).unwrap().shape_type().unwrap() == kind)
        .count()
}

fn ExpectShapeCount(shape: &Shape, kind: ShapeEnum, expected: usize) {
    if !shape.is_null() {
        assert!(
            BRepCheckAnalyzer::new(shape, true, false, false)
                .unwrap()
                .is_valid()
                .unwrap()
        );
    }
    assert_eq!(CountShapes(shape, kind), expected);
}

fn expect_near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} differs from {expected} by more than {tolerance}"
    );
}

fn MakePlaneFace(first_u: f64, last_u: f64, first_v: f64, last_v: f64, z: f64) -> Shape {
    let plane = Plane3::new(Point3::new(0.0, 0.0, z), gp::dz()).unwrap();
    let maker =
        BRepBuilderAPIMakeFace::from_plane_bounds(plane, (first_u, last_u, first_v, last_v))
            .unwrap();
    maker.face().unwrap().into_shape()
}

fn cylinder_surface() -> Handle<dyn GeomSurface> {
    Handle::new(GeomCylindricalSurface::new(Axis3::from_axis2(gp::xoy()), 10.0).unwrap())
        .into_cast()
}

fn MakeCylinderFace() -> Shape {
    // The native literal 3.14159265358979323846 rounds to f64 PI.
    let bounds = (0.0, 2.0 * PI, -20.0, 20.0);
    let maker = BRepBuilderAPIMakeFace::from_surface_bounds(
        &cylinder_surface(),
        bounds,
        precision::confusion(),
    )
    .unwrap();
    maker.face().unwrap().into_shape()
}

fn edge_of(curve: &Handle<dyn GeomCurve>, range: Option<(f64, f64)>) -> Shape {
    let maker = match range {
        Some((first, last)) => BRepBuilderAPIMakeEdge::from_curve_params(curve, first, last),
        None => BRepBuilderAPIMakeEdge::from_curve(curve),
    };
    maker.unwrap().edge().unwrap().into_shape()
}

fn MakeCylinderIsoEdges() -> Shape {
    let builder = BRepBuilder::default();
    let mut edges = Compound::default();
    builder.make_compound(&mut edges);
    let surface = cylinder_surface();
    // The native literal 3.14159265358979323846 rounds to f64 PI.
    let pi = PI;
    let nb_u_edges = 10;
    for index in 0..=nb_u_edges {
        let angle = 2.0 * pi * f64::from(index) / f64::from(nb_u_edges);
        let curve = surface.get().unwrap().u_iso(angle).unwrap();
        builder
            .add(&mut edges, &edge_of(&curve, Some((-25.0, 25.0))))
            .unwrap();
    }
    let nb_v_edges = 10;
    for index in 0..=nb_v_edges {
        let z = -20.0 + 40.0 * f64::from(index) / f64::from(nb_v_edges);
        let curve = surface.get().unwrap().v_iso(z).unwrap();
        builder.add(&mut edges, &edge_of(&curve, None)).unwrap();
    }
    edges.into_shape()
}

fn MakeCylinderGrid() -> Shape {
    let builder = BRepBuilder::default();
    let mut cylinders = Compound::default();
    builder.make_compound(&mut cylinders);
    for x in (0..=100).step_by(5) {
        for y in (0..=100).step_by(5) {
            let axis = Axis2::new(Point3::new(f64::from(x), f64::from(y), -5.0), gp::dz()).unwrap();
            let cylinder = BRepPrimAPIMakeCylinder::with_axes(axis, 1.0, 11.0)
                .shape()
                .unwrap()
                .clone();
            builder.add(&mut cylinders, &cylinder).unwrap();
        }
    }
    cylinders.into_shape()
}

fn split_bop_algo(object: &Shape, tools: &[Shape]) -> BOPAlgoSplitter {
    let mut splitter = BOPAlgoSplitter::new();
    splitter.add_argument(object);
    for tool in tools {
        splitter.add_tool(tool);
    }
    splitter.options_mut().set_run_parallel(false);
    splitter.perform();
    splitter
}

fn split_brep_algo_api(object: &Shape, tools: &[Shape]) -> BRepAlgoAPISplitter {
    let mut splitter = BRepAlgoAPISplitter::new();
    splitter.set_arguments(&[object.clone()].into_iter().collect::<List<Shape>>());
    splitter.set_tools(&tools.iter().cloned().collect::<List<Shape>>());
    splitter.set_run_parallel(false);
    splitter.build().unwrap();
    splitter
}

fn SplitWithBOPAlgo(object: &Shape, tool: &Shape) -> Shape {
    let splitter = split_bop_algo(object, std::slice::from_ref(tool));
    assert!(!splitter.options().has_errors());
    splitter.shape().clone()
}

fn SplitWithBRepAlgoAPI(object: &Shape, tool: &Shape) -> Shape {
    let splitter = split_brep_algo_api(object, std::slice::from_ref(tool));
    assert!(!splitter.has_errors());
    splitter.shape().clone()
}

/// The direct children of `tools` (the `TopoDS_Iterator` loop of the native helpers).
fn children(tools: &Shape) -> Vec<Shape> {
    let mut iterator = ShapeIterator::new(tools, true, true);
    let mut found = Vec::new();
    while let Ok(child) = iterator.value() {
        found.push(child.clone());
        iterator.next();
    }
    found
}

fn SplitCylinderFaceWithBOPAlgo(object: &Shape, tools: &Shape) -> Shape {
    let splitter = split_bop_algo(object, &children(tools));
    assert!(!splitter.options().has_errors());
    splitter.shape().clone()
}

fn SplitCylinderFaceWithBRepAlgoAPI(object: &Shape, tools: &Shape) -> Shape {
    let splitter = split_brep_algo_api(object, &children(tools));
    assert!(!splitter.has_errors());
    splitter.shape().clone()
}

fn make_box(corner: Point3, dx: f64, dy: f64, dz: f64) -> Shape {
    BRepPrimAPIMakeBox::at_point(corner, dx, dy, dz)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

fn sphere5() -> Shape {
    BRepPrimAPIMakeSphere::new(5.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

fn crossing_boxes() -> [Shape; 3] {
    [
        make_box(Point3::new(0.0, 0.0, 0.0), 10.0, 10.0, 10.0),
        make_box(Point3::new(4.0, -2.0, -2.0), 2.0, 14.0, 14.0),
        make_box(Point3::new(-2.0, 4.0, -2.0), 14.0, 2.0, 14.0),
    ]
}

fn box_solid_and_face() -> [Shape; 3] {
    [
        make_box(Point3::new(0.0, 0.0, 0.0), 10.0, 10.0, 10.0),
        make_box(Point3::new(5.0, 0.0, 0.0), 10.0, 10.0, 10.0),
        MakePlaneFace(-20.0, 20.0, -20.0, 20.0, 5.0),
    ]
}

// Equivalent to tests/boolean/splitter/A1.
#[test]
fn BRepAlgoAPI_SplitterTest__A1_BoxSplitByCrossingBoxes_ProducesNineSolids() {
    let [box_shape, tool_x, tool_y] = crossing_boxes();
    let splitter = split_bop_algo(&box_shape, &[tool_x, tool_y]);
    assert!(!splitter.options().has_errors());
    let result = splitter.shape();
    assert!(!result.is_null());
    ExpectShapeCount(result, ShapeEnum::Solid, 9);
    expect_near(GetVolume(result), 1000.0, precision::confusion());
}

// Equivalent to tests/boolean/splitter/A2.
#[test]
fn BRepAlgoAPI_SplitterTest__A2_BoxSplitBySolidAndFace_ProducesFourSolids() {
    let [box_shape, tool_box, tool_face] = box_solid_and_face();
    let splitter = split_bop_algo(&box_shape, &[tool_box, tool_face]);
    assert!(!splitter.options().has_errors());
    let result = splitter.shape();
    assert!(!result.is_null());
    ExpectShapeCount(result, ShapeEnum::Solid, 4);
    expect_near(GetVolume(result), 1000.0, precision::confusion());
}

// Equivalent to tests/boolean/splitter/A3.
#[test]
fn BRepAlgoAPI_SplitterTest__A3_PlaneSplitBySphere_ProducesTwoFaces() {
    let result = SplitWithBOPAlgo(&MakePlaneFace(-20.0, 20.0, -20.0, 20.0, 0.0), &sphere5());
    assert!(!result.is_null());
    ExpectShapeCount(&result, ShapeEnum::Face, 2);
    expect_near(GetSurfaceArea(&result), 1600.0, precision::confusion());
}

// Equivalent to tests/boolean/splitter/B1.
#[test]
fn BRepAlgoAPI_SplitterTest__B1_ApiBoxSplitByCrossingBoxes_ProducesNineSolids() {
    let [box_shape, tool_x, tool_y] = crossing_boxes();
    let splitter = split_brep_algo_api(&box_shape, &[tool_x, tool_y]);
    assert!(!splitter.has_errors());
    let result = splitter.shape();
    assert!(!result.is_null());
    ExpectShapeCount(result, ShapeEnum::Solid, 9);
    expect_near(GetVolume(result), 1000.0, precision::confusion());
}

// Equivalent to tests/boolean/splitter/B2.
#[test]
fn BRepAlgoAPI_SplitterTest__B2_ApiBoxSplitBySolidAndFace_ProducesFourSolids() {
    let [box_shape, tool_box, tool_face] = box_solid_and_face();
    let splitter = split_brep_algo_api(&box_shape, &[tool_box, tool_face]);
    assert!(!splitter.has_errors());
    let result = splitter.shape();
    assert!(!result.is_null());
    ExpectShapeCount(result, ShapeEnum::Solid, 4);
    expect_near(GetVolume(result), 1000.0, precision::confusion());
}

// Equivalent to tests/boolean/splitter/B3.
#[test]
fn BRepAlgoAPI_SplitterTest__B3_ApiPlaneSplitBySphere_ProducesTwoFaces() {
    let result = SplitWithBRepAlgoAPI(&MakePlaneFace(-20.0, 20.0, -20.0, 20.0, 0.0), &sphere5());
    assert!(!result.is_null());
    ExpectShapeCount(&result, ShapeEnum::Face, 2);
    expect_near(GetSurfaceArea(&result), 1600.0, precision::confusion());
}

// Equivalent to tests/boolean/splitter/A4.
#[test]
fn BRepAlgoAPI_SplitterTest__A4_CylinderFaceSplitByIsoEdges_ProducesHundredFaces() {
    let result = SplitCylinderFaceWithBOPAlgo(&MakeCylinderFace(), &MakeCylinderIsoEdges());
    assert!(!result.is_null());
    ExpectShapeCount(&result, ShapeEnum::Face, 100);
    expect_near(GetSurfaceArea(&result), 2513.27, 0.01);
}

// Equivalent to tests/boolean/splitter/B4.
#[test]
fn BRepAlgoAPI_SplitterTest__B4_ApiCylinderFaceSplitByIsoEdges_ProducesHundredFaces() {
    let result = SplitCylinderFaceWithBRepAlgoAPI(&MakeCylinderFace(), &MakeCylinderIsoEdges());
    assert!(!result.is_null());
    ExpectShapeCount(&result, ShapeEnum::Face, 100);
    expect_near(GetSurfaceArea(&result), 2513.27, 0.01);
}

// Equivalent to tests/boolean/splitter/A6.
#[test]
fn BRepAlgoAPI_SplitterTest__A6_PlateSplitByCylinderGrid_ProducesFourHundredFortyTwoSolids() {
    let plate = make_box(Point3::new(0.0, 0.0, 0.0), 100.0, 100.0, 1.0);
    let result = SplitWithBOPAlgo(&plate, &MakeCylinderGrid());
    assert!(!result.is_null());
    ExpectShapeCount(&result, ShapeEnum::Solid, 442);
    expect_near(GetVolume(&result), 10000.0, precision::confusion());
}

// Equivalent to tests/boolean/splitter/B6.
#[test]
fn BRepAlgoAPI_SplitterTest__B6_ApiPlateSplitByCylinderGrid_ProducesFourHundredFortyTwoSolids() {
    let plate = make_box(Point3::new(0.0, 0.0, 0.0), 100.0, 100.0, 1.0);
    let result = SplitWithBRepAlgoAPI(&plate, &MakeCylinderGrid());
    assert!(!result.is_null());
    ExpectShapeCount(&result, ShapeEnum::Solid, 442);
    expect_near(GetVolume(&result), 10000.0, precision::confusion());
}
