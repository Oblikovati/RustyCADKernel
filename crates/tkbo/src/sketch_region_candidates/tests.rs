// SPDX-License-Identifier: GPL-2.0-only
use super::*;
pub(super) use crate::closed_analytic_cubic as cubic;
use crate::sketch_region::{
    ArrangementCompletion, ArrangementSource, OwnedArrangement, RegionSourceId,
    arrange_owned_sources,
};
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::{
    gp::{Plane3, Point3},
    predicates::roots::RootIsolationOptions,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakePolygon;
pub(crate) fn arranged(shapes: &[Shape]) -> OwnedArrangement {
    let sources: Vec<_> = shapes
        .iter()
        .enumerate()
        .map(|(index, shape)| ArrangementSource {
            identity: RegionSourceId(index as u64 + 1),
            shape,
        })
        .collect();
    let ArrangementCompletion::Arranged(owned) = arrange_owned_sources(&sources, &mut |_| false)
    else {
        panic!("expected owned arrangement");
    };
    owned
}
pub(crate) fn candidates(shapes: &[Shape]) -> CandidateBatch {
    extract_candidates(&arranged(shapes), Plane3::default(), &mut |_| false)
        .unwrap_or_else(|_| panic!("expected mapped candidates with captured boundaries"))
}
pub(super) fn selection(batch: &CandidateBatch, seed: [f64; 2]) -> SeedSelection {
    let result = classify_candidates(batch, seed, RootIsolationOptions::new(0.001), || false)
        .unwrap_or_else(|_| panic!("expected exact stored-boundary classification"));
    let CandidateClassification::Incomplete {
        selection, missing, ..
    } = result;
    assert!(missing.len() >= 5);
    selection
}
pub(crate) fn rectangle(x: f64, y: f64, width: f64, height: f64) -> Vec<Shape> {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for (u, v) in [
        (x, y),
        (x + width, y),
        (x + width, y + height),
        (x, y + height),
    ] {
        polygon.add_point(Point3::new(u, v, 0.)).unwrap();
    }
    polygon.close().unwrap();
    ShapeIterator::new(&polygon.wire().unwrap(), true, true).collect()
}
#[test]
fn cubic_selection_has_no_expected_poles_or_fixture_sign_shortcut() {
    for scale in [0.5, 1., 2.] {
        for reversed in [false, true] {
            let batch = candidates(&[cubic::closed_analytic_cubic(scale, reversed)]);
            assert!(matches!(
                selection(&batch, [scale, scale]),
                SeedSelection::Unique { .. }
            ));
            assert_eq!(selection(&batch, [-scale, scale]), SeedSelection::Empty);
            assert!(matches!(
                selection(&batch, [0., 0.]),
                SeedSelection::Boundary { .. }
            ));
        }
    }
}
#[test]
fn exact_line_regions_and_reversed_occurrences_use_same_pipeline() {
    let mut edges = rectangle(0., 0., 4., 3.);
    for _ in 0..2 {
        let batch = candidates(&edges);
        assert!(matches!(
            selection(&batch, [2., 1.]),
            SeedSelection::Unique { .. }
        ));
        assert_eq!(selection(&batch, [5., 1.]), SeedSelection::Empty);
        assert!(matches!(
            selection(&batch, [0., 1.]),
            SeedSelection::Boundary { .. }
        ));
        edges = edges
            .iter()
            .map(|e| e.oriented(Orientation::Reversed))
            .collect();
    }
}
#[test]
fn nested_boundaries_have_hole_candidate_and_unique_inner_region() {
    let mut edges = rectangle(0., 0., 8., 8.);
    edges.extend(rectangle(2., 2., 4., 4.));
    let batch = candidates(&edges);
    assert!(batch.candidates.iter().any(|c| c.wires.len() == 2));
    assert!(matches!(
        selection(&batch, [1., 1.]),
        SeedSelection::Unique { .. }
    ));
    assert!(matches!(
        selection(&batch, [3., 3.]),
        SeedSelection::Unique { .. }
    ));
    assert!(matches!(
        selection(&batch, [2., 3.]),
        SeedSelection::Boundary { .. }
    ));
}
#[test]
fn touching_source_regions_report_all_boundary_occurrences() {
    let mut edges = rectangle(0., 0., 2., 2.);
    edges.extend(rectangle(2., 2., 2., 2.));
    let batch = candidates(&edges);
    let SeedSelection::Boundary { occurrences } = selection(&batch, [2., 2.]) else {
        panic!("expected boundary");
    };
    assert!(occurrences.len() >= 2);
    assert!(matches!(
        selection(&batch, [1., 1.]),
        SeedSelection::Unique { .. }
    ));
}
#[test]
fn original_parent_ids_and_occurrences_survive_candidate_copy() {
    let original = cubic::closed_analytic_cubic(1., false);
    let reversed = original.oriented(Orientation::Reversed);
    let batch = candidates(&[original, reversed]);
    for segment in batch
        .candidates
        .iter()
        .flat_map(|c| &c.wires)
        .flat_map(|w| &w.segments)
    {
        let ids: Vec<_> = segment.parents.iter().map(|p| p.source).collect();
        assert!(ids.contains(&RegionSourceId(1)) && ids.contains(&RegionSourceId(2)));
    }
}
#[test]
fn missing_ancestry_and_boundary_cancellation_are_not_success() {
    let mut owned = arranged(&[cubic::closed_analytic_cubic(1., false)]);
    for phase in [
        CandidatePhase::Copy,
        CandidatePhase::Generation,
        CandidatePhase::Capture,
    ] {
        let result = extract_candidates(&owned, Plane3::default(), &mut |current| current == phase);
        assert!(matches!(result,Err(CandidateCause::Cancelled(actual)) if actual==phase));
    }
    owned.relations.clear();
    let result = extract_candidates(&owned, Plane3::default(), &mut |_| false);
    assert!(matches!(
        result,
        Err(CandidateCause::Incomplete(
            CandidateGap::MissingParents { .. }
        ))
    ));
}

pub(super) fn rational_closed_curve() -> Shape {
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
    BRepBuilderAPIMakeEdge::from_curve(&rational_fixture_curve())
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}
fn rational_fixture_curve()
-> rustycad_tkernel::standard::Handle<dyn rustycad_tkg3d::geom::GeomCurve> {
    use rustycad_tkernel::{ncollection::Array1, standard::Handle};
    use rustycad_tkg3d::geom::GeomBezierCurve;
    let points = [
        Point3::new(0., 0., 0.),
        Point3::new(4., 0., 0.),
        Point3::new(0., 4., 0.),
        Point3::new(0., 0., 0.),
    ];
    let weights = Array1::from_shared(&[1., 2., 2., 1.], 1);
    Handle::new(GeomBezierCurve::with_weights(&Array1::from_shared(&points, 1), &weights).unwrap())
        .cast()
}

#[test]
fn rational_stored_boundaries_share_the_exact_wire_classifier() {
    let batch = candidates(&[rational_closed_curve()]);
    assert!(
        batch
            .candidates
            .iter()
            .flat_map(|c| &c.wires)
            .flat_map(|w| &w.segments)
            .all(|s| s.weights.is_some())
    );
    assert!(matches!(
        selection(&batch, [1., 1.]),
        SeedSelection::Unique { .. }
    ));
    assert_eq!(selection(&batch, [5., 1.]), SeedSelection::Empty);
}
#[test]
fn duplicate_actual_candidates_report_ambiguity_without_first_match() {
    let mut batch = candidates(&rectangle(0., 0., 4., 4.));
    let SeedSelection::Unique { candidate } = selection(&batch, [1., 1.]) else {
        panic!("expected unique");
    };
    let mut repeated = candidates(&rectangle(0., 0., 4., 4.));
    batch.candidates.push(repeated.candidates.remove(candidate));
    assert!(
        matches!(selection(&batch,[1.,1.]),SeedSelection::Ambiguous{candidates} if candidates.len()==2)
    );
    assert_eq!(
        selection(&empty_candidate_batch(), [1., 1.]),
        SeedSelection::Empty
    );
}
#[test]
fn unsupported_circle_is_explicit_incomplete_not_conversion_retry() {
    use rustycad_tkmath::gp::{Axis2, Circle3};
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
    let circle = BRepBuilderAPIMakeEdge::from_circ(Circle3::new(Axis2::default(), 2.).unwrap())
        .unwrap()
        .edge()
        .unwrap()
        .into_shape();
    let owned = arranged(&[circle]);
    let outcome = extract_candidates(&owned, Plane3::default(), &mut |_| false);
    assert!(matches!(
        outcome,
        Err(CandidateCause::Incomplete(
            CandidateGap::UnsupportedCurve { .. }
        ))
    ));
}
#[test]
fn numerical_cancellation_keeps_original_typed_cause() {
    use rustycad_tkmath::predicates::roots::bezier_line::BezierWireError;
    let batch = candidates(&rectangle(0., 0., 4., 4.));
    let outcome = classify_candidates(&batch, [1., 1.], RootIsolationOptions::new(0.001), || true);
    assert!(matches!(
        outcome,
        Err(CandidateCause::Numerical(BezierWireError::Cancelled { .. }))
    ));
}

fn empty_candidate_batch() -> CandidateBatch {
    CandidateBatch {
        plane: Plane3::default(),
        candidates: Vec::new(),
        input_deleted: Vec::new(),
        unrepresented: Vec::new(),
        mapped_report: std::sync::Arc::new(rustycad_tkernel::message::Report::new()),
    }
}
