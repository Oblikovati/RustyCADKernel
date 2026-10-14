// SPDX-License-Identifier: GPL-2.0-only
use super::super::{annular_fixture::*, topology::*};
use super::*;
use rustycad_tkernel::standard::Failure;

pub(super) struct JoinProgress<F>(pub F);
impl<F: FnMut(AnnularFrozenPhase) -> Result<bool, Failure> + Send> AnnularFrozenProgress
    for JoinProgress<F>
{
    fn checkpoint(&mut self, phase: AnnularFrozenPhase) -> Result<bool, Failure> {
        (self.0)(phase)
    }
}
struct ContinueTopology;
impl TopologyProgress for ContinueTopology {
    fn checkpoint(&mut self, _: TopologyCheckpoint) -> Result<bool, Failure> {
        Ok(false)
    }
}
pub(super) fn with_named<T>(
    run: impl FnOnce(
        &TwoLoopRegionRequest<'_, '_>,
        &mut PrismReceipt,
        &mut PrismPublicationBindings,
        AnnularEmbeddingEvidence,
    ) -> T,
) -> T {
    with_bound(|request, receipt, names, binding| {
        let proof = inspect_bound_annular_embedding(
            request,
            receipt,
            binding,
            tests::embedding_options(),
            || false,
        )
        .unwrap_or_else(|_| panic!("actual embedding"));
        run(request, receipt, names, proof)
    })
}
pub(in crate::sketch_region_prism) fn joined() -> AnnularFrozenDomain {
    with_named(|request, receipt, names, proof| {
        inspect_default(
            request,
            receipt,
            names,
            proof,
            &mut JoinProgress(|_| Ok(false)),
        )
        .unwrap_or_else(|failure| {
            panic!(
                "actual join refused at {:?}: {:?}",
                failure.phase,
                failure.classification()
            )
        })
    })
}
#[test]
fn actual_annular_join_and_parameter_domain_topology_leave_body_validation_incomplete() {
    let joined = joined();
    let AnnularTopologyOutcome::Certified(topology) =
        certify_annular_frozen_topology(&joined, &mut ContinueTopology)
    else {
        panic!("actual annular topology")
    };
    assert_topology_binding(&topology, &joined);
    assert!(!joined.projection().is_valid_body());
    assert!(matches!(
        certify_frozen_topology(joined.projection(), &mut ContinueTopology),
        FrozenTopologyOutcome::Failed(_)
    ));
}
#[test]
fn both_coarse_checkpoints_cancel_without_a_published_candidate() {
    for cutoff in [
        AnnularFrozenPhase::CurrentBinding,
        AnnularFrozenPhase::Commit,
    ] {
        with_named(|request, receipt, names, proof| {
            let mut calls = Vec::new();
            let result = inspect_default(
                request,
                receipt,
                names,
                proof,
                &mut JoinProgress(|phase| {
                    calls.push(phase);
                    Ok(phase == cutoff)
                }),
            );
            assert_cancelled(result, cutoff, &calls, receipt);
        });
    }
}
#[test]
fn annular_join_rebind_detects_same_revision_source_mutation_and_retains_owned_bytes() {
    with_named(|request, receipt, names, proof| {
        let joined = inspect_default(
            request,
            receipt,
            names,
            proof,
            &mut JoinProgress(|_| Ok(false)),
        )
        .unwrap_or_else(|_| panic!("join"));
        let before = joined.replay_bytes().to_vec();
        assert!(rebind_annular_frozen_domain(request, receipt, names, &joined).is_ok());
        let edge = rustycad_tkbrep::topods::edge(request.sources[0].shape).unwrap();
        crate::sketch_region_prism::isolation_tests::set_pole(&edge, 5.);
        let failure = rebind_annular_frozen_domain(request, receipt, names, &joined)
            .err()
            .unwrap();
        assert_eq!(failure.phase, AnnularFrozenPhase::Commit);
        assert!(matches!(failure.cause, AnnularFrozenCause::Embedding(_)));
        assert_eq!(joined.replay_bytes(), before);
    });
}
#[test]
fn commit_alias_mutation_is_refused_after_the_final_callback() {
    with_named(|request, receipt, names, proof| {
        let edge = rustycad_tkbrep::topods::edge(request.sources[0].shape).unwrap();
        let mut progress = JoinProgress(|phase| {
            if phase == AnnularFrozenPhase::Commit {
                crate::sketch_region_prism::isolation_tests::set_pole(&edge, 5.);
            }
            Ok(false)
        });
        let failure = inspect_default(request, receipt, names, proof, &mut progress)
            .err()
            .unwrap();
        assert_eq!(failure.phase, AnnularFrozenPhase::Commit);
        assert!(matches!(failure.cause, AnnularFrozenCause::Embedding(_)));
    });
}
#[test]
fn name_budget_and_version_admission_precede_external_callbacks() {
    for options in inadmissible_options() {
        with_named(|request, receipt, names, proof| {
            let failure = join_annular_frozen_domain(
                request,
                receipt,
                names,
                proof,
                options,
                &mut JoinProgress(|_| panic!("admit before callback")),
            )
            .err()
            .unwrap();
            assert_eq!(failure.phase, AnnularFrozenPhase::Admission);
            assert_eq!(failure.requested, options);
            assert_eq!(failure.classification(), TwoLoopRefusalClass::Incomplete);
        });
    }
}

pub(super) fn inspect_default<'r>(
    request: &TwoLoopRegionRequest<'r, '_>,
    receipt: &PrismReceipt,
    names: &PrismPublicationBindings,
    proof: AnnularEmbeddingEvidence,
    progress: &mut dyn AnnularFrozenProgress,
) -> Result<AnnularFrozenDomain, Box<AnnularFrozenFailure<'r>>> {
    join_annular_frozen_domain(request, receipt, names, proof, tests::options(), progress)
}

fn assert_topology_binding(topology: &AnnularTopologyCertificate, joined: &AnnularFrozenDomain) {
    assert!(topology.binds(joined));
    assert_eq!(topology.counts(), [4, 6, 12, 6, 4, 1, 1]);
    assert_eq!(topology.operation(), AuthoredPrismId(77));
    assert_eq!(topology.root().operation, topology.operation());
    assert_eq!(topology.source_bytes(), joined.replay_bytes());
    assert_eq!(
        topology.evidence_kind(),
        TopologyEvidenceKind::ExactCombinatorial
    );
    assert_eq!(
        topology.numerical_error(),
        TopologyNumericalError::NotApplicable
    );
}

fn inadmissible_options() -> [AnnularFrozenOptions; 2] {
    [
        AnnularFrozenOptions {
            maximum_name_units: 1,
            ..tests::options()
        },
        AnnularFrozenOptions {
            version: 9,
            ..tests::options()
        },
    ]
}

fn assert_cancelled(
    result: Result<AnnularFrozenDomain, Box<AnnularFrozenFailure<'_>>>,
    cutoff: AnnularFrozenPhase,
    calls: &[AnnularFrozenPhase],
    receipt: &PrismReceipt,
) {
    let failure = result.err().unwrap();
    assert_eq!(failure.phase, cutoff);
    assert_eq!(failure.classification(), TwoLoopRefusalClass::Cancelled);
    assert_eq!(calls.last(), Some(&cutoff));
    assert!(super::super::admission::admit_receipt(receipt).is_ok());
}
