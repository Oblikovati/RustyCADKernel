// SPDX-License-Identifier: GPL-2.0-only
//! Stage order, verdicts and naming; full bodies are exercised through the kernel facade.
use super::ledger::Ledger;
use super::naming::TopologyNames;
use super::report::{
    CheckMode, Finding, FindingKind, Severity, StageVerdict, TopologyRef, ValidationStage,
};
use super::{ResolutionRejected, validate_body};
use crate::profile_extrusion::{DisplayTolerance, Resolution, ToleranceCap};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

fn finding(severity: Severity) -> Finding {
    let subject = TopologyRef {
        kind: ShapeEnum::Edge,
        index: Some(3),
    };
    Finding {
        severity,
        kind: FindingKind::FreeEdge,
        subject,
        context: None,
    }
}

#[test]
fn a_non_pass_blocks_every_later_stage() {
    let mut ledger = Ledger::new();
    assert!(ledger.record(ValidationStage::Topology, CheckMode::Exact, Vec::new()));
    let gap = vec![finding(Severity::Inconclusive)];
    assert!(!ledger.record(ValidationStage::Geometry, CheckMode::Sampled, gap));
    let report = ledger.finish(Vec::new(), Some(1e-3));
    let verdicts = report.stages.each_ref().map(|stage| stage.verdict);
    use StageVerdict::{Inconclusive, NotRun, Passed};
    assert_eq!(verdicts, [Passed, Inconclusive, NotRun, NotRun]);
    assert!(!report.is_valid());
    let first = report.first_non_pass().unwrap();
    assert_eq!(first.stage, ValidationStage::Geometry);
    assert_eq!(
        report.stage(ValidationStage::Tolerance).stage,
        ValidationStage::Tolerance
    );
}

#[test]
fn a_defect_fails_even_beside_gaps() {
    let mut ledger = Ledger::new();
    let mixed = vec![finding(Severity::Inconclusive), finding(Severity::Defect)];
    assert!(!ledger.record(ValidationStage::Topology, CheckMode::Exact, mixed));
    let report = ledger.finish(Vec::new(), None);
    assert_eq!(report.stages[0].verdict, StageVerdict::Failed);
}

#[test]
fn names_resolve_back_to_the_same_sub_shape() {
    let mut maker = BRepPrimAPIMakeBox::from_sizes(1.0, 1.0, 1.0).unwrap();
    let solid = maker.shape().unwrap().clone();
    let names = TopologyNames::of(&solid).unwrap();
    let mut explorer = Explorer::new(&solid, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    explorer.next().unwrap();
    let edge = explorer.current().clone();
    let name = names.name(&edge.reversed());
    assert_eq!(name.kind, ShapeEnum::Edge);
    assert!(name.resolve(&solid).unwrap().is_same(&edge));
    assert_eq!(names.all(ShapeEnum::Edge).len(), 12);
    assert_eq!(
        names.name(&solid),
        TopologyRef {
            kind: ShapeEnum::Solid,
            index: Some(0)
        }
    );
}

#[test]
fn an_invalid_resolution_is_rejected_with_its_field() {
    let mut maker = BRepPrimAPIMakeBox::from_sizes(1.0, 1.0, 1.0).unwrap();
    let display = DisplayTolerance {
        chordal: 1e-3,
        angular: 0.1,
    };
    let resolution = Resolution::new(1e-6, 1e-7, 1e-9, display, ToleranceCap::Report(-1.0));
    let rejected = validate_body(maker.shape().unwrap(), &resolution).unwrap_err();
    assert_eq!(
        rejected,
        ResolutionRejected {
            field: "cap",
            value: -1.0
        }
    );
}
