//! Translation of BRepBuilderAPI_Copy_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871, including caf/nam/A4.
#![allow(non_snake_case)]
mod primitive_legacy_support;
use primitive_legacy_support::*;
use rustycad_tkbrep::{topexp::Explorer, topods::Shape};
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{Point3, Transform3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

fn legacy_copy(source: &Shape, copy_geometry: bool) -> Shape {
    let mut copier = BRepBuilderAPICopy::with_shape(source, copy_geometry, false).unwrap();
    let copied = copier.shape().unwrap().clone();
    assert!(!copied.is_null(), "Copied shape is null");
    copied
}

#[test]
fn BRepBuilderAPI_CopyTest__CopyIsValid() {
    let source = legacy_box_sizes(10.0, 10.0, 10.0);
    let copied = legacy_copy(&source, true);
    let analyzer = BRepCheckAnalyzer::new(&copied, true, false, false).unwrap();
    assert!(analyzer.is_valid().unwrap());
}

#[test]
fn BRepBuilderAPI_CopyTest__CopyVolume() {
    let source = legacy_box_sizes(10.0, 10.0, 10.0);
    let original_volume = legacy_volume_properties(&source).mass();
    let copied = legacy_copy(&source, true);
    let copied_volume = legacy_volume_properties(&copied).mass();
    assert!((copied_volume - original_volume).abs() <= precision::confusion());
}

#[test]
fn BRepBuilderAPI_CopyTest__CopyIsDistinct() {
    let source = legacy_box_sizes(10.0, 10.0, 10.0);
    let copied = legacy_copy(&source, true);
    assert!(!copied.is_equal(&source));
}

#[test]
fn BRepBuilderAPI_CopyTest__CopyGeomTrue() {
    let source = legacy_box_sizes(10.0, 10.0, 10.0);
    let copied = legacy_copy(&source, true);
    let original_properties = legacy_volume_properties(&source);
    let copied_properties = legacy_volume_properties(&copied);
    assert!(
        (copied_properties.mass() - original_properties.mass()).abs() <= precision::confusion()
    );
}

#[test]
fn BRepBuilderAPI_CopyTest__CopyGeomFalse() {
    let source = legacy_box_sizes(10.0, 10.0, 10.0);
    let copied = legacy_copy(&source, false);
    let analyzer = BRepCheckAnalyzer::new(&copied, true, false, false).unwrap();
    assert!(analyzer.is_valid().unwrap());
}

fn legacy_surface_center(shape: &Shape) -> Point3 {
    let mut properties = GProps::new();
    BRepGProp::surface_properties(shape, &mut properties, false, false).unwrap();
    properties.centre_of_mass()
}

fn matching_copied_faces(original: &Shape, copied: &Shape) -> usize {
    let center = legacy_surface_center(original);
    let mut matches = 0;
    for face in Explorer::new(copied, ShapeEnum::Face, ShapeEnum::Shape).unwrap() {
        if legacy_surface_center(&face).is_equal(&center, precision::confusion()) {
            matches += 1;
            assert!(!face.is_same(original));
        }
    }
    matches
}

fn assert_copy_face_centers(source: &Shape, copied: &Shape) {
    let mut original_count = 0;
    for face in Explorer::new(source, ShapeEnum::Face, ShapeEnum::Shape).unwrap() {
        original_count += 1;
        assert_eq!(matching_copied_faces(&face, copied), 1);
    }
    let copied_count = Explorer::new(copied, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .count();
    assert_eq!(original_count, 6);
    assert_eq!(copied_count, original_count);
}

// caf/nam/A4: copied faces stay distinct and preserve their centers after translation.
#[test]
fn BRepBuilderAPI_CopyTest__CafNamedShape_A4_CopyAndTranslate() {
    let source = legacy_box_sizes(12.0, 13.0, 14.0);
    let copied = legacy_copy(&source, true);
    assert_copy_face_centers(&source, &copied);
    let moved = legacy_transform(&source, Transform3::translation([50.0, 40.0, 30.0]));
    let moved_copy = legacy_copy(&moved, true);
    let original_properties = legacy_volume_properties(&moved);
    let copied_properties = legacy_volume_properties(&moved_copy);
    let center = original_properties.centre_of_mass();
    assert!(center.is_equal(&copied_properties.centre_of_mass(), precision::confusion()));
    assert!(center.is_equal(&Point3::new(56.0, 46.5, 37.0), precision::confusion()));
    assert!(
        BRepCheckAnalyzer::new(&moved_copy, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
}
