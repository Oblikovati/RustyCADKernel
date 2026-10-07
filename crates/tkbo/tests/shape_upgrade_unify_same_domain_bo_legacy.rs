//! Direct translation of the tests of
//! src/ModelingAlgorithms/TKShHealing/GTests/ShapeUpgrade_UnifySameDomain_Test.cxx (3d097a0)
//! that build their input with TKBO (BRepAlgoAPI_Common, BRepAlgoAPI_Fuse, BOPAlgo_Splitter).
//! They run here because TKBO depends on TKShHealing: a TKShHealing test target linking TKBO
//! would build TKShHealing twice.
#![allow(non_snake_case)]

use rustycad_tkbo::bop_algo::BOPAlgoSplitter;
use rustycad_tkbo::brep_algo_api::{BRepAlgoAPICommon, BRepAlgoAPIFuse};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};

use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve, GeomLine};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Vector3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakePrism};
use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeUnifySameDomain;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape,
    BRepBuilderAPIMakeVertex, BRepBuilderAPIMakeWire,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

fn explore(shape: &Shape, kind: ShapeEnum) -> Vec<Shape> {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut found = Vec::new();
    while explorer.more() {
        found.push(explorer.current().clone());
        explorer.next().unwrap();
    }
    found
}

/// The distinct sub-shapes of a type, located shapes counted once as DRAW's nbshapes does.
fn countSubShapes(shape: &Shape, kind: ShapeEnum) -> usize {
    let mut map = IndexedMapOfShape::default();
    for mut sub_shape in explore(shape, kind) {
        sub_shape.set_location(&Location::default(), false).unwrap();
        map.add(sub_shape);
    }
    map.extent()
}

fn IsValid(shape: &Shape) -> bool {
    BRepCheckAnalyzer::new(shape, true, false, false)
        .unwrap()
        .is_valid()
        .unwrap()
}

fn unified(shape: &Shape, edges: bool, faces: bool, concat_bsplines: bool) -> Shape {
    let unifier = ShapeUpgradeUnifySameDomain::new(shape, edges, faces, concat_bsplines);
    unifier.build().unwrap();
    unifier.shape()
}

fn bezier_prism() -> Shape {
    let mut poles = Array1::with_bounds(1, 5).unwrap();
    let points = [(0.0, 0.0), (1.0, 0.0), (2.0, 2.0), (0.0, 0.5), (0.0, 0.0)];
    for (index, (x, y)) in points.into_iter().enumerate() {
        poles
            .set_value(index as i32 + 1, Point3::new(x, y, 0.0))
            .unwrap();
    }
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomBezierCurve::new(&poles).unwrap()).into_cast();
    let edge = BRepBuilderAPIMakeEdge::from_curve(&curve)
        .unwrap()
        .edge()
        .unwrap();
    let wire = BRepBuilderAPIMakeWire::from_edge(&edge)
        .unwrap()
        .wire()
        .unwrap();
    let face = BRepBuilderAPIMakeFace::from_wire(&wire, false)
        .unwrap()
        .face()
        .unwrap();
    let mut prism =
        BRepPrimAPIMakePrism::new(&face.into_shape(), Vector3::new(0.0, 0.0, 1.0), false, true)
            .unwrap();
    prism.shape().unwrap().clone()
}

/// The largest vertex, edge and face tolerance of the shape.
fn max_tolerance(shape: &Shape) -> f64 {
    let mut maximum: f64 = 0.0;
    for vertex in explore(shape, ShapeEnum::Vertex) {
        maximum =
            maximum.max(BRepTool::vertex_tolerance(&topods::vertex(&vertex).unwrap()).unwrap());
    }
    for edge in explore(shape, ShapeEnum::Edge) {
        maximum = maximum.max(BRepTool::edge_tolerance(&topods::edge(&edge).unwrap()).unwrap());
    }
    for face in explore(shape, ShapeEnum::Face) {
        maximum = maximum.max(BRepTool::face_tolerance(&topods::face(&face).unwrap()).unwrap());
    }
    maximum
}

// Migrated from tests/bugs/heal/bug29382_3.
#[test]
fn ShapeUpgrade_UnifySameDomainTest__OCC29382_CommonShapeHasStableTopology() {
    let prism = bezier_prism();
    let mut box_builder =
        BRepPrimAPIMakeBox::at_point(Point3::new(-0.3, -0.2, 0.0), 1.0, 0.4, 1.0).unwrap();
    let box_shape = box_builder.shape().unwrap().clone();
    let mut common = BRepAlgoAPICommon::from_shapes(&prism, &box_shape).unwrap();
    common.build().unwrap();
    assert!(common.is_done());
    assert!(!common.shape().is_null());

    let result = unified(common.shape(), true, true, true);
    assert!(!result.is_null());
    assert!(IsValid(&result));
    let counts = [
        (ShapeEnum::Solid, 1),
        (ShapeEnum::Shell, 1),
        (ShapeEnum::Face, 5),
        (ShapeEnum::Wire, 5),
        (ShapeEnum::Edge, 9),
        (ShapeEnum::Vertex, 5),
    ];
    for (kind, count) in counts {
        assert_eq!(countSubShapes(&result, kind), count, "{kind:?}");
    }
    assert!(max_tolerance(&result) <= 6.0e-6);
}

// Migrated from tests/bugs/heal/bug30927.
#[test]
fn ShapeUpgrade_UnifySameDomainTest__OCC30927_FusedSolidKeepsClosedWires() {
    let box1 = BRepPrimAPIMakeBox::from_sizes(10.0, 10.0, 10.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let mut box2 =
        BRepPrimAPIMakeBox::at_point(Point3::new(5.0, 0.0, -5.0), 10.0, 10.0, 20.0).unwrap();
    let box2 = box2.shape().unwrap().clone();
    let mut fuse = BRepAlgoAPIFuse::from_shapes(&box1, &box2).unwrap();
    fuse.build().unwrap();
    assert!(fuse.is_done());
    assert!(!fuse.shape().is_null());

    let result = unified(fuse.shape(), true, true, false);
    assert!(!result.is_null());
    let wires = explore(&result, ShapeEnum::Wire);
    for wire in &wires {
        assert!(wire.closed().unwrap());
    }
    assert!(!wires.is_empty());
}

// Migrated from tests/bugs/heal/bug29544_2.
#[test]
fn ShapeUpgrade_UnifySameDomainTest__OCC29544_NearbyLinearSplitIsUnified() {
    let axis = Axis1::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(axis)).into_cast();
    let reference_edge = BRepBuilderAPIMakeEdge::from_curve_params(&line, 0.0, 10.0)
        .unwrap()
        .edge()
        .unwrap();
    let split_vertex = BRepBuilderAPIMakeVertex::new(Point3::new(5.0, 2.0e-7, 0.0))
        .unwrap()
        .vertex()
        .unwrap();
    assert!(!split_vertex.is_null());

    let mut splitter = BOPAlgoSplitter::new();
    splitter.add_argument(&reference_edge);
    splitter.add_tool(&split_vertex);
    splitter.options_mut().set_run_parallel(false);
    splitter.perform();
    assert!(!splitter.options().has_errors());
    let split_shape = splitter.shape().clone();
    assert!(!split_shape.is_null());

    let result = unified(&split_shape, true, true, false);
    assert!(!result.is_null());
    assert!(IsValid(&result));
    assert_eq!(countSubShapes(&result, ShapeEnum::Vertex), 2);
    assert_eq!(countSubShapes(&result, ShapeEnum::Edge), 1);
    let mut properties = GProps::default();
    BRepGProp::linear_properties(&result, &mut properties, false, false).unwrap();
    assert!((properties.mass() - 10.0).abs() <= precision::confusion());
}
