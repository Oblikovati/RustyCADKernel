// SPDX-License-Identifier: GPL-2.0-only
//! Independent metric expectations supplement complete occurrence history, never replace certificates.
use super::{tests::*, *};
use crate::sketch_region::equal_occurrences;
use crate::sketch_region_candidates::tests::candidates;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg3d::{
    geom::{GeomBezierCurve, GeomCurve},
    top_abs::ShapeEnum,
};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::{
    gp::{Point3, Transform3, Vector3},
    toploc::Location,
};
use rustycad_tktopalgo::{brep_builder_api::BRepBuilderAPIMakeEdge, brep_gprop::BRepGProp};
fn mixed_boundary() -> Vec<Shape> {
    let points = [
        Point3::default(),
        Point3::new(2., 0., 0.),
        Point3::new(4., 3., 0.),
    ];
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomBezierCurve::new(&Array1::from_shared(&points, 1)).unwrap()).cast();
    let curved = BRepBuilderAPIMakeEdge::from_curve(&curve)
        .unwrap()
        .edge()
        .unwrap();
    let straight = BRepBuilderAPIMakeEdge::from_points(points[2], points[0])
        .unwrap()
        .edge()
        .unwrap();
    vec![curved.into_shape(), straight.into_shape()]
}
fn assert_metrics(result: &PrismReceipt, volume: f64, faces: usize) {
    let diagnostic = result.diagnostic.as_ref().unwrap();
    let count = equal_occurrences(&diagnostic.body)
        .unwrap()
        .iter()
        .filter(|s| s.shape_type().unwrap() == ShapeEnum::Face)
        .count();
    assert_eq!(count, faces);
    let mut properties = GProps::new();
    BRepGProp::volume_properties(&diagnostic.body, &mut properties, true, false, false).unwrap();
    // Independent fixture integrals are diagnostic metric checks, not modelling error limits.
    assert!((properties.mass() - volume).abs() < 1e-8);
    assert!(matches!(result.completion, PrismCompletion::Incomplete));
    assert!(!diagnostic.coverage.as_ref().unwrap().complete());
}
#[test]
fn mixed_bezier_line_boundary_uses_same_pipeline_with_independent_integral() {
    let result = retain_selected_prism(
        candidates(&mixed_boundary()),
        request([2., 1.]),
        &mut progress(None, None),
    );
    // Integral of 3x/4 - 3x²/16 over [0,4] is 2; extrusion height 2 gives 4.
    assert_metrics(&result, 4., 4);
    assert!(
        result
            .diagnostic
            .unwrap()
            .relations
            .iter()
            .all(|r| !r.modified.is_empty() || !r.generated.is_empty())
    );
}
#[test]
fn cubic_prism_retains_independent_native_metrics_beside_full_history() {
    let result = cubic_receipt([1., 1.], &mut progress(None, None));
    assert_metrics(&result, 4.8, 3);
    assert_native_cubic_receipt();
}
fn shifted_alias(source: &Shape) -> Shape {
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(10., 0., 0.));
    source.located(&Location::new(transform), true).unwrap()
}
#[test]
fn same_physical_source_at_two_locations_keeps_selected_parent_context() {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    for (seed, expected) in [([1., 1.], 1), ([11., 1.], 2)] {
        let alias = shifted_alias(&source);
        let result = retain_selected_prism(
            candidates(&[source.clone(), alias]),
            request(seed),
            &mut progress(None, None),
        );
        let rows = &result.diagnostic.as_ref().unwrap().relations;
        assert!(rows.iter().all(|r| {
            r.modified
                .iter()
                .chain(&r.generated)
                .all(|key| key.source.0 == expected)
        }));
        assert_metrics(&result, 4.8, 3);
    }
}

fn assert_native_cubic_receipt() {
    let row = include_str!("../../tests/exact_region_prism/occt-8.0.0.csv")
        .lines()
        .find(|row| row.starts_with("solid,"))
        .unwrap();
    let values: Vec<f64> = row
        .split(',')
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect();
    assert!((values[0] - 4.8).abs() < 1e-8);
    assert_eq!(values[1], 3.);
}
