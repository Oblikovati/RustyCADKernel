// SPDX-License-Identifier: GPL-2.0-only
//! Named scalar fixture supplies authored identities; no topology address becomes result identity.
use super::*;
use crate::sketch_region::{OriginalMemberIdentity, RegionSourceId};
use crate::sketch_region_candidates::CandidateCause;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, metric_bounds::*};
use static_assertions::assert_impl_all;

assert_impl_all!(OwnedPrismJoinRefusal: Send, Sync);
assert_impl_all!(OwnedPrismJoinCause: Send, Sync);
assert_impl_all!(OwnedPrismEmbeddingFailure: Send, Sync);
assert_impl_all!(OwnedPrismEmbeddingCause: Send, Sync);
assert_impl_all!(OwnedPrismBindingCause: Send, Sync);
assert_impl_all!(OwnedPrismBodyCause: Send, Sync);
assert_impl_all!(OwnedPrismSweepCause: Send, Sync);
assert_impl_all!(OwnedPrismRegionCause: Send, Sync);
assert_impl_all!(OwnedPrismSourceCause: Send, Sync);
assert_impl_all!(OwnedPrismCandidateCause: Send, Sync);
assert_impl_all!(PrismRefusalProjectionIssue: Send, Sync);

assert_impl_all!(PrismRefusalClass: Send, Sync);
assert_impl_all!(PrismRefusalPath: Send, Sync);
assert_impl_all!(PrismRefusalProjectionGap: Send, Sync);
assert_impl_all!(PendingPrismRefusalPayload: Send, Sync);

struct NamedRefusalFixture {
    bindings: PrismPublicationBindings,
}
impl NamedRefusalFixture {
    fn new() -> Self {
        Self {
            bindings: PrismPublicationBindings {
                operation: AuthoredPrismId(401),
                selected_region: AuthoredPrismId(402),
                originals: vec![PrismOriginalBinding {
                    actual: original(0xdeadbeef),
                    member: AuthoredPrismId(403),
                    directed_use: AuthoredPrismId(404),
                }],
                sources: vec![],
                directed: vec![],
            },
        }
    }
    fn run(
        &self,
        cause: PrismProofJoinCause,
        class: PrismRefusalClass,
    ) -> PrismJoinRefusalProjection {
        let failure = PrismProofJoinFailure {
            operation: AuthoredPrismId(401),
            phase: PrismProofJoinPhase::Embedding,
            entity: None,
            cause,
        };
        let context = PrismRefusalIdentityContext {
            bindings: &self.bindings,
        };
        project_join_refusal(failure, class, &context)
    }
    fn projected(&self, cause: PrismProofJoinCause) -> OwnedPrismJoinRefusal {
        match self.run(cause, PrismRefusalClass::Failed) {
            PrismJoinRefusalProjection::Projected(value) => *value,
            PrismJoinRefusalProjection::Incomplete(_) => {
                panic!("expected complete diagnostic projection")
            }
        }
    }
}
fn original(address: usize) -> OriginalMemberIdentity {
    OriginalMemberIdentity {
        source: RegionSourceId(17),
        topology_address: address,
        location: Default::default(),
        orientation: Orientation::Forward,
    }
}
fn native() -> Failure {
    let cause = Failure::with_type_bytes("Standard_DomainError", &[255, 65], &[254, 66]);
    cause.with_exception_parent("Standard_Failure")
}
fn missing_original(address: usize) -> PrismProofJoinCause {
    let cause = SourceConformanceCause::MissingOriginal(original(address));
    PrismProofJoinCause::ClosureQuery(Box::new(cause))
}
fn retained(result: PrismJoinRefusalProjection) -> LocalPrismProjectionFailure {
    match result {
        PrismJoinRefusalProjection::Incomplete(value) => *value,
        PrismJoinRefusalProjection::Projected(_) => {
            panic!("expected explicit projection incompleteness")
        }
    }
}

#[test]
fn authored_original_projection_removes_process_address_and_survives_context_drop() {
    let fixture = NamedRefusalFixture::new();
    let result = fixture.projected(missing_original(0xdeadbeef));
    drop(fixture);
    assert_eq!(result.version(), 1);
    assert_eq!(result.operation(), AuthoredPrismId(401));
    assert_eq!(result.phase(), PrismProofJoinPhase::Embedding);
    assert_eq!(result.class(), PrismRefusalClass::Failed);
    assert_eq!(result.entity(), None);
    assert!(
        matches!(result.cause(), OwnedPrismJoinCause::ClosureQuery(cause)
        if matches!(**cause, OwnedPrismSourceCause::MissingOriginal(FrozenOriginalKey {source:17, member:AuthoredPrismId(403), directed_use:AuthoredPrismId(404)})))
    );
    assert!(!format!("{result:?}").contains("3735928559"));
}

#[test]
fn missing_original_returns_unchanged_local_identity_and_operation_class() {
    let fixture = NamedRefusalFixture::new();
    let error = retained(fixture.run(missing_original(99), PrismRefusalClass::Cancelled));
    assert!(matches!(
        error.issue().cause(),
        PrismRefusalProjectionGap::Original { source: 17, .. }
    ));
    assert_eq!(
        error.issue().path(),
        [PrismRefusalPath::Join, PrismRefusalPath::Source]
    );
    let (class, original) = error.into_original();
    assert_eq!(class, PrismRefusalClass::Cancelled);
    assert!(
        matches!(original.cause, PrismProofJoinCause::ClosureQuery(cause)
        if matches!(&*cause, SourceConformanceCause::MissingOriginal(value) if value.topology_address == 99))
    );
}

#[test]
fn ambiguous_original_never_selects_first_binding() {
    let mut fixture = NamedRefusalFixture::new();
    fixture.bindings.originals.push(PrismOriginalBinding {
        actual: original(0xdeadbeef),
        member: AuthoredPrismId(500),
        directed_use: AuthoredPrismId(501),
    });
    let error = retained(fixture.run(missing_original(0xdeadbeef), PrismRefusalClass::Failed));
    assert!(matches!(
        error.issue().cause(),
        PrismRefusalProjectionGap::Original { .. }
    ));
    let (_, local) = error.into_original();
    assert!(matches!(local.cause, PrismProofJoinCause::ClosureQuery(_)));
}

#[test]
fn wrong_operation_context_returns_original_before_projection() {
    let mut fixture = NamedRefusalFixture::new();
    fixture.bindings.operation = AuthoredPrismId(777);
    let error = retained(fixture.run(
        PrismProofJoinCause::MissingClosureQuery,
        PrismRefusalClass::Incomplete,
    ));
    assert!(matches!(
        error.issue().cause(),
        PrismRefusalProjectionGap::Operation {
            actual: AuthoredPrismId(777),
            expected: AuthoredPrismId(401)
        }
    ));
    assert!(matches!(
        error.into_original().1.cause,
        PrismProofJoinCause::MissingClosureQuery
    ));
}

#[test]
fn nested_native_bytes_and_parent_survive_projection_exactly() {
    let cause = body_refusal(PrismCause::Mapped {
        phase: PrismPhase::Sweep,
        failure: native(),
    });
    let result = NamedRefusalFixture::new().projected(cause);
    let (phase, failure) = mapped_native(&result);
    assert_eq!(*phase, PrismPhase::Sweep);
    assert_eq!(*failure, native());
    assert_eq!(
        (failure.message_bytes(), failure.stack_bytes()),
        (&[255, 65][..], &[254, 66][..])
    );
}

#[test]
fn pending_report_keeps_same_report_owner_and_unmodified_failure_class() {
    let report = std::sync::Arc::new(rustycad_tkernel::message::Report::new());
    let cause =
        PrismBindingFailure::Classification(Box::new(CandidateCause::MappedReport(report.clone())));
    let error = retained(NamedRefusalFixture::new().run(
        PrismProofJoinCause::Binding(Box::new(cause)),
        PrismRefusalClass::Failed,
    ));
    assert!(matches!(
        error.issue().cause(),
        PrismRefusalProjectionGap::Pending(PendingPrismRefusalPayload::CandidateReport)
    ));
    let (class, local) = error.into_original();
    assert_eq!(class, PrismRefusalClass::Failed);
    let retained = retained_report(local);
    assert!(std::sync::Arc::ptr_eq(&retained, &report));
}

#[test]
fn exact_residual_failure_retains_offending_scalar_bits() {
    let cause = SourceConformanceCause::Exact(ResidualError::Value {
        role: "weight",
        index: 3,
        value: -0.0,
    });
    let result =
        NamedRefusalFixture::new().projected(PrismProofJoinCause::ClosureQuery(Box::new(cause)));
    let OwnedPrismJoinCause::ClosureQuery(cause) = result.cause() else {
        panic!("expected source")
    };
    assert!(
        matches!(&**cause, OwnedPrismSourceCause::Exact(ResidualError::Value { role:"weight", index:3, value }) if value.to_bits() == (-0.0f64).to_bits())
    );
}

fn embedding_refusal(cause: PrismEmbeddingCause) -> PrismProofJoinCause {
    let requested =
        PrismEmbeddingOptions::v1(BezierRegularityOptions::v1(RootIsolationOptions::new(1e-6)));
    PrismProofJoinCause::Embedding(Box::new(PrismEmbeddingFailure {
        requested,
        phase: PrismEmbeddingPhase::Regularity,
        original: Some(Box::new(original(0xdeadbeef))),
        cause,
    }))
}

#[test]
fn embedding_keeps_weights_options_phase_and_authored_original() {
    let cause = embedding_refusal(PrismEmbeddingCause::PolynomialRequired {
        curve_id: 7,
        explicit_weights: vec![1.0, -0.0],
    });
    let result = NamedRefusalFixture::new().projected(cause);
    let OwnedPrismJoinCause::Embedding(value) = result.cause() else {
        panic!("expected embedding")
    };
    assert_eq!(value.phase, PrismEmbeddingPhase::Regularity);
    assert_eq!(value.requested.regularity.roots.max_width, 1e-6);
    assert_eq!(
        value.original.as_ref().unwrap().member,
        AuthoredPrismId(403)
    );
    assert!(
        matches!(&value.cause, OwnedPrismEmbeddingCause::PolynomialRequired {curve_id:7,explicit_weights} if explicit_weights[1].to_bits() == (-0.0f64).to_bits())
    );
}

#[test]
fn pending_nested_embedding_reconstructs_complete_original() {
    let cause = pending_embedding();
    let error = retained(NamedRefusalFixture::new().run(cause, PrismRefusalClass::Incomplete));
    let (class, local) = error.into_original();
    assert_eq!(class, PrismRefusalClass::Incomplete);
    let PrismProofJoinCause::Embedding(value) = local.cause else {
        panic!("expected embedding")
    };
    assert_eq!(value.original.unwrap().topology_address, 0xdeadbeef);
    assert_eq!(value.phase, PrismEmbeddingPhase::Regularity);
    let PrismEmbeddingCause::Binding(binding) = value.cause else {
        panic!("expected binding")
    };
    let PrismBindingFailure::Classification(candidate) = *binding else {
        panic!("expected candidate")
    };
    assert!(matches!(*candidate, CandidateCause::MappedReport(_)));
}

#[test]
fn zero_original_token_and_reverse_collision_are_explicitly_incomplete() {
    let mut fixture = NamedRefusalFixture::new();
    fixture.bindings.originals[0].member = AuthoredPrismId(0);
    assert!(matches!(
        fixture.run(missing_original(0xdeadbeef), PrismRefusalClass::Failed),
        PrismJoinRefusalProjection::Incomplete(_)
    ));
    fixture.bindings.originals[0].member = AuthoredPrismId(403);
    fixture.bindings.originals.push(PrismOriginalBinding {
        actual: original(99),
        member: AuthoredPrismId(403),
        directed_use: AuthoredPrismId(404),
    });
    assert!(matches!(
        fixture.run(missing_original(0xdeadbeef), PrismRefusalClass::Failed),
        PrismJoinRefusalProjection::Incomplete(_)
    ));
}

#[test]
fn successful_projection_keeps_each_supplied_operation_class() {
    for class in [
        PrismRefusalClass::Failed,
        PrismRefusalClass::Incomplete,
        PrismRefusalClass::Cancelled,
    ] {
        let result =
            NamedRefusalFixture::new().run(PrismProofJoinCause::MissingClosureQuery, class);
        let PrismJoinRefusalProjection::Projected(result) = result else {
            panic!("expected projection")
        };
        assert_eq!(result.class(), class);
    }
}

fn numerical_refusal() -> PrismProofJoinCause {
    let poles = [[1., 0., 0.], [-2., 0., 0.], [4., 0., 0.]];
    let mut options = BezierRegularityOptions::v1(RootIsolationOptions::new(1e-6));
    options.roots.max_bisections = 0;
    let query = BezierRegularityQuery::new(7, &poles, options);
    let failure = certify_bezier_regularity(query, &mut || false).unwrap_err();
    embedding_refusal(PrismEmbeddingCause::Regularity(Box::new(failure)))
}

#[test]
fn failed_root_attempt_keeps_coefficients_options_and_actual_work() {
    let projected = NamedRefusalFixture::new().projected(numerical_refusal());
    let OwnedPrismJoinCause::Embedding(value) = projected.cause() else {
        panic!("expected embedding")
    };
    let OwnedPrismEmbeddingCause::Regularity(failure) = &value.cause else {
        panic!("expected numerical failure")
    };
    assert_eq!(failure.source().options().roots.max_bisections, 0);
    assert_eq!(failure.bisections(), 0);
    assert_eq!(failure.completion(), BezierRegularityCompletion::Incomplete);
    assert!(matches!(
        failure.cause(),
        BezierRegularityCause::Roots { .. }
    ));
}

#[test]
fn candidate_native_and_callback_companion_are_both_retained() {
    let cause = callback_refusal();
    let result = NamedRefusalFixture::new().projected(cause);
    let OwnedPrismJoinCause::Binding(value) = result.cause() else {
        panic!("expected binding")
    };
    let OwnedPrismBindingCause::Body(OwnedPrismBodyCause::Failed(
        OwnedPrismSweepCause::ClassificationProgress { failure, query },
    )) = &**value
    else {
        panic!("expected compound")
    };
    assert_eq!(failure.exception_type(), "Callback");
    assert!(matches!(&**query,OwnedPrismCandidateCause::Native(value) if value == &native()));
}

#[test]
fn immutable_readers_and_independent_threads_observe_identical_refusals() {
    let fixture = NamedRefusalFixture::new();
    let shared = std::sync::Arc::new(fixture.projected(numerical_refusal()));
    let expected = format!("{shared:?}");
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let shared = shared.clone();
            std::thread::spawn(move || thread_receipts(shared))
        })
        .collect();
    for thread in threads {
        let (shared, independent) = thread.join().unwrap();
        assert_eq!(shared, expected);
        assert_eq!(independent, expected);
    }
}

#[path = "case_tests.rs"]
mod case_tests;

fn body_refusal(cause: PrismCause) -> PrismProofJoinCause {
    let body = PrismBindingFailure::Body(PrismCompletion::Failed(cause));
    PrismProofJoinCause::Binding(Box::new(body))
}
fn callback_refusal() -> PrismProofJoinCause {
    let query = CandidateCause::Mapped(native());
    body_refusal(PrismCause::ClassificationProgress {
        failure: Box::new(Failure::with_type("Callback", "stop", "")),
        query: Box::new(query),
    })
}
fn retained_report(
    local: PrismProofJoinFailure,
) -> std::sync::Arc<rustycad_tkernel::message::Report> {
    let PrismProofJoinCause::Binding(binding) = local.cause else {
        panic!("expected binding")
    };
    let PrismBindingFailure::Classification(candidate) = *binding else {
        panic!("expected candidate")
    };
    let CandidateCause::MappedReport(retained) = *candidate else {
        panic!("expected report")
    };
    retained
}
fn pending_embedding() -> PrismProofJoinCause {
    let report = std::sync::Arc::new(rustycad_tkernel::message::Report::new());
    let cause = PrismBindingFailure::Classification(Box::new(CandidateCause::MappedReport(report)));
    embedding_refusal(PrismEmbeddingCause::Binding(Box::new(cause)))
}
fn thread_receipts(shared: std::sync::Arc<OwnedPrismJoinRefusal>) -> (String, String) {
    let independent = NamedRefusalFixture::new().projected(numerical_refusal());
    (format!("{shared:?}"), format!("{independent:?}"))
}

fn mapped_native(result: &OwnedPrismJoinRefusal) -> (&PrismPhase, &Failure) {
    let OwnedPrismJoinCause::Binding(binding) = result.cause() else {
        panic!("expected binding")
    };
    let OwnedPrismBindingCause::Body(OwnedPrismBodyCause::Failed(OwnedPrismSweepCause::Mapped {
        phase,
        failure,
    })) = &**binding
    else {
        panic!("expected typed native cause")
    };
    (phase, failure)
}
