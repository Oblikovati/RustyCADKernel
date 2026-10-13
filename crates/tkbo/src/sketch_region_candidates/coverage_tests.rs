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
    let parents = segments
        .iter()
        .map(|s| crate::sketch_region::ArrangementRelation {
            output: s.edge.clone(),
            preserved: Vec::new(),
            modified: s.parents.clone(),
            generated: Vec::new(),
        })
        .collect();
    let result = capture_wire(&shape, &candidate.face, &parents, &mut 1);
    assert!(
        matches!(result,Err(CandidateCause::Incomplete(CandidateGap::IncompleteWire{omitted,..})) if omitted.len()==1)
    );
}
#[test]
fn candidate_pcurve_mutation_isolated_from_published_arrangement_and_input() {
    let source = cubic::closed_analytic_cubic(1., false);
    let owned = arranged(&[source.clone()]);
    let original = owned
        .relations
        .iter()
        .find(|r| r.output.shape_type().unwrap() == ShapeEnum::Edge)
        .unwrap()
        .output
        .clone();
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
    use rustycad_tkmath::{
        gp::{Transform3, Vector3},
        toploc::Location,
    };
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(3., 4., 5.));
    let source = cubic::closed_analytic_cubic(1., false)
        .located(&Location::new(transform), true)
        .unwrap();
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
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
    let mut roots = rectangle(0., 0., 4., 4.);
    roots.push(
        BRepBuilderAPIMakeEdge::from_points(Point3::new(-1., 2., 0.), Point3::new(5., 2., 0.))
            .unwrap()
            .edge()
            .unwrap()
            .into_shape(),
    );
    let owned = arranged(&roots);
    let original = generated_vertex_keys(&owned);
    assert!(!original.is_empty());
    let batch = super::extract_candidates(&owned, Plane3::default(), &mut |_| false)
        .unwrap_or_else(|cause| panic!("cut {}", candidate_problem(cause)));
    for key in original {
        assert!(
            batch
                .candidates
                .iter()
                .flat_map(|c| &c.relations)
                .any(|r| r.generated.contains(&key))
        );
        assert!(
            !batch.candidates.iter().flat_map(|c| &c.relations).any(|r| r
                .output
                .shape_type()
                .unwrap()
                == ShapeEnum::Vertex
                && r.modified.contains(&key))
        );
    }
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

fn candidate_problem(cause: CandidateCause) -> String {
    match cause {
        CandidateCause::Mapped(failure) => {
            format!("{} {}", failure.exception_type(), failure.message())
        }
        CandidateCause::MappedReport(report) => {
            let mut bytes = Vec::new();
            report
                .dump(&mut bytes)
                .unwrap();
            String::from_utf8(bytes).unwrap()
        }
        CandidateCause::Incomplete(CandidateGap::UnsupportedCurve { dynamic_type, .. }) => {
            format!("unsupported {dynamic_type}")
        }
        CandidateCause::Incomplete(CandidateGap::IncompleteWire {
            omitted,
            unexpected,
        }) => format!("omitted={} unexpected={}", omitted.len(), unexpected.len()),
        CandidateCause::Incomplete(CandidateGap::UnattributedMember { member }) => {
            format!("unattributed {:?}", member.shape_type().unwrap())
        }
        _ => "other cause".into(),
    }
}
