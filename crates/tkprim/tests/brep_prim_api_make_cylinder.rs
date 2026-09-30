//! 1:1 translation of src/ModelingAlgorithms/TKPrim/GTests/BRepPrimAPI_MakeCylinder_Test.cxx.
#![allow(non_snake_case)]

use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeCylinder;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use std::f64::consts::PI;

/// The face-counting `TopExp_Explorer` loops of the suite.
fn face_count(shape: &Shape) -> i32 {
    let mut count = 0;
    let mut explorer = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
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
fn BRepPrimAPI_MakeCylinderTest__FullCylinder() {
    let (a_radius, a_height) = (5.0, 10.0);
    let mut a_cylinder = BRepPrimAPIMakeCylinder::new(a_radius, a_height);
    let a_shape = a_cylinder.shape().unwrap().clone();
    assert!(a_cylinder.is_done(), "Full cylinder creation failed");
    assert!(!a_shape.is_null(), "Resulting shape is null");
    assert!(is_valid(&a_shape), "Full cylinder shape is not valid");
}

#[test]
fn BRepPrimAPI_MakeCylinderTest__FaceCount() {
    let mut a_cylinder = BRepPrimAPIMakeCylinder::new(5.0, 10.0);
    let a_shape = a_cylinder.shape().unwrap().clone();
    assert!(a_cylinder.is_done());
    // Full cylinder: 1 lateral + 2 caps = 3
    assert_eq!(
        face_count(&a_shape),
        3,
        "Full cylinder should have 3 faces (lateral + 2 caps)"
    );
}

#[test]
fn BRepPrimAPI_MakeCylinderTest__Volume() {
    let (a_radius, a_height) = (5.0, 10.0);
    let mut a_cylinder = BRepPrimAPIMakeCylinder::new(a_radius, a_height);
    let a_shape = a_cylinder.shape().unwrap().clone();
    assert!(a_cylinder.is_done());
    let mut a_props = GProps::new();
    BRepGProp::volume_properties(&a_shape, &mut a_props, false, false, false).unwrap();
    let an_expected_volume = PI * a_radius * a_radius * a_height;
    assert!((a_props.mass() - an_expected_volume).abs() <= 0.01);
}

#[test]
fn BRepPrimAPI_MakeCylinderTest__PartialCylinder_AngleLimited() {
    let (a_radius, a_height) = (5.0, 10.0);
    // Half cylinder: angle = PI
    let mut a_cylinder = BRepPrimAPIMakeCylinder::with_angle(a_radius, a_height, PI).unwrap();
    let a_shape = a_cylinder.shape().unwrap().clone();
    assert!(a_cylinder.is_done(), "Partial cylinder creation failed");
    assert!(!a_shape.is_null());
    assert!(is_valid(&a_shape), "Partial cylinder shape is not valid");
    // Partial cylinder should have more faces: 1 lateral + 2 caps + 2 planar sides = 5
    assert_eq!(face_count(&a_shape), 5, "Half cylinder should have 5 faces");
}

#[test]
fn BRepPrimAPI_MakeCylinderTest__ShapeValidity() {
    let mut a_cylinder = BRepPrimAPIMakeCylinder::new(100.0, 200.0);
    let a_shape = a_cylinder.shape().unwrap().clone();
    assert!(a_cylinder.is_done());
    assert!(is_valid(&a_shape), "Large cylinder shape is not valid");
}
