// SPDX-License-Identifier: GPL-2.0-only
use super::super::tests::{fixture, fixture_for, run};
use super::*;
use crate::sketch_region_partition_bindings::binding_sources;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, metric_bounds::*};
#[path = "tests/domain.rs"]
pub(in crate::sketch_region_prism) mod domain_tests;
#[path = "tests/mutation.rs"]
mod mutation_tests;
#[path = "tests/refusal.rs"]
mod refusal_tests;
#[path = "tests/replay.rs"]
mod replay_tests;

fn options() -> PrismOrientationOptions {
    let roots = RootIsolationOptions::new(1e-6);
    PrismOrientationOptions::v1(PrismEmbeddingOptions::v1(BezierRegularityOptions::v1(
        roots,
    )))
}
fn attempt(
    bridge: &PairedContactTransaction,
    shapes: &[Shape],
    receipt: &PrismReceipt,
    policy: PrismOrientationOptions,
    cancelled: impl FnMut() -> bool,
) -> Result<PrismOrientationEvidence, PrismOrientationFailure> {
    inspect_prism_orientation(bridge, &binding_sources(shapes), receipt, policy, cancelled)
}
fn oriented(
    bridge: &PairedContactTransaction,
    shapes: &[Shape],
    receipt: &PrismReceipt,
) -> PrismOrientationEvidence {
    attempt(bridge, shapes, receipt, options(), || false).unwrap_or_else(|error| {
        panic!(
            "orientation {:?}: {}",
            error.phase,
            cause_name(&error.cause)
        )
    })
}
fn cause_name(cause: &PrismOrientationCause) -> String {
    match cause {
        PrismOrientationCause::Face {
            domain,
            actual,
            expected,
        } => format!("face {:?}: {actual:?}; expected {expected:?}", domain.role),
        PrismOrientationCause::Trim {
            authored,
            actual,
            expected,
        } => format!(
            "trim {:?}: {actual:?}; expected {expected:?}",
            authored.role
        ),
        PrismOrientationCause::Embedding(cause) => format!("embedding {:?}", cause.phase),
        _ => "typed orientation refusal".into(),
    }
}
#[test]
fn owned_cubic_has_outward_faces_and_induced_trim_orientations() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let evidence = oriented(&bridge, &shapes, &receipt);
    assert_eq!(evidence.winding().factors(), (1, 1, 1, 1));
    assert_eq!(evidence.jacobian_sign(), 1);
    assert_eq!(evidence.requested(), options());
    check_expected_rows(&evidence);
    assert!(matches!(receipt.completion, PrismCompletion::Incomplete));
    assert!(receipt.limits.achieved_error.is_none());
}
fn check_expected_rows(evidence: &PrismOrientationEvidence) {
    use Orientation::{Forward as F, Reversed as R};
    let expected = [
        (PrismImageRole::First, R),
        (PrismImageRole::Last, F),
        (PrismImageRole::Generated, R),
    ];
    for (role, direction) in expected {
        let row = evidence
            .faces()
            .iter()
            .find(|r| r.domain().role == role)
            .unwrap();
        assert_eq!(row.orientations(), (direction, direction));
        assert!(!row.domain().parents.is_empty());
    }
    check_side_trims(&evidence.faces()[2]);
}
fn check_side_trims(side: &PrismFaceOrientation) {
    use Orientation::{Forward as F, Reversed as R};
    for (role, direction) in [
        (PrismTrimRole::First, F),
        (PrismTrimRole::Last, R),
        (PrismTrimRole::Start, R),
        (PrismTrimRole::End, F),
    ] {
        let row = side
            .trims()
            .iter()
            .find(|r| r.authored().role == role)
            .unwrap();
        assert_eq!(row.orientations(), (direction, direction));
    }
}
#[test]
fn owned_orientation_evidence_is_send_sync() {
    static_assertions::assert_impl_all!(PrismOrientationOptions: Send, Sync);
    static_assertions::assert_impl_all!(PrismOrientationEvidence: Send, Sync);
    static_assertions::assert_impl_all!(DerivedFromFreshFacedWinding: Send, Sync);
    static_assertions::assert_impl_all!(PrismFaceOrientation: Send, Sync);
    static_assertions::assert_impl_all!(PrismTrimOrientation: Send, Sync);
    static_assertions::assert_impl_all!(PrismOrientationPhase: Send, Sync);
    static_assertions::assert_impl_all!(PrismOrientationCause: Send, Sync);
    static_assertions::assert_impl_all!(PrismOrientationFailure: Send, Sync);
}

#[path = "tests/compatibility.rs"]
mod compatibility_tests;

/// Reuse existing actual geometry mutation; `mutate_orientation_fixture(target, shapes, receipt)`.
pub(in crate::sketch_region_prism) fn mutate_orientation_fixture(
    target: usize,
    shapes: &[Shape],
    receipt: &PrismReceipt,
) {
    super::super::measurements::tests::mutation_tests::edit_target(target, shapes, receipt);
}

#[path = "tests/evaluation.rs"]
mod evaluation_tests;
