//! 1:1 translation of src/ModelingAlgorithms/TKPrim/GTests/BRepPrimAPI_MakeWedge_Test.cxx.
#![allow(non_snake_case)]

use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeWedge;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

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
fn BRepPrimAPI_MakeWedgeTest__WedgeFromDimensions() {
    let mut a_wedge = BRepPrimAPIMakeWedge::with_ltx(10.0, 10.0, 10.0, 5.0).unwrap();
    let a_shape = a_wedge.shape().unwrap().clone();
    assert!(a_wedge.is_done(), "Wedge creation failed");
    assert!(!a_shape.is_null(), "Resulting shape is null");
    assert!(is_valid(&a_shape), "Wedge shape is not valid");
}

#[test]
fn BRepPrimAPI_MakeWedgeTest__WedgeFaceCount_Ltx5() {
    let mut a_wedge = BRepPrimAPIMakeWedge::with_ltx(10.0, 10.0, 10.0, 5.0).unwrap();
    let a_shape = a_wedge.shape().unwrap().clone();
    assert!(a_wedge.is_done());
    assert_eq!(
        face_count(&a_shape),
        6,
        "Wedge with ltx=5 should have 6 faces"
    );
}

#[test]
fn BRepPrimAPI_MakeWedgeTest__WedgeFaceCount_Ltx0() {
    let mut a_wedge = BRepPrimAPIMakeWedge::with_ltx(10.0, 10.0, 10.0, 0.0).unwrap();
    let a_shape = a_wedge.shape().unwrap().clone();
    assert!(a_wedge.is_done());
    assert_eq!(
        face_count(&a_shape),
        5,
        "Wedge with ltx=0 should have 5 faces (top degenerates to a line)"
    );
}

#[test]
fn BRepPrimAPI_MakeWedgeTest__WedgeVolume() {
    let (a_dx, a_dy, a_dz, a_ltx) = (10.0, 10.0, 10.0, 5.0);
    let mut a_wedge = BRepPrimAPIMakeWedge::with_ltx(a_dx, a_dy, a_dz, a_ltx).unwrap();
    let a_shape = a_wedge.shape().unwrap().clone();
    assert!(a_wedge.is_done());
    let mut a_props = GProps::new();
    BRepGProp::volume_properties(&a_shape, &mut a_props, false, false, false).unwrap();
    // Volume = dy * dz * (dx + ltx) / 2
    let an_expected_volume = a_dy * a_dz * (a_dx + a_ltx) / 2.0;
    assert!((a_props.mass() - an_expected_volume).abs() <= precision::confusion());
}

#[test]
fn BRepPrimAPI_MakeWedgeTest__WedgeFullParams() {
    let mut a_wedge =
        BRepPrimAPIMakeWedge::with_limits(20.0, 10.0, 20.0, [5.0, 5.0, 15.0, 15.0]).unwrap();
    let a_shape = a_wedge.shape().unwrap().clone();
    assert!(
        a_wedge.is_done(),
        "Wedge with full parameters creation failed"
    );
    assert!(!a_shape.is_null(), "Resulting shape is null");
    assert!(
        is_valid(&a_shape),
        "Wedge with full parameters shape is not valid"
    );
}
