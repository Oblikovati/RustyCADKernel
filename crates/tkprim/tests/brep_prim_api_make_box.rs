//! 1:1 translation of src/ModelingAlgorithms/TKPrim/GTests/BRepPrimAPI_MakeBox_Test.cxx.
#![allow(non_snake_case)]

use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

/// The `for (TopExp_Explorer anExp(aShape, kind); anExp.More(); anExp.Next()) count++`
/// loops of the suite.
fn explored_count(shape: &Shape, kind: ShapeEnum) -> i32 {
    let mut count = 0;
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    while explorer.more() {
        count += 1;
        explorer.next().unwrap();
    }
    count
}

/// `BRepCheck_Analyzer(aShape).IsValid()`.
fn is_valid(shape: &Shape) -> bool {
    BRepCheckAnalyzer::new(shape, true, false, false)
        .unwrap()
        .is_valid()
        .unwrap()
}

#[test]
fn BRepPrimAPI_MakeBoxTest__UnitBox() {
    let mut a_box = BRepPrimAPIMakeBox::from_sizes(1.0, 1.0, 1.0).unwrap();
    let a_shape = a_box.shape().unwrap().clone();
    assert!(a_box.is_done(), "Unit box creation failed");
    assert!(!a_shape.is_null(), "Resulting shape is null");
    assert!(is_valid(&a_shape), "Unit box shape is not valid");
}

#[test]
fn BRepPrimAPI_MakeBoxTest__TopologyCounts() {
    let mut a_box = BRepPrimAPIMakeBox::from_sizes(10.0, 20.0, 30.0).unwrap();
    let a_shape = a_box.shape().unwrap().clone();
    assert!(a_box.is_done());
    assert_eq!(
        explored_count(&a_shape, ShapeEnum::Face),
        6,
        "Box should have 6 faces"
    );
    assert_eq!(
        explored_count(&a_shape, ShapeEnum::Edge),
        24,
        "Box should have 24 edge occurrences (12 edges x 2 faces each)"
    );
    assert_eq!(
        explored_count(&a_shape, ShapeEnum::Vertex),
        48,
        "Box should have 48 vertex occurrences (8 vertices x 6 faces)"
    );
}

#[test]
fn BRepPrimAPI_MakeBoxTest__Volume() {
    let (a_width, a_height, a_depth) = (3.0, 4.0, 5.0);
    let mut a_box = BRepPrimAPIMakeBox::from_sizes(a_width, a_height, a_depth).unwrap();
    let a_shape = a_box.shape().unwrap().clone();
    assert!(a_box.is_done());
    let mut a_props = GProps::new();
    BRepGProp::volume_properties(&a_shape, &mut a_props, false, false, false).unwrap();
    assert!((a_props.mass() - a_width * a_height * a_depth).abs() <= precision::confusion());
}

#[test]
fn BRepPrimAPI_MakeBoxTest__SurfaceArea() {
    let (a_w, a_h, a_d) = (3.0, 4.0, 5.0);
    let mut a_box = BRepPrimAPIMakeBox::from_sizes(a_w, a_h, a_d).unwrap();
    let a_shape = a_box.shape().unwrap().clone();
    assert!(a_box.is_done());
    let mut a_props = GProps::new();
    BRepGProp::surface_properties(&a_shape, &mut a_props, false, false).unwrap();
    let an_expected_area = 2.0 * (a_w * a_h + a_w * a_d + a_h * a_d);
    assert!((a_props.mass() - an_expected_area).abs() <= precision::confusion());
}

#[test]
fn BRepPrimAPI_MakeBoxTest__TwoCornerPoints() {
    let a_p1 = Point3::new(1.0, 2.0, 3.0);
    let a_p2 = Point3::new(4.0, 6.0, 8.0);
    let mut a_box = BRepPrimAPIMakeBox::from_points(a_p1, a_p2).unwrap();
    let a_shape = a_box.shape().unwrap().clone();
    assert!(a_box.is_done(), "Box from two corner points failed");
    assert!(!a_shape.is_null());
    assert!(is_valid(&a_shape), "Box from two points is not valid");
    // Volume = (4-1)*(6-2)*(8-3) = 3*4*5 = 60
    let mut a_props = GProps::new();
    BRepGProp::volume_properties(&a_shape, &mut a_props, false, false, false).unwrap();
    assert!((a_props.mass() - 60.0).abs() <= precision::confusion());
}

#[test]
fn BRepPrimAPI_MakeBoxTest__ShapeValidity() {
    let mut a_box = BRepPrimAPIMakeBox::from_sizes(100.0, 200.0, 300.0).unwrap();
    let a_shape = a_box.shape().unwrap().clone();
    assert!(a_box.is_done());
    assert!(is_valid(&a_shape), "Large box shape is not valid");
}
