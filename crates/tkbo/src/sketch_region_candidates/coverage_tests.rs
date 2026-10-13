// SPDX-License-Identifier: GPL-2.0-only
use super::{boundaries::capture_wire, outcome::*, tests::*};
use crate::sketch_region::equal_occurrences;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Builder, Shape, Wire, edge},
};
use rustycad_tkg3d::{geom::GeomBezierCurve, top_abs::ShapeEnum};
use rustycad_tkmath::gp::{Plane3, Point3};
#[test]
fn all_candidate_occurrences_have_actual_composed_parent_relations() {
    let batch = candidates(&rectangle(0., 0., 4., 4.));
    for candidate in &batch.candidates {
        let outputs = equal_occurrences(&candidate.face).unwrap();
        assert_eq!(outputs.len(), candidate.relations.len());
        for output in &outputs {
            let relation = candidate
                .relations
                .iter()
                .find(|r| r.output.is_equal(output))
                .unwrap();
            assert!(!relation.modified.is_empty() || !relation.generated.is_empty());
        }
    }
}
#[test]
fn disconnected_wire_retains_omitted_actual_occurrence_and_is_incomplete() {
    let batch = candidates(&rectangle(0., 0., 4., 4.));
    let candidate = &batch.candidates[0];
    let segments = &candidate.wires[0].segments;
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    let mut shape = wire.into_shape();
    Builder.add(&mut shape, &segments[0].edge).unwrap();
    Builder.add(&mut shape, &segments[2].edge).unwrap();
    let parents = segment_test_relations(segments);
    let result = capture_wire(&shape, &candidate.face, &parents, &mut 1);
    assert!(
        matches!(result,Err(CandidateCause::Incomplete(CandidateGap::IncompleteWire{omitted,..})) if omitted.len()==1)
    );
}
#[test]
fn candidate_pcurve_mutation_isolated_from_published_arrangement_and_input() {
    let source = cubic::closed_analytic_cubic(1., false);
    let owned = arranged(std::slice::from_ref(&source));
    let original = first_arranged_edge(&owned);
    let before = representation_count(&original);
    let batch = super::extract_candidates(&owned, Plane3::default(), &mut |_| false)
        .unwrap_or_else(|_| panic!("expected candidates"));
    assert_eq!(representation_count(&original), before);
    mutate_source_curve(&source);
    assert!(matches!(
        selection(&batch, [1., 1.]),
        SeedSelection::Unique { .. }
    ));
    assert_copied_geometry(&batch);
}
fn representation_count(shape: &Shape) -> usize {
    rustycad_tkbrep::brep::access::edge_tshape(shape, "candidate isolation")
        .unwrap()
        .curves()
        .len()
}
fn assert_copied_geometry(batch: &CandidateBatch) {
    for segment in batch
        .candidates
        .iter()
        .flat_map(|c| &c.wires)
        .flat_map(|w| &w.segments)
    {
        let (curve, _, _) = BRepTool::curve(&edge(&segment.edge).unwrap()).unwrap();
        assert_eq!(
            curve
                .cast::<GeomBezierCurve>()
                .get()
                .unwrap()
                .pole(2)
                .unwrap(),
            Point3::new(4., 0., 0.)
        );
    }
}

fn mutate_source_curve(source: &Shape) {
    let (curve, _, _) = BRepTool::curve(&edge(source).unwrap()).unwrap();
    curve
        .cast::<GeomBezierCurve>()
        .get()
        .unwrap()
        .set_pole(2, Point3::new(8., 0., 0.))
        .unwrap();
}

#[test]
fn mapped_candidate_report_retains_alert_payload_after_builder_reuse() {
    use crate::bop_algo::BuilderFace;
    use rustycad_tkernel::message::Gravity;
    let mut builder = BuilderFace::new();
    builder.perform().unwrap();
    let Err(CandidateCause::MappedReport(retained)) = super::generation::check_builder(&builder)
    else {
        panic!("expected mapped report");
    };
    let actual = builder.options().report().get_alerts(Gravity::Fail);
    assert_eq!(actual.len(), 1);
    builder.options().report().clear();
    let alerts = retained.get_alerts(Gravity::Fail);
    assert_eq!(alerts.len(), 1);
    assert!(std::sync::Arc::ptr_eq(&alerts[0], &actual[0]));
}
#[test]
fn classification_records_shared_work_budget_and_unknown_construction_error() {
    let batch = candidates(&[cubic::closed_analytic_cubic(1., false)]);
    let options = rustycad_tkmath::predicates::roots::RootIsolationOptions::new(0.001);
    let result = super::classify_candidates(&batch, [1., 1.], options, || false)
        .unwrap_or_else(|_| panic!("expected complete stored-polynomial classification"));
    let CandidateClassification::Incomplete {
        bisections,
        requested,
        remaining,
        achieved_error,
        ..
    } = result;
    assert_eq!(requested, options);
    assert_eq!(
        u32::from(remaining.max_bisections) + bisections,
        u32::from(options.max_bisections)
    );
    assert_eq!(remaining.max_width, options.max_width);
    assert_eq!(achieved_error, None);
}

#[test]
fn located_source_and_declared_plane_preserve_local_seed_coordinates() {
    let source = located_cubic();
    let owned = arranged(&[source]);
    let mut plane = Plane3::default();
    plane.set_location(Point3::new(3., 4., 5.));
    let batch = super::extract_candidates(&owned, plane, &mut |_| false)
        .unwrap_or_else(|_| panic!("expected located candidate"));
    assert_eq!(batch.plane.location(), plane.location());
    assert!(matches!(
        selection(&batch, [1., 1.]),
        SeedSelection::Unique { .. }
    ));
}

#[test]
fn generated_intersection_vertex_keeps_generated_relation_through_copy() {
    let owned = arranged(&cut_edges());
    let original = generated_vertex_keys(&owned);
    assert!(!original.is_empty());
    let relations = generated_candidate_relations(&owned);
    assert_generated_keys(&relations, &original);
    let outcome = super::extract_candidates(&owned, Plane3::default(), &mut |_| false);
    assert!(matches!(
        outcome,
        Err(CandidateCause::Incomplete(
            CandidateGap::IncompleteWire { .. }
        ))
    ));
}
pub(super) fn generated_candidate_relations(
    owned: &crate::sketch_region::OwnedArrangement,
) -> Vec<crate::sketch_region::ArrangementRelation> {
    use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};
    let mut copied = BRepBuilderAPICopy::with_shape(&owned.arrangement, true, true).unwrap();
    let parents = super::generation::copy_parents(owned, &mut copied)
        .unwrap_or_else(|_| panic!("expected copy parents"));
    let builder = super::generation::generate_faces(copied.shape().unwrap(), Plane3::default())
        .unwrap_or_else(|_| panic!("expected candidates"));
    builder
        .areas()
        .iter()
        .flat_map(|shape| {
            super::ancestry::candidate_relations(shape, &parents)
                .unwrap_or_else(|_| panic!("expected composed history"))
        })
        .collect()
}

fn generated_vertex_keys(
    owned: &crate::sketch_region::OwnedArrangement,
) -> Vec<crate::sketch_region::OriginalMemberIdentity> {
    owned
        .relations
        .iter()
        .filter(|r| r.output.shape_type().unwrap() == ShapeEnum::Vertex)
        .flat_map(|r| r.generated.clone())
        .collect()
}

#[test]
fn actual_more_value_fault_is_not_silent_end_for_face_or_wire() {
    use crate::changing_child_count::changing_shape;
    let batch = candidates(&rectangle(0., 0., 4., 4.));
    let face = &batch.candidates[0].face;
    let parents = Vec::new();
    let fault_face = changing_shape(ShapeEnum::Face);
    let face_result = super::boundaries::capture_candidate(&fault_face, &parents, &mut 1);
    assert_iterator_fault(face_result.err().unwrap());
    let fault_wire = changing_shape(ShapeEnum::Wire);
    let wire_result = capture_wire(&fault_wire, face, &parents, &mut 1);
    assert_iterator_fault(wire_result.err().unwrap());
}
fn assert_iterator_fault(cause: CandidateCause) {
    let CandidateCause::Mapped(failure) = cause else {
        panic!("expected original Value failure");
    };
    assert_eq!(failure.exception_type(), "Standard_NoSuchObject");
    assert!(failure.message().contains("TopoDS_Iterator::Value"));
}

pub(super) fn cut_edges() -> Vec<Shape> {
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
    let mut roots = rectangle(0., 0., 4., 4.);
    roots.push(
        BRepBuilderAPIMakeEdge::from_points(Point3::new(-1., 2., 0.), Point3::new(5., 2., 0.))
            .unwrap()
            .edge()
            .unwrap()
            .into_shape(),
    );
    roots
}

#[test]
fn candidate_side_geometry_mutation_cannot_reach_source_or_arrangement() {
    let source = cubic::closed_analytic_cubic(1., false);
    let owned = arranged(std::slice::from_ref(&source));
    let batch = super::extract_candidates(&owned, Plane3::default(), &mut |_| false)
        .unwrap_or_else(|_| panic!("expected batch"));
    let segment = &batch.candidates[0].wires[0].segments[0];
    mutate_source_curve(&segment.edge);
    assert_original_geometry(&source);
    let recovered = super::extract_candidates(&owned, Plane3::default(), &mut |_| false)
        .unwrap_or_else(|_| panic!("expected independent prior arrangement"));
    assert_copied_geometry(&recovered);
}
#[test]
fn prior_candidate_survives_later_failure_cancel_and_shared_source_reuse() {
    let source = cubic::closed_analytic_cubic(1., false);
    let owned = arranged(std::slice::from_ref(&source));
    let batch = super::extract_candidates(&owned, Plane3::default(), &mut |_| false)
        .unwrap_or_else(|_| panic!("expected prior batch"));
    for phase in [
        CandidatePhase::Copy,
        CandidatePhase::Generation,
        CandidatePhase::Capture,
    ] {
        assert!(matches!(
            super::extract_candidates(&owned, Plane3::default(), &mut |p| p == phase),
            Err(CandidateCause::Cancelled(_))
        ));
    }
    assert_null_copy_failure();
    mutate_source_curve(&source);
    assert_copied_geometry(&batch);
}
#[test]
fn internal_external_and_duplicate_wire_occurrences_are_not_silently_lost() {
    use rustycad_tkg3d::top_abs::Orientation;
    let batch = candidates(&rectangle(0., 0., 4., 4.));
    let candidate = &batch.candidates[0];
    let edge = &candidate.wires[0].segments[0].edge;
    for orientation in [
        Orientation::Internal,
        Orientation::External,
        Orientation::Forward,
    ] {
        let shape = duplicate_wire(edge).oriented(orientation);
        let parents = segment_test_relations(&candidate.wires[0].segments);
        let result = capture_wire(&shape, &candidate.face, &parents, &mut 1);
        assert!(
            matches!(result,Err(CandidateCause::Incomplete(CandidateGap::IncompleteWire{omitted,..})) if !omitted.is_empty())
        );
    }
}
fn duplicate_wire(edge: &Shape) -> Shape {
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    let mut shape = wire.into_shape();
    Builder.add(&mut shape, edge).unwrap();
    Builder.add(&mut shape, edge).unwrap();
    shape
}

fn segment_test_relations(segments: &[CandidateSegment]) -> super::generation::CandidateParents {
    segments
        .iter()
        .map(|s| crate::sketch_region::ArrangementRelation {
            output: s.edge.clone(),
            preserved: Vec::new(),
            modified: s.parents.clone(),
            generated: Vec::new(),
        })
        .collect()
}

fn first_arranged_edge(owned: &crate::sketch_region::OwnedArrangement) -> Shape {
    owned
        .relations
        .iter()
        .find(|r| r.output.shape_type().unwrap() == ShapeEnum::Edge)
        .unwrap()
        .output
        .clone()
}

fn assert_original_geometry(source: &Shape) {
    let (curve, _, _) = BRepTool::curve(&edge(source).unwrap()).unwrap();
    assert_eq!(
        curve
            .cast::<GeomBezierCurve>()
            .get()
            .unwrap()
            .pole(2)
            .unwrap(),
        Point3::new(4., 0., 0.)
    );
}

fn assert_null_copy_failure() {
    let null = crate::sketch_region::OwnedArrangement {
        arrangement: Shape::default(),
        relations: Vec::new(),
        deleted: Vec::new(),
        achieved_error: None,
        incidences_certified: false,
    };
    assert!(matches!(
        super::extract_candidates(&null, Plane3::default(), &mut |_| false),
        Err(CandidateCause::Mapped(_))
    ));
}

fn assert_generated_keys(
    relations: &[crate::sketch_region::ArrangementRelation],
    original: &[crate::sketch_region::OriginalMemberIdentity],
) {
    for key in original {
        assert!(relations.iter().any(|r| r.generated.contains(key)));
        assert!(!relations.iter().any(
            |r| r.output.shape_type().unwrap() == ShapeEnum::Vertex && r.modified.contains(key)
        ));
    }
}

fn located_cubic() -> Shape {
    use rustycad_tkmath::{
        gp::{Transform3, Vector3},
        toploc::Location,
    };
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(3., 4., 5.));
    cubic::closed_analytic_cubic(1., false)
        .located(&Location::new(transform), true)
        .unwrap()
}

#[test]
fn non_wire_face_occurrence_is_explicit_incomplete() {
    use rustycad_tkbrep::{brep::BRepBuilder, topods::Vertex};
    use rustycad_tkg3d::top_abs::Orientation;
    let batch = candidates(&rectangle(0., 0., 4., 4.));
    let candidate = &batch.candidates[0];
    let parents = segment_test_relations(&candidate.wires[0].segments);
    let mut root = candidate.face.clone().into_shape();
    let mut vertex = Vertex::default();
    BRepBuilder::default().make_vertex(&mut vertex);
    Builder
        .add(&mut root, &vertex.oriented(Orientation::Internal))
        .unwrap();
    let result = super::boundaries::capture_candidate(&root, &parents, &mut 1);
    assert!(matches!(
        result,
        Err(CandidateCause::Incomplete(
            CandidateGap::UnclassifiedOccurrence { .. }
        ))
    ));
}

#[test]
fn reversed_face_keeps_material_selection_and_cumulative_edge_direction() {
    use rustycad_tkg3d::top_abs::Orientation;
    let mut batch = candidates(&rectangle(0., 0., 4., 4.));
    let SeedSelection::Unique { candidate } = selection(&batch, [1., 1.]) else {
        panic!("expected finite region");
    };
    let original = &batch.candidates[candidate];
    let reverse = original.face.oriented(Orientation::Reversed);
    let captured = super::boundaries::capture_candidate(&reverse, &original.relations, &mut 100)
        .unwrap_or_else(|_| panic!("expected reverse capture"));
    assert!(captured.wires[0].segments.iter().all(|s| s.reversed));
    batch.candidates = vec![captured];
    assert_eq!(
        selection(&batch, [1., 1.]),
        SeedSelection::Unique { candidate: 0 }
    );
}
