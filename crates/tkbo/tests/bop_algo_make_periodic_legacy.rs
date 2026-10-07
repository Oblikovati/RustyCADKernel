//! Direct translation of src/ModelingAlgorithms/TKBO/GTests/BOPAlgo_MakePeriodic_Test.cxx (3d097a0).
//! The BOPAlgo_MakeConnected tests are in the `connected` module.
#![allow(non_snake_case)]

#[path = "bop_test_utilities/mod.rs"]
mod bop_test_utilities;
#[path = "bop_algo_make_periodic_legacy/connected.rs"]
mod connected;
#[path = "bop_algo_make_periodic_legacy/helpers.rs"]
mod helpers;

use bop_test_utilities as BOPTest_Utilities;
use helpers::*;
use rustycad_tkbo::bop_algo::{BOPAlgoMakePeriodic, BOPAlgoMakerVolume};
use rustycad_tkbo::brep_algo_api::BRepAlgoAPICut;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPINurbsConvert};

fn near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} vs {expected} (tolerance {tolerance})"
    );
}

fn make_box(x: f64, y: f64, z: f64) -> Shape {
    BRepPrimAPIMakeBox::from_sizes(x, y, z)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

// Equivalent to tests/boolean/periodicity/A1.
#[test]
fn BOPAlgo_MakePeriodicTest__A1_BoxWithTrimmedPeriodicity_RepeatsWithExpectedTopology() {
    let box_shape = make_box(10.0, 10.0, 10.0);
    let mut maker = BOPAlgoMakePeriodic::new();
    MakeBoxPeriodic(
        &mut maker,
        &box_shape,
        [5.0, 8.0, 12.0],
        [false; 3],
        [2.0, 1.0, -1.0],
        true,
    );
    assert!(!maker.options().has_errors());
    let periodic = maker.shape().clone();
    assert!(!periodic.is_null());
    ExpectShapeCounts(&periodic, [8, 12, 6, 6, 1, 1]);
    near(
        BOPTest_Utilities::GetSurfaceArea(&periodic),
        340.0,
        precision::confusion(),
    );
    near(
        BOPTest_Utilities::GetVolume(&periodic),
        400.0,
        precision::confusion(),
    );

    maker.x_repeat(5).unwrap();
    maker.y_repeat(5).unwrap();
    maker.z_repeat(5).unwrap();
    let repeated = maker.repeated_shape().clone();
    assert!(!repeated.is_null());
    ExpectShapeCounts(&repeated, [588, 1302, 936, 936, 216, 216]);
    near(
        BOPTest_Utilities::GetSurfaceArea(&repeated),
        73440.0,
        precision::confusion(),
    );
    near(
        BOPTest_Utilities::GetVolume(&repeated),
        86400.0,
        precision::confusion(),
    );
}

/// The two first faces of the profile are split in two, each with one twin face.
fn expect_split_side_faces(maker: &BOPAlgoMakePeriodic, shape: &Shape) {
    let history = maker.history().get().unwrap();
    for index in 0..2 {
        let face = GetFace(shape, index);
        assert!(!face.is_null());
        let modified = history.modified(&face).unwrap();
        assert_eq!(CountShapesList(&modified, ShapeEnum::Vertex), 6);
        assert_eq!(CountShapesList(&modified, ShapeEnum::Edge), 7);
        assert_eq!(CountShapesList(&modified, ShapeEnum::Wire), 2);
        assert_eq!(CountShapesList(&modified, ShapeEnum::Face), 2);
        let modified_faces = GetFaces(&MakeCompound(&modified));
        assert_eq!(modified_faces.len(), 2);
        let twins: Vec<Shape> = modified_faces
            .iter()
            .flat_map(|face| maker.get_twins(face).iter().cloned().collect::<Vec<_>>())
            .collect();
        assert_eq!(CountShapesList(&twins, ShapeEnum::Face), 1);
    }
}

/// The generated faces of a profile face after the repetitions.
fn expect_generated(
    maker: &BOPAlgoMakePeriodic,
    face: &Shape,
    counts: [i32; 4],
    area: f64,
) -> Vec<Shape> {
    let generated = maker.history().get().unwrap().generated(face).unwrap();
    let kinds = [
        ShapeEnum::Vertex,
        ShapeEnum::Edge,
        ShapeEnum::Wire,
        ShapeEnum::Face,
    ];
    for (kind, count) in kinds.into_iter().zip(counts) {
        assert_eq!(CountShapesList(&generated, kind), count, "{kind:?}");
    }
    let generated_shape = MakeCompound(&generated);
    near(
        BOPTest_Utilities::GetSurfaceArea(&generated_shape),
        area,
        precision::confusion(),
    );
    ExpectValidShape(&generated_shape);
    generated
}

// Equivalent to tests/boolean/periodicity/A2.
#[test]
fn BOPAlgo_MakePeriodicTest__A2_ProfilePeriodicity_HistoryAndRepetitionAreConsistent() {
    let shape = MakePeriodicityProfile();
    let mut maker = BOPAlgoMakePeriodic::new();
    maker.set_shape(&shape);
    maker.make_x_periodic(true, 10.0);
    maker.make_z_periodic(true, 10.0);
    maker.make_y_periodic(true, 5.0);
    maker.options_mut().set_run_parallel(false);
    maker.perform().unwrap();
    assert!(!maker.options().has_errors());
    let periodic = maker.shape().clone();
    assert!(!periodic.is_null());
    ExpectShapeCounts(&periodic, [16, 24, 10, 10, 1, 1]);
    near(
        BOPTest_Utilities::GetSurfaceArea(&periodic),
        350.0,
        precision::confusion(),
    );
    near(
        BOPTest_Utilities::GetVolume(&periodic),
        375.0,
        precision::confusion(),
    );
    assert!(!maker.history().is_null());
    assert!(GetFaces(&shape).len() >= 3);
    expect_split_side_faces(&maker, &shape);

    maker.x_repeat(1).unwrap();
    maker.y_repeat(2).unwrap();
    let repeated = maker.repeated_shape().clone();
    assert!(!repeated.is_null());
    ExpectShapeCounts(&repeated, [56, 102, 53, 53, 6, 6]);
    near(
        BOPTest_Utilities::GetSurfaceArea(&repeated),
        2100.0,
        precision::confusion(),
    );
    near(
        BOPTest_Utilities::GetVolume(&repeated),
        2250.0,
        precision::confusion(),
    );

    let (bottom, top) = (GetFace(&shape, 0), GetFace(&shape, 2));
    assert!(!bottom.is_null() && !top.is_null());
    expect_generated(&maker, &bottom, [20, 31, 12, 12], 300.0);
    let top_generated = expect_generated(&maker, &top, [16, 20, 6, 6], 150.0);
    for generated in &top_generated {
        let twins: Vec<Shape> = maker.get_twins(generated).iter().cloned().collect();
        assert_eq!(CountShapesList(&twins, ShapeEnum::Wire), 11);
        assert_eq!(CountShapesList(&twins, ShapeEnum::Face), 11);
        let twins_shape = MakeCompound(&twins);
        near(
            BOPTest_Utilities::GetSurfaceArea(&twins_shape),
            275.0,
            precision::confusion(),
        );
        ExpectValidShape(&twins_shape);
    }

    maker.clear_repetitions().unwrap();
    assert!(
        maker
            .history()
            .get()
            .unwrap()
            .generated(&bottom)
            .unwrap()
            .is_empty()
    );
}

fn cylinder_at_cell_center() -> Shape {
    BOPTest_Utilities::Translate(&BOPTest_Utilities::CreateCylinder(2.0, 2.0), 2.5, 2.5, 0.0)
}

// Equivalent to tests/boolean/periodicity/A3.
#[test]
fn BOPAlgo_MakePeriodicTest__A3_PeriodicCylinderCut_RepeatedCutHasExpectedTopology() {
    let box_shape = BOPTest_Utilities::CreateBox(Point3::new(0.0, 0.0, 0.0), 100.0, 100.0, 1.0);
    let cylinder = cylinder_at_cell_center();
    let mut maker = BOPAlgoMakePeriodic::new();
    maker.set_shape(&cylinder);
    maker.make_x_periodic(true, 5.0);
    maker.make_y_periodic(true, 5.0);
    maker.set_x_trimmed(false, false);
    maker.set_y_trimmed(false, false);
    maker.options_mut().set_run_parallel(false);
    maker.perform().unwrap();
    assert!(!maker.options().has_errors());
    maker.x_repeat(19).unwrap();
    maker.y_repeat(19).unwrap();

    let cut = BRepAlgoAPICut::from_shapes(&box_shape, maker.repeated_shape()).unwrap();
    assert!(cut.is_done());
    let result = cut.shape().clone();
    assert!(!result.is_null());
    ExpectShapeCounts(&result, [808, 1212, 1206, 406, 1, 1]);
    near(
        BOPTest_Utilities::GetSurfaceArea(&result),
        15373.5,
        15373.5 * 0.01,
    );
    near(
        BOPTest_Utilities::GetVolume(&result),
        4973.45,
        4973.45 * 0.01,
    );
}

// Equivalent to tests/boolean/periodicity/A4.
#[test]
fn BOPAlgo_MakePeriodicTest__A4_PeriodicCutResult_RepeatsInBothDirections() {
    let box_shape = BOPTest_Utilities::CreateBox(Point3::new(0.0, 0.0, 0.0), 5.0, 5.0, 1.0);
    let cut = BRepAlgoAPICut::from_shapes(&box_shape, &cylinder_at_cell_center()).unwrap();
    assert!(cut.is_done());
    let mut maker = BOPAlgoMakePeriodic::new();
    maker.set_shape(cut.shape());
    maker.make_x_periodic(true, 5.0);
    maker.make_y_periodic(true, 5.0);
    maker.options_mut().set_run_parallel(false);
    maker.perform().unwrap();
    assert!(!maker.options().has_errors());
    maker.x_repeat(-9).unwrap();
    maker.x_repeat(1).unwrap();
    maker.y_repeat(-9).unwrap();
    maker.y_repeat(1).unwrap();

    let result = maker.repeated_shape().clone();
    assert!(!result.is_null());
    ExpectShapeCounts(&result, [1682, 3321, 2840, 2040, 400, 400]);
    near(
        BOPTest_Utilities::GetSurfaceArea(&result),
        22973.5,
        22973.5 * 0.01,
    );
    near(
        BOPTest_Utilities::GetVolume(&result),
        4973.45,
        4973.45 * 0.01,
    );
}

/// The NURBS conversion of a box sub-shape, glued back into a solid with the box faces;
/// the solid and the image of the converted shape.
fn nurbs_volume(converted_from: &Shape, faces: &[rustycad_tkbrep::topods::Face]) -> (Shape, Shape) {
    let mut converter = BRepBuilderAPINurbsConvert::new();
    converter.perform(converted_from, false).unwrap();
    assert!(converter.is_done());
    let nurbs = converter.shape().unwrap().clone();
    assert!(!nurbs.is_null());
    let mut volume = BOPAlgoMakerVolume::new();
    let volume_history = MakeVolume(&mut volume, &nurbs, faces);
    assert!(!volume.options().has_errors());
    let solid = volume.shape().clone();
    assert!(!solid.is_null());
    ExpectValidShape(&solid);
    let modified = volume_history.modified(&nurbs).unwrap();
    (solid, modified.first().cloned().unwrap_or_default())
}

/// The single image of `shape` in the periodic shape.
fn periodic_image(maker: &BOPAlgoMakePeriodic, shape: &Shape, kind: ShapeEnum) -> Shape {
    let modified = maker.history().get().unwrap().modified(shape).unwrap();
    assert_eq!(CountShapesList(&modified, kind), 1);
    modified[0].clone()
}

// Equivalent to tests/boolean/periodicity/A5.
#[test]
fn BOPAlgo_MakePeriodicTest__A5_NurbsEdges_PeriodicTwinsPreserveCurveType() {
    let box_shape = make_box(10.0, 10.0, 10.0);
    let faces = GetFaces(&box_shape);
    for edge in GetEdges(&box_shape) {
        let (solid, modified) = nurbs_volume(&edge, &faces);
        assert_eq!(CountShapes(&modified, ShapeEnum::Edge), 1);
        let modified_edge = topods::edge(&modified).unwrap();
        assert!(IsBSplineCurve(&modified_edge));

        let mut maker = BOPAlgoMakePeriodic::new();
        maker.clear().unwrap();
        MakeBoxPeriodic(&mut maker, &solid, [10.0; 3], [true; 3], [0.0; 3], false);
        assert!(!maker.options().has_errors());
        assert!(!maker.history().is_null());
        let periodic_edge =
            topods::edge(&periodic_image(&maker, &modified_edge, ShapeEnum::Edge)).unwrap();
        let periodic_type = CurveTypeName(&periodic_edge).unwrap();
        let twins: Vec<Shape> = maker.get_twins(&periodic_edge).iter().cloned().collect();
        assert_eq!(CountShapesList(&twins, ShapeEnum::Edge), 2);
        for twin in &twins {
            assert_eq!(
                CurveTypeName(&topods::edge(twin).unwrap()).unwrap(),
                periodic_type
            );
        }
    }
}

// Equivalent to tests/boolean/periodicity/A6.
#[test]
fn BOPAlgo_MakePeriodicTest__A6_NurbsFaces_PeriodicTwinsPreserveGeometryTypes() {
    let box_shape = make_box(10.0, 10.0, 10.0);
    let faces = GetFaces(&box_shape);
    for face in &faces {
        let (solid, modified) = nurbs_volume(face, &faces);
        assert_eq!(CountShapes(&modified, ShapeEnum::Face), 1);
        let modified_face = topods::face(&modified).unwrap();
        assert!(IsBSplineSurface(&modified_face));

        let mut maker = BOPAlgoMakePeriodic::new();
        MakeBoxPeriodic(&mut maker, &solid, [10.0; 3], [true; 3], [0.0; 3], false);
        assert!(!maker.options().has_errors());
        assert!(!maker.history().is_null());
        let periodic_face =
            topods::face(&periodic_image(&maker, &modified_face, ShapeEnum::Face)).unwrap();
        let surface_type = SurfaceTypeName(&periodic_face).unwrap();
        let face_twins: Vec<Shape> = maker.get_twins(&periodic_face).iter().cloned().collect();
        assert_eq!(CountShapesList(&face_twins, ShapeEnum::Face), 1);
        for twin in &face_twins {
            assert_eq!(
                SurfaceTypeName(&topods::face(twin).unwrap()).unwrap(),
                surface_type
            );
        }
        for edge in GetEdges(&periodic_face) {
            let curve_type = CurveTypeName(&edge).unwrap();
            let edge_twins: Vec<Shape> = maker.get_twins(&edge).iter().cloned().collect();
            assert_eq!(CountShapesList(&edge_twins, ShapeEnum::Edge), 2);
            for twin in &edge_twins {
                assert_eq!(
                    CurveTypeName(&topods::edge(twin).unwrap()).unwrap(),
                    curve_type
                );
            }
        }
    }
}
