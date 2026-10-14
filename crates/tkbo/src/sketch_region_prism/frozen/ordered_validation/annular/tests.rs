// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_prism::frozen::annular_join::operation_tests::joined;
use rustycad_tkernel::standard::Failure;
#[derive(Default)]
pub(super) struct RecordedAnnularProgress {
    pub points: Vec<AnnularValidationCheckpoint>,
    pub stop: Option<usize>,
    pub fail: bool,
}
impl AnnularValidationProgress for RecordedAnnularProgress {
    fn checkpoint(&mut self, point: AnnularValidationCheckpoint) -> Result<bool, Failure> {
        self.points.push(point);
        let stop = self.stop == Some(point.ordinal);
        if stop && self.fail {
            return Err(Failure::new("injected annular validator native failure"));
        }
        Ok(stop)
    }
}
pub(super) fn options() -> AnnularValidationOptions {
    AnnularValidationOptions {
        version: 2,
        stored_cap: StoredToleranceCap::refuse(1.).unwrap(),
    }
}
pub(super) fn complete(join: &AnnularFrozenDomain) -> AnnularValidationCertificate {
    validate_annular_prism(join, options(), &mut RecordedAnnularProgress::default())
        .unwrap_or_else(|failure| panic!("annular validation failed at {:?}", failure.checkpoint))
}
#[test]
fn actual_annular_validation_has_all_ordered_exact_children_without_publication() {
    let join = joined();
    let proof = complete(&join);
    assert_complete_bindings(&join, &proof);
    assert_eq!(
        proof.mode(),
        AnnularValidationMode::ExactTwoBoundaryPolynomialPrism
    );
    assert_eq!(proof.requested(), options());
    assert_domain_evidence(&join, &proof);
}
fn assert_complete_bindings(join: &AnnularFrozenDomain, proof: &AnnularValidationCertificate) {
    for stage in OrderedPrismStage::ALL {
        assert_eq!(proof.trace().status(stage), OrderedPrismStageStatus::Passed);
    }
    assert!(proof.binds(join) && proof.topology().binds(join) && proof.orientation().binds(join));
    assert!(
        proof.conformance().binds(join)
            && proof.intersection().binds(join)
            && proof.tolerance().binds(join)
    );
}
fn assert_domain_evidence(join: &AnnularFrozenDomain, proof: &AnnularValidationCertificate) {
    assert_eq!(proof.intersection().domains().len(), 4);
    assert_eq!(proof.intersection().incidences().len(), 10);
    assert_eq!(proof.tolerance().metadata().len(), 14);
    assert_eq!(proof.conformance().evidence().body.trims.len(), 12);
    assert_eq!(
        proof.historical().findings.len(),
        join.projection().coverage().findings.len()
    );
    assert!(!join.projection().is_valid_body());
    assert!(!proof.replay_bytes().is_empty());
}
#[test]
fn validation_callbacks_are_totally_ordered_across_all_stages() {
    let join = joined();
    let mut progress = RecordedAnnularProgress::default();
    let proof = validate_annular_prism(&join, options(), &mut progress)
        .ok()
        .unwrap();
    for (index, point) in progress.points.iter().enumerate() {
        assert_eq!(point.ordinal, index + 1);
    }
    let stages: Vec<_> = progress.points.iter().map(|p| p.stage.index()).collect();
    assert!(stages.windows(2).all(|pair| pair[0] <= pair[1]));
    assert_eq!(
        progress.points.last().unwrap().check,
        AnnularValidationCheck::Commit
    );
    assert_eq!(proof.trace().next(), None);
}
#[test]
fn invalid_version_and_refused_cap_keep_required_later_checks_blocked() {
    let join = joined();
    let mut requested = options();
    requested.version = 1;
    let failure = validate_annular_prism(&join, requested, &mut RecordedAnnularProgress::default())
        .err()
        .unwrap();
    assert_eq!(failure.class, OrderedPrismFailureClass::Incomplete);
    assert!(failure.partial.topology.is_none());
    assert_eq!(
        failure.trace.status(OrderedPrismStage::Geometry),
        OrderedPrismStageStatus::Blocked
    );
}
#[test]
fn refused_cap_retains_completed_geometry_and_intersection_only() {
    let join = joined();
    let mut requested = options();
    requested.stored_cap = StoredToleranceCap::refuse(0.).unwrap();
    let failure = validate_annular_prism(&join, requested, &mut RecordedAnnularProgress::default())
        .err()
        .unwrap();
    assert_eq!(failure.class, OrderedPrismFailureClass::Failed);
    assert!(failure.partial.intersection.is_some() && failure.partial.tolerance.is_none());
    assert_eq!(
        failure.trace.status(OrderedPrismStage::Tolerance),
        OrderedPrismStageStatus::Failed
    );
}
#[test]
fn ordered_annular_results_and_failures_are_owned_send_sync() {
    static_assertions::assert_impl_all!(AnnularValidationCertificate: Send, Sync);
    static_assertions::assert_impl_all!(AnnularValidationFailure: Send, Sync);
    static_assertions::assert_impl_all!(AnnularValidationOptions: Send, Sync);
    static_assertions::assert_impl_all!(AnnularValidationContext<'static>: Send);
}
