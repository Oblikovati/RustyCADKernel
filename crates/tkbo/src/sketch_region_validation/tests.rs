// SPDX-License-Identifier: GPL-2.0-only
//! Real mapped checker reuse exposes absent context without changing its behavior.
use super::inspection::inspect_candidate;
use super::outcome::{BodyKind, StageCoverage, ValidationStage};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;

pub(super) fn box_face() -> Shape {
    let mut maker = BRepPrimAPIMakeBox::from_sizes(1.0, 1.0, 1.0).unwrap();
    let shape = maker.shape().unwrap();
    Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .current()
        .clone()
}

#[test]
fn required_missing_context_is_incomplete_despite_mapped_validity() {
    let face = box_face();
    let analyzer = BRepCheckAnalyzer::new(&face, false, false, true).unwrap();
    let edge = first_edge(&face);
    let checker = analyzer.result(&edge).unwrap();
    assert!(checker.is_status_on_shape(&face));
    checker.init(&edge).unwrap();
    assert!(!checker.is_status_on_shape(&face));
    assert!(analyzer.is_valid().unwrap());
    let findings = inspect_candidate(
        &analyzer,
        &face,
        BodyKind::PlanarRegion,
        &[],
        ValidationStage::Topology,
    );
    check_missing_relation(&findings, &edge, &face);
}

fn first_edge(face: &Shape) -> Shape {
    Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .current()
        .clone()
}

fn check_missing_relation(
    findings: &[super::outcome::ValidationFinding],
    edge: &Shape,
    face: &Shape,
) {
    assert_eq!(findings.len(), 1);
    let finding = &findings[0];
    assert_eq!(finding.coverage, StageCoverage::Incomplete);
    assert!(matches!(
        finding.cause,
        super::outcome::ValidationCause::MissingContext
    ));
    assert!(finding.affected.shape.is_equal(edge));
    assert!(finding.context.as_ref().unwrap().shape.is_equal(face));
}
