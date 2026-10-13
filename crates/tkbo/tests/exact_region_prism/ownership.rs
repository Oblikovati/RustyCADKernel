// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};
#[test]
fn exact_region_prism_records_shared_input_mutation() {
    let source = cubic_source(1., false);
    let arrangement = arranged_cubic(&source);
    let before = representation_count(&source);
    let builder = regions(&arrangement);
    let after = representation_count(&source);
    // Non-destructive General Fuse preserves an unchanged edge by sharing it. Adding its
    // region pcurve then changes caller-owned state; this is NOT an immutable extension.
    assert_eq!((before, after), (1, 2));
    assert!(
        include_str!("occt-8.0.0.csv")
            .lines()
            .any(|row| row == "input-curves,1,2")
    );
    assert_eq!(builder.areas().len(), 2);
}

#[test]
fn exact_region_prism_copy_preserves_source_and_survives_source_mutation() {
    let source = cubic_source(1., false);
    let copied = copied_cubic(&source);
    let builder = regions(&arranged_cubic(&copied));
    assert_eq!(representation_count(&source), 1);
    assert_eq!(representation_count(&copied), 2);
    let region = classification::bounded_cubic(&builder, 1.);
    mutate_source_pole(&source);
    classification::check_seed_corpus(&region, 1.);
    solid::verify_region_prism(&region, 1.);
}
fn copied_cubic(source: &Shape) -> Shape {
    let mut copier = BRepBuilderAPICopy::with_shape(source, true, false).unwrap();
    let copied = copier.shape().unwrap().clone();
    assert!(!copied.is_same(source));
    let relations = copier.modified(source).unwrap();
    assert_eq!(relations.len(), 1);
    assert!(relations.first().unwrap().is_same(&copied));
    copied
}
fn representation_count(source: &Shape) -> usize {
    edge_tshape(source, "cubic ownership probe")
        .unwrap()
        .curves()
        .len()
}
fn mutate_source_pole(source: &Shape) {
    let (curve, _, _) = BRepTool::curve(&edge(source).unwrap()).unwrap();
    let curve = curve.cast::<GeomBezierCurve>();
    let curve = curve.get().unwrap();
    let moved = Point3::new(8., 0., 0.);
    curve.set_pole(2, moved).unwrap();
    assert_eq!(curve.pole(2).unwrap(), moved);
    assert_native_copy_receipt();
}

fn assert_native_copy_receipt() {
    let receipt = include_str!("occt-8.0.0.csv");
    for expected in ["copied-input-curves,1,2", "copied-source-pole,4,8"] {
        assert!(receipt.lines().any(|row| row == expected));
    }
}
