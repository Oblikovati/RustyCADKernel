// SPDX-License-Identifier: GPL-2.0-only
//! Actual annular publication refusals: callback stops at every phase, child version refusals and
//! a history refusal are projected from the real operation; no detached header is constructed.
use super::*;
use crate::sketch_region_prism::frozen::annular_history::fixture::{
    AnnularHistoryCase, fresh_embedding,
};
use crate::sketch_region_prism::frozen::{
    annular_measures::AnnularMeasureCause, ordered_validation::annular::AnnularValidationCause,
};
use crate::sketch_region_prism::publication::annular::{
    AnnularPublicationRequest, publish_annular_prism, tests::options,
};
use crate::sketch_region_prism::publication::{
    PrismPublicationCheckpoint, PrismPublicationProgress,
};
use rustycad_tkernel::standard::Failure;

const _: () = {
    const fn owned<T: Send + Sync>() {}
    owned::<OwnedAnnularPublicationDiagnostic>();
};
struct Stop<F>(F);
impl<F: FnMut(PrismPublicationCheckpoint) -> Result<bool, Failure> + Send> PrismPublicationProgress
    for Stop<F>
{
    fn checkpoint(&mut self, point: PrismPublicationCheckpoint) -> Result<bool, Failure> {
        (self.0)(point)
    }
}
/// The owned diagnostic and whether the original Box had to be retained.
type Projected = (Box<OwnedAnnularPublicationDiagnostic>, bool);
fn refuse(
    case: &AnnularHistoryCase,
    requested: AnnularPublicationOptions,
    stop: impl FnMut(PrismPublicationCheckpoint) -> Result<bool, Failure> + Send,
) -> Projected {
    let mut progress = Stop(stop);
    case.with_region(|region| {
        let request = AnnularPublicationRequest {
            region,
            receipt: &case.receipt,
            bindings: &case.publication,
            copied: &case.names.copied,
            historical: &case.names.historical,
            requested,
        };
        let embedding = fresh_embedding(region, &case.receipt);
        let failure = publish_annular_prism(&request, embedding, &mut progress)
            .err()
            .expect("expected an annular publication refusal");
        let projection = project_annular_publication_refusal(failure);
        let retained = projection.original().is_some();
        (projection.into_diagnostic(), retained)
    })
}
fn complete((diagnostic, retained): Projected) -> Box<OwnedAnnularPublicationDiagnostic> {
    assert!(diagnostic.gaps().is_empty(), "{:?}", diagnostic.gaps());
    assert!(!retained, "a complete projection retains no original");
    diagnostic
}
#[test]
fn version_refusal_is_owned_with_independent_tail_bytes() {
    let mut requested = options();
    requested.version = 2;
    let diagnostic = complete(refuse(&AnnularHistoryCase::nested(), requested, |_| {
        Ok(false)
    }));
    let expected_cause = OwnedAnnularPublicationCause::UnsupportedVersion {
        actual: 2,
        expected: 1,
    };
    assert!(matches!(diagnostic.cause(), c if same_version(c, &expected_cause)));
    // Phase Admission (0), class Incomplete (1), cause tag 0, versions, zero gaps.
    let mut tail = vec![0, 1, 0];
    tail.extend([2u32, 1].map(u32::to_le_bytes).concat());
    tail.extend(0u64.to_le_bytes());
    assert!(diagnostic.replay_bytes().ends_with(&tail));
}
fn same_version(a: &OwnedAnnularPublicationCause, b: &OwnedAnnularPublicationCause) -> bool {
    use OwnedAnnularPublicationCause::UnsupportedVersion as V;
    matches!((a, b), (V { actual: x, expected: y }, V { actual: p, expected: q }) if (x, y) == (p, q))
}
/// Every interrupted child (join, validation, history, measures, commit) owns its stop.
#[test]
fn every_phase_callback_stop_is_owned_with_exact_checkpoint_and_class() {
    use PrismPublicationPhase::*;
    let case = AnnularHistoryCase::nested();
    for phase in [Join, Validation, History, Measures, Commit] {
        let diagnostic = complete(refuse(&case, options(), |p| Ok(p.phase == phase)));
        assert_eq!(
            (diagnostic.phase(), diagnostic.class()),
            (phase, PrismPublicationClass::Cancelled)
        );
        let OwnedAnnularPublicationCause::Callback { stop, .. } = diagnostic.cause() else {
            panic!("expected an owned callback stop at {phase:?}");
        };
        assert_eq!(stop.checkpoint.phase, phase);
        assert_eq!(stop.cause, OwnedDiagnosticCallbackCause::Cancelled);
        let fail = |p: PrismPublicationCheckpoint| match p.phase == phase {
            true => Err(Failure::new("fixture callback failure")),
            false => Ok(false),
        };
        let diagnostic = complete(refuse(&case, options(), fail));
        assert_eq!(diagnostic.class(), PrismPublicationClass::Failed);
        let OwnedAnnularPublicationCause::Callback { stop, .. } = diagnostic.cause() else {
            panic!("expected an owned native callback at {phase:?}");
        };
        let OwnedDiagnosticCallbackCause::Failed(native) = &stop.cause else {
            panic!("expected the native callback failure");
        };
        assert_eq!(native.message_bytes(), b"fixture callback failure");
    }
}
#[test]
fn child_version_refusals_move_the_actual_typed_child_failure() {
    let case = AnnularHistoryCase::nested();
    let mut requested = options();
    requested.validation.version = 99;
    let diagnostic = complete(refuse(&case, requested, |_| Ok(false)));
    let OwnedAnnularPublicationCause::Validation(child) = diagnostic.cause() else {
        panic!("expected the moved validation refusal");
    };
    let version = &child.failure().cause;
    assert!(matches!(
        version,
        AnnularValidationCause::Version {
            actual: 99,
            expected: 2
        }
    ));
    assert!(
        child
            .encoded()
            .starts_with(b"Martensite.AnnularValidationRefusal/v1\0")
    );
    let mut requested = options();
    requested.measures.version = 99;
    let diagnostic = complete(refuse(&case, requested, |_| Ok(false)));
    let OwnedAnnularPublicationCause::Measures(child) = diagnostic.cause() else {
        panic!("expected the moved measure refusal");
    };
    let version = &child.failure().cause;
    assert!(matches!(
        version,
        AnnularMeasureCause::Version {
            actual: 99,
            expected: 2
        }
    ));
    let mut requested = options();
    requested.join.version = 99;
    let diagnostic = complete(refuse(&case, requested, |_| Ok(false)));
    let OwnedAnnularPublicationCause::Join(join) = diagnostic.cause() else {
        panic!("expected the owned join refusal");
    };
    assert!(matches!(
        join.cause(),
        OwnedAnnularJoinCause::Version { actual: 99, .. }
    ));
}
#[test]
fn history_scope_refusal_moves_its_owned_finding() {
    let mut case = AnnularHistoryCase::nested();
    case.names.historical.scopes.pop();
    let diagnostic = complete(refuse(&case, options(), |_| Ok(false)));
    assert_eq!(diagnostic.phase(), PrismPublicationPhase::History);
    let OwnedAnnularPublicationCause::History(history) = diagnostic.cause() else {
        panic!("expected the moved history finding");
    };
    assert!(
        history
            .encoded()
            .starts_with(b"Martensite.PublicationHistoryDiagnostic/v1\0")
    );
}
#[test]
fn replay_is_identical_across_operations_and_threads() {
    let bytes = || {
        let mut requested = options();
        requested.measures.version = 99;
        let case = AnnularHistoryCase::nested();
        complete(refuse(&case, requested, |_| Ok(false))).replay_bytes()
    };
    let first = bytes();
    assert_eq!(first, bytes());
    assert_eq!(
        first,
        std::thread::spawn(bytes).join().expect("replay thread")
    );
}
/// A foreign embedding refuses inside a borrowed join family: the exact original is retained.
#[test]
fn foreign_embedding_keeps_the_original_beside_an_explicit_join_gap() {
    let foreign = AnnularHistoryCase::variant(Default::default(), true);
    let embedding = foreign.with_region(|region| fresh_embedding(region, &foreign.receipt));
    let case = AnnularHistoryCase::nested();
    let (diagnostic, retained, class) = case.with_region(|region| {
        let request = AnnularPublicationRequest {
            region,
            receipt: &case.receipt,
            bindings: &case.publication,
            copied: &case.names.copied,
            historical: &case.names.historical,
            requested: options(),
        };
        let failure = publish_annular_prism(&request, embedding, &mut Stop(|_| Ok(false)))
            .err()
            .expect("a foreign embedding must refuse");
        let class = failure.class;
        let projection = project_annular_publication_refusal(failure);
        let retained = projection.original().map(|original| original.class);
        (projection.into_diagnostic(), retained, class)
    });
    assert_eq!(retained, Some(class), "original retained unchanged");
    assert_eq!(diagnostic.class(), class);
    let [PublicationDiagnosticGap::Payload(payload)] = diagnostic.gaps() else {
        panic!("expected one payload gap, got {:?}", diagnostic.gaps());
    };
    let cause = diagnostic.cause();
    assert!(matches!(cause, OwnedAnnularPublicationCause::Unprojected(p) if p == payload));
}
