//! Direct translation of src/ModelingAlgorithms/TKBO/GTests/BRepAlgoAPI_Simplify_Test.cxx (3d097a0).
#![allow(non_snake_case)]

use rustycad_tkbo::brep_algo_api::{BRepAlgoAPIFuse, BRepAlgoAPISection};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{self, Plane3, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPISewing,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

fn CountShapes(shape: &Shape, kind: ShapeEnum) -> usize {
    let mut map = IndexedMapOfShape::default();
    topexp::map_shapes(shape, &mut map, true, true);
    map.iter()
        .filter(|sub_shape| sub_shape.shape_type().unwrap() == kind)
        .count()
}

fn MakePlaneFace(plane: Plane3, bounds: (f64, f64, f64, f64)) -> Shape {
    BRepBuilderAPIMakeFace::from_plane_bounds(plane, bounds)
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

fn MakeThreeFaceShell() -> Shape {
    let plane = Plane3::new(Point3::new(0.0, 0.0, 0.0), gp::dz()).unwrap();
    let mut sewing = BRepBuilderAPISewing::default();
    for bounds in [
        (-10.0, 10.0, -10.0, 10.0),
        (10.0, 30.0, -10.0, 10.0),
        (30.0, 50.0, -10.0, 10.0),
    ] {
        sewing.add(&MakePlaneFace(plane, bounds)).unwrap();
    }
    sewing.perform(&ProgressRange::default()).unwrap();
    sewing.sewed_shape().clone()
}

fn GetLength(shape: &Shape) -> f64 {
    let mut properties = GProps::default();
    BRepGProp::linear_properties(shape, &mut properties, false, false).unwrap();
    properties.mass()
}

fn IsValid(shape: &Shape) -> bool {
    BRepCheckAnalyzer::new(shape, true, false, false)
        .unwrap()
        .is_valid()
        .unwrap()
}

fn one(shape: &Shape) -> List<Shape> {
    [shape.clone()].into_iter().collect()
}

fn expect_counts(shape: &Shape, kinds: &[(ShapeEnum, usize)]) {
    assert!(IsValid(shape));
    for (kind, count) in kinds {
        assert_eq!(CountShapes(shape, *kind), *count, "{kind:?}");
    }
}

fn box_at(corner: Point3, z: f64) -> Shape {
    BRepPrimAPIMakeBox::at_point(corner, 10.0, 10.0, z)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

// Equivalent to tests/boolean/simplify/A1.
#[test]
fn BRepAlgoAPI_SimplifyTest__A1_FuseCoplanarBoxes_SimplifiesFaces() {
    let box1 = box_at(Point3::new(0.0, 0.0, 0.0), 15.0);
    let box2 = box_at(Point3::new(3.0, 7.0, 0.0), 15.0);
    let mut fuse = BRepAlgoAPIFuse::new();
    fuse.set_arguments(&one(&box1));
    fuse.set_tools(&one(&box2));
    fuse.set_run_parallel(false);
    fuse.build().unwrap();
    assert!(!fuse.has_errors());
    assert!(!fuse.shape().is_null());
    let faces = |count| [(ShapeEnum::Wire, count), (ShapeEnum::Face, count)];
    expect_counts(
        fuse.shape(),
        &[(ShapeEnum::Vertex, 20), (ShapeEnum::Edge, 32)],
    );
    expect_counts(fuse.shape(), &faces(14));

    fuse.simplify_result(true, true, precision::angular())
        .unwrap();
    expect_counts(
        fuse.shape(),
        &[(ShapeEnum::Vertex, 16), (ShapeEnum::Edge, 24)],
    );
    expect_counts(fuse.shape(), &faces(10));
}

// Equivalent to tests/boolean/simplify/A2.
#[test]
fn BRepAlgoAPI_SimplifyTest__A2_SectionThroughSewnShell_SimplifiesEdges() {
    let shell = MakeThreeFaceShell();
    let cutting_plane = Plane3::new(Point3::new(0.0, 5.0, 0.0), gp::dy()).unwrap();
    let cutting_face = MakePlaneFace(cutting_plane, (-100.0, 100.0, -100.0, 100.0));
    let mut section = BRepAlgoAPISection::new();
    section.set_arguments(&one(&shell));
    section.set_tools(&one(&cutting_face));
    section.set_run_parallel(false);
    section.build().unwrap();
    assert!(!section.has_errors());
    assert!(!section.shape().is_null());
    expect_counts(
        section.shape(),
        &[(ShapeEnum::Vertex, 4), (ShapeEnum::Edge, 3)],
    );
    assert!((GetLength(section.shape()) - 60.0).abs() <= precision::confusion());

    section
        .simplify_result(true, true, precision::angular())
        .unwrap();
    expect_counts(
        section.shape(),
        &[(ShapeEnum::Vertex, 2), (ShapeEnum::Edge, 1)],
    );
    assert!((GetLength(section.shape()) - 60.0).abs() <= precision::confusion());
}
