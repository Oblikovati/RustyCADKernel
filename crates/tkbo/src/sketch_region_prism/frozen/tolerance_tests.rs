// SPDX-License-Identifier: GPL-2.0-only
//! Owned raw metadata distinguishes values hidden by the compatibility floor.
use super::{tests::*, *};
use rustycad_tkbrep::{
    brep::access::{edge_tshape, face_tshape, vertex_tshape},
    topods::Shape,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

fn actual_entity(receipt: &super::super::PrismReceipt, kind: ShapeEnum) -> Shape {
    super::super::contexts::body_contexts(&receipt.diagnostic.as_ref().unwrap().body)
        .ok()
        .unwrap()
        .into_iter()
        .find(|context| context.occurrence.shape_type().ok() == Some(kind))
        .unwrap()
        .occurrence
}
fn set_raw(actual: &Shape, value: f64) {
    match actual.shape_type().unwrap() {
        ShapeEnum::Vertex => vertex_tshape(actual, "owned fixture")
            .unwrap()
            .set_tolerance(value),
        ShapeEnum::Edge => edge_tshape(actual, "owned fixture")
            .unwrap()
            .set_tolerance(value),
        ShapeEnum::Face => face_tshape(actual, "owned fixture")
            .unwrap()
            .set_tolerance(value),
        _ => panic!("expected authored vertex, edge, or face"),
    }
}
fn capture_with_raw(kind: ShapeEnum, raw: f64) -> FrozenPrismProjection {
    let (source, receipt) = authored_receipt();
    let bindings = authored_bindings(&receipt, &source);
    set_raw(&actual_entity(&receipt, kind), raw);
    captured(freeze_prism(&receipt, &bindings, &mut NamedFreezeProgress))
}
#[test]
fn zero_and_subfloor_raw_bytes_survive_the_same_effective_floor() {
    for kind in [ShapeEnum::Vertex, ShapeEnum::Edge, ShapeEnum::Face] {
        let zero = capture_with_raw(kind, 0.);
        let subfloor = capture_with_raw(kind, 5e-8);
        assert_ne!(zero.replay_bytes(), subfloor.replay_bytes());
        for (snapshot, raw) in [(&zero, 0.), (&subfloor, 5e-8)] {
            let node = snapshot
                .nodes()
                .iter()
                .find(|n| n.raw_tolerance == Some(raw))
                .unwrap();
            assert_eq!(node.effective_tolerance, Some(1e-7));
            assert_eq!(node.raw_tolerance.unwrap().to_bits(), raw.to_bits());
        }
    }
}
fn assert_invalid_raw(kind: ShapeEnum, raw: f64) {
    let (source, receipt) = authored_receipt();
    let bindings = authored_bindings(&receipt, &source);
    let actual = actual_entity(&receipt, kind);
    let expected = super::error_context_tests::actual_key(&receipt, &bindings, &actual);
    set_raw(&actual, raw);
    let outcome = freeze_prism(&receipt, &bindings, &mut NamedFreezeProgress);
    super::error_context_tests::assert_raw_context(outcome, expected, raw);
}
#[test]
fn invalid_raw_values_are_refused_before_the_compatibility_floor() {
    for kind in [ShapeEnum::Vertex, ShapeEnum::Edge, ShapeEnum::Face] {
        for raw in [f64::NAN, -1., f64::INFINITY, f64::NEG_INFINITY] {
            assert_invalid_raw(kind, raw);
        }
    }
}
struct ChangeRawAtCommit {
    actual: Shape,
}
impl FreezeProgress for ChangeRawAtCommit {
    fn checkpoint(&mut self, phase: FreezePhase) -> Result<bool, Failure> {
        if phase == FreezePhase::Commit {
            set_raw(&self.actual, 5e-8);
        }
        Ok(false)
    }
}
fn assert_commit_metadata_mutation(kind: ShapeEnum) {
    let (source, receipt) = authored_receipt();
    let bindings = authored_bindings(&receipt, &source);
    let actual = actual_entity(&receipt, kind);
    set_raw(&actual, 0.);
    let before = captured(freeze_prism(&receipt, &bindings, &mut NamedFreezeProgress));
    let bytes = before.replay_bytes();
    let outcome = freeze_prism(&receipt, &bindings, &mut ChangeRawAtCommit { actual });
    assert!(matches!(
        outcome,
        PrismFreezeOutcome::Incomplete(PrismFreezeFailure {
            cause: PrismFreezeCause::StaleCapture { .. },
            ..
        })
    ));
    assert_eq!(before.replay_bytes(), bytes);
}
#[test]
fn commit_subfloor_mutations_are_detected_without_mutating_owned_results() {
    for kind in [ShapeEnum::Vertex, ShapeEnum::Edge, ShapeEnum::Face] {
        assert_commit_metadata_mutation(kind);
    }
}
