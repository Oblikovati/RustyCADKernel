//! 1:1 translation of src/ModelingAlgorithms/TKPrim/GTests/BRepPrimAPI_MakeCone_Test.cxx.
#![allow(non_snake_case)]

use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::xoy;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeCone;
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
fn BRepPrimAPI_MakeConeTest__FullCone() {
    let mut a_cone = BRepPrimAPIMakeCone::with_axes(xoy(), 5.0, 0.0, 10.0).unwrap();
    let a_shape = a_cone.shape().unwrap().clone();
    assert!(a_cone.is_done(), "Full cone creation failed");
    assert!(!a_shape.is_null(), "Resulting shape is null");
    assert!(is_valid(&a_shape), "Full cone shape is not valid");
}

#[test]
fn BRepPrimAPI_MakeConeTest__TruncatedCone() {
    let mut a_cone = BRepPrimAPIMakeCone::with_axes(xoy(), 5.0, 2.0, 10.0).unwrap();
    let a_shape = a_cone.shape().unwrap().clone();
    assert!(a_cone.is_done(), "Truncated cone creation failed");
    assert!(!a_shape.is_null(), "Resulting shape is null");
    assert!(is_valid(&a_shape), "Truncated cone shape is not valid");
}

#[test]
fn BRepPrimAPI_MakeConeTest__FullConeFaceCount() {
    let mut a_cone = BRepPrimAPIMakeCone::with_axes(xoy(), 5.0, 0.0, 10.0).unwrap();
    let a_shape = a_cone.shape().unwrap().clone();
    assert!(a_cone.is_done());
    assert_eq!(
        face_count(&a_shape),
        2,
        "Full cone should have 2 faces (lateral + base)"
    );
}

#[test]
fn BRepPrimAPI_MakeConeTest__TruncatedConeFaceCount() {
    let mut a_cone = BRepPrimAPIMakeCone::with_axes(xoy(), 5.0, 2.0, 10.0).unwrap();
    let a_shape = a_cone.shape().unwrap().clone();
    assert!(a_cone.is_done());
    assert_eq!(
        face_count(&a_shape),
        3,
        "Truncated cone should have 3 faces (lateral + 2 bases)"
    );
}

#[test]
fn BRepPrimAPI_MakeConeTest__TruncatedConeVolume() {
    let (a_r1, a_r2, a_h) = (5.0, 2.0, 10.0);
    let mut a_cone = BRepPrimAPIMakeCone::with_axes(xoy(), a_r1, a_r2, a_h).unwrap();
    let a_shape = a_cone.shape().unwrap().clone();
    assert!(a_cone.is_done());
    let mut a_props = GProps::new();
    BRepGProp::volume_properties(&a_shape, &mut a_props, false, false, false).unwrap();
    // V = PI * H / 3 * (R1^2 + R1*R2 + R2^2)
    let an_expected_volume = PI * a_h / 3.0 * (a_r1 * a_r1 + a_r1 * a_r2 + a_r2 * a_r2);
    assert!((a_props.mass() - an_expected_volume).abs() <= precision::confusion());
}

#[test]
fn BRepPrimAPI_MakeConeTest__PartialCone() {
    let mut a_cone = BRepPrimAPIMakeCone::with_axes_angle(xoy(), 5.0, 2.0, 10.0, PI).unwrap();
    let a_shape = a_cone.shape().unwrap().clone();
    assert!(a_cone.is_done(), "Partial cone creation failed");
    assert!(!a_shape.is_null(), "Resulting shape is null");
    assert!(is_valid(&a_shape), "Partial cone shape is not valid");
    assert!(
        face_count(&a_shape) > 3,
        "Partial cone should have more than 3 faces"
    );
}
