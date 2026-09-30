//! 1:1 translation of src/ModelingAlgorithms/TKPrim/GTests/BRepPrimAPI_MakeSphere_Test.cxx.
#![allow(non_snake_case)]

use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeSphere;
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
fn BRepPrimAPI_MakeSphereTest__FullSphere() {
    let a_radius = 5.0;
    let mut a_sphere = BRepPrimAPIMakeSphere::new(a_radius).unwrap();
    let a_shape = a_sphere.shape().unwrap().clone();
    assert!(a_sphere.is_done(), "Full sphere creation failed");
    assert!(!a_shape.is_null(), "Resulting shape is null");
    assert!(is_valid(&a_shape), "Full sphere shape is not valid");
}

#[test]
fn BRepPrimAPI_MakeSphereTest__FaceCount() {
    let mut a_sphere = BRepPrimAPIMakeSphere::new(5.0).unwrap();
    let a_shape = a_sphere.shape().unwrap().clone();
    assert!(a_sphere.is_done());
    assert_eq!(face_count(&a_shape), 1, "Full sphere should have 1 face");
}

#[test]
fn BRepPrimAPI_MakeSphereTest__Volume() {
    let a_radius = 5.0;
    let mut a_sphere = BRepPrimAPIMakeSphere::new(a_radius).unwrap();
    let a_shape = a_sphere.shape().unwrap().clone();
    assert!(a_sphere.is_done());
    let mut a_props = GProps::new();
    BRepGProp::volume_properties(&a_shape, &mut a_props, false, false, false).unwrap();
    let an_expected_volume = (4.0 / 3.0) * PI * a_radius * a_radius * a_radius;
    assert!((a_props.mass() - an_expected_volume).abs() <= 0.01);
}

#[test]
fn BRepPrimAPI_MakeSphereTest__PartialSphere_AngleLimited() {
    let a_radius = 5.0;
    // Half sphere: angle from 0 to PI
    let mut a_sphere = BRepPrimAPIMakeSphere::with_angle(a_radius, PI).unwrap();
    let a_shape = a_sphere.shape().unwrap().clone();
    assert!(a_sphere.is_done(), "Partial sphere creation failed");
    assert!(!a_shape.is_null());
    assert!(is_valid(&a_shape), "Partial sphere shape is not valid");
    // Partial sphere should have more faces than a full sphere
    assert!(
        face_count(&a_shape) > 1,
        "Partial sphere should have more than 1 face"
    );
}

#[test]
fn BRepPrimAPI_MakeSphereTest__CenterOfMass() {
    let a_radius = 5.0;
    let mut a_sphere = BRepPrimAPIMakeSphere::new(a_radius).unwrap();
    let a_shape = a_sphere.shape().unwrap().clone();
    assert!(a_sphere.is_done());
    let mut a_props = GProps::new();
    BRepGProp::volume_properties(&a_shape, &mut a_props, false, false, false).unwrap();
    let a_com = a_props.centre_of_mass();
    // Center of mass of full sphere should be at origin
    assert!(a_com.x().abs() <= precision::confusion());
    assert!(a_com.y().abs() <= precision::confusion());
    assert!(a_com.z().abs() <= precision::confusion());
}
