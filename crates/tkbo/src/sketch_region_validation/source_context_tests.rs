// SPDX-License-Identifier: GPL-2.0-only
//! Real mapped context faults retain separate occurrence attribution and typed causes.
use super::inspection::inspect_candidate;
use super::outcome::{
    BodyKind, StageCoverage, ValidationCause, ValidationFinding, ValidationStage,
};
use super::source_relation_test_support::{member_identity, test_source_relation};
use super::source_relations::SourceRelation;
use rustycad_tkbrep::{topexp::Explorer, topods::Shape};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;

fn missing_context_findings(include_context: bool) -> (Shape, Shape, Vec<ValidationFinding>) {
    let face = super::tests::box_face();
    let edge = Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .current()
        .clone();
    let analyzer = BRepCheckAnalyzer::new(&face, false, false, true).unwrap();
    analyzer.result(&edge).unwrap().init(&edge).unwrap();
    let relations = context_relations(&edge, &face, include_context);
    let findings = inspect_candidate(
        &analyzer,
        &face,
        BodyKind::PlanarRegion,
        &relations,
        ValidationStage::Topology,
    );
    (face, edge, findings)
}

#[test]
fn actual_checker_fault_keeps_affected_and_context_members_separate() {
    let (face, edge, findings) = missing_context_findings(true);
    let finding = findings.first().unwrap();
    assert_eq!(finding.coverage, StageCoverage::Incomplete);
    assert!(matches!(finding.cause, ValidationCause::MissingContext));
    let affected = finding.affected.original_parents.as_ref().unwrap();
    let context = finding.context.as_ref().unwrap();
    let context = context.original_parents.as_ref().unwrap();
    assert_eq!(affected.modified, vec![member_identity(7, &edge)]);
    assert_eq!(context.generated, vec![member_identity(7, &face)]);
    assert!(context.modified.is_empty());
    assert_ne!(
        affected.modified[0].topology_address,
        context.generated[0].topology_address
    );
}

#[test]
fn missing_context_relation_does_not_inherit_affected_parents() {
    let (_, _, findings) = missing_context_findings(false);
    let finding = findings.first().unwrap();
    assert!(finding.affected.original_parents.is_some());
    let context = finding.context.as_ref().unwrap();
    assert!(context.original_parents.is_none());
    assert!(context.source_parents.is_none());
}

#[test]
fn typed_validation_failure_preserves_cause_and_original_member() {
    let output = super::source_relation_test_support::actual_vertex();
    let relations: [SourceRelation; 1] = [test_source_relation(&output, &[7])];
    let failure = rustycad_tkernel::standard::Failure::with_message(
        "Standard_DomainError",
        "value=7; expected valid domain",
    );
    let finding =
        super::inspection::native_failure(&output, &relations, ValidationStage::Geometry, failure);
    let ValidationCause::NativeFailure(cause) = finding.cause else {
        panic!("expected typed failure")
    };
    assert_eq!(cause.exception_type(), "Standard_DomainError");
    assert_eq!(
        finding.affected.original_parents.unwrap().modified,
        vec![member_identity(7, &output)]
    );
    assert!(finding.context.is_none());
}

fn context_relations(edge: &Shape, face: &Shape, include_context: bool) -> Vec<SourceRelation> {
    let mut relations = vec![test_source_relation(edge, &[7])];
    if include_context {
        relations.push(generated_context(face));
    }
    relations
}

fn generated_context(face: &Shape) -> SourceRelation {
    let mut context = test_source_relation(face, &[7]);
    context.parents.generated = std::mem::take(&mut context.parents.modified);
    context
}
