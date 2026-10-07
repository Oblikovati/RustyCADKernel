//! Direct translation of src/ModelingAlgorithms/TKBO/GTests/BRepAlgoAPI_Common_Test.cxx (3d097a0).
#![allow(non_snake_case)]

#[path = "bop_test_utilities/mod.rs"]
mod bop_test_utilities;
use bop_test_utilities::*;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeSphere;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use std::f64::consts::PI;

fn count_shapes(shape: &Shape, kind: ShapeEnum) -> usize {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut count = 0;
    while explorer.more() {
        count += 1;
        explorer.next().unwrap();
    }
    count
}

fn sphere(radius: f64) -> Shape {
    BRepPrimAPIMakeSphere::new(radius)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

// Test bopcommon_simple/A1: box b1 0 0 0 1 1 1; box b2 0 0 0 1 1 1; bop b1 b2; bopcommon result;
// checkprops result -s 6
#[test]
fn BOPCommonSimpleTest__IdenticalBoxes_A1() {
    let box1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    let box2 = CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0);
    let result = PerformCommon(&box1, &box2);
    ValidateResult(&result, 6.0, -1.0, false);
}

// Migrated from tests/bugs/modalg_5/bug23855.  The common of two coincident
// spheres must preserve the complete solid, not produce a partial shell.
#[test]
fn BOPCommonSimpleTest__OCC23855_CoincidentSpheresPreserveSolid() {
    let result = PerformCommon(&sphere(10.0), &sphere(10.0));
    assert!(!result.is_null());
    assert_eq!(count_shapes(&result, ShapeEnum::Solid), 1);
    assert_eq!(count_shapes(&result, ShapeEnum::Face), 1);
    let area = GetSurfaceArea(&result);
    assert!((area - 4.0 * PI * 100.0).abs() <= 1.0e-6, "area {area}");
    let volume = GetVolume(&result);
    assert!(
        (volume - 4.0 * PI * 1000.0 / 3.0).abs() <= 1.0e-6,
        "volume {volume}"
    );
}
