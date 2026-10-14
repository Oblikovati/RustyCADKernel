// SPDX-License-Identifier: GPL-2.0-only
//! Actual annular preparation refusals: admission is owned losslessly, every callback cutoff keeps
//! its exact checkpoint and original class, and retained partial transactions stay explicit gaps.
use super::*;
use crate::sketch_region_prism::owned_request::annular::prepare_owned_annular_prism;
use crate::sketch_region_prism::owned_request::annular::tests::owned_request;
use crate::sketch_region_prism::owned_request::request_replay::encode_owned_annular_request;
use rustycad_tkernel::standard::Failure;

const _: () = {
    const fn owned<T: Send + Sync>() {}
    owned::<OwnedAnnularPreparationDiagnostic>();
};
struct CancelAt(usize, usize);
impl OwnedPrismProgress for CancelAt {
    fn checkpoint(&mut self, _: OwnedPrismCheckpoint) -> Result<bool, Failure> {
        self.1 += 1;
        Ok(self.1 == self.0)
    }
}
fn refused(request: OwnedAnnularPrismRequest, cutoff: usize) -> Box<AnnularPreparationFailure> {
    let outcome = prepare_owned_annular_prism(request, &mut CancelAt(cutoff, 0));
    outcome
        .err()
        .expect("expected an annular preparation refusal")
}
fn projected(failure: Box<AnnularPreparationFailure>) -> Box<OwnedAnnularPreparationDiagnostic> {
    match project_annular_preparation_refusal(failure) {
        AnnularPreparationRefusalProjection::Projected(diagnostic) => diagnostic,
        AnnularPreparationRefusalProjection::Incomplete { diagnostic, .. } => {
            panic!("unexpected gaps {:?}", diagnostic.gaps())
        }
    }
}
fn duplicate_names() -> OwnedAnnularPrismRequest {
    let mut request = owned_request(false);
    request.regions.inner_domain = request.regions.annulus;
    request
}
#[test]
fn name_refusal_is_owned_losslessly_with_independent_tail_bytes() {
    let request = duplicate_names();
    let diagnostic = projected(refused(request.clone(), 0));
    let quantity = "nonzero distinct region and wire names";
    let names = AnnularAdmissionCause::Names { quantity };
    assert_eq!(
        diagnostic.cause(),
        &OwnedAnnularPreparationCause::Admission(names)
    );
    assert_eq!(diagnostic.class(), PrismDiagnosticClass::Failed);
    let mut expected = b"Martensite.OwnedAnnularPreparationDiagnostic/v1\0".to_vec();
    let encoded = encode_owned_annular_request(&request);
    expected.extend((encoded.len() as u64).to_le_bytes());
    expected.extend(encoded);
    expected.extend([0, 2, 0, 1]);
    expected.extend((quantity.len() as u64).to_le_bytes());
    expected.extend(quantity.as_bytes());
    expected.push(0);
    expected.extend(0u64.to_le_bytes());
    assert_eq!(diagnostic.replay_bytes(), expected);
}
#[test]
fn pole_and_version_refusals_keep_typed_operands_and_original_class() {
    let mut request = owned_request(false);
    request.inner.world_poles.pop();
    let diagnostic = projected(refused(request, 0));
    // Removing the closing pole leaves the inner loop open at its exact authored endpoints.
    let open = OwnedPrismAdmissionCause::OpenBoundary {
        source: CallerBoundaryId(202),
        first: [0.5, 0.5, 0.],
        last: [0.5, 2.5, 0.],
    };
    let expected = OwnedAnnularPreparationCause::Admission(AnnularAdmissionCause::Owned(open));
    assert_eq!(diagnostic.cause(), &expected);
    let mut request = owned_request(false);
    request.version = 2;
    let diagnostic = projected(refused(request, 0));
    assert_eq!(diagnostic.class(), PrismDiagnosticClass::Incomplete);
    let version = OwnedPrismAdmissionCause::UnsupportedVersion {
        actual: 2,
        expected: 1,
    };
    let expected = OwnedAnnularPreparationCause::Admission(AnnularAdmissionCause::Owned(version));
    assert_eq!(diagnostic.cause(), &expected);
}
fn checkpoints() -> usize {
    let mut counter = CancelAt(0, 0);
    assert!(prepare_owned_annular_prism(owned_request(false), &mut counter).is_ok());
    counter.1
}
/// The original Box is retained whenever a partial transaction owns native shapes.
fn assert_cutoff(cutoff: usize) {
    let failure = refused(owned_request(false), cutoff);
    let (phase, class) = (failure.phase, failure.class);
    let owners = failure.retained_stages();
    let (diagnostic, original) = match project_annular_preparation_refusal(failure) {
        AnnularPreparationRefusalProjection::Projected(d) => (d, None),
        AnnularPreparationRefusalProjection::Incomplete {
            diagnostic,
            original,
        } => (diagnostic, Some(original)),
    };
    assert_eq!(diagnostic.class(), PrismDiagnosticClass::Cancelled);
    assert_eq!(diagnostic.phase(), phase);
    let callback = diagnostic.callback().expect("exact callback evidence");
    assert_eq!(callback.checkpoint.ordinal + 1, cutoff, "cutoff {cutoff}");
    assert_eq!(callback.cause, OwnedDiagnosticCallbackCause::Cancelled);
    assert_eq!(
        original.as_ref().map(|o| o.class),
        (!owners.is_empty()).then_some(class)
    );
    if !owners.is_empty() {
        assert!(
            diagnostic
                .gaps()
                .contains(&PrismDiagnosticGap::OuterOwners(owners))
        );
    }
}
#[test]
fn sampled_callback_cutoffs_keep_checkpoint_class_and_partial_transaction_gaps() {
    let total = checkpoints();
    for cutoff in [1, 2, 3, total / 2, total - 1, total] {
        assert_cutoff(cutoff);
    }
}
#[test]
fn thread_replay_matches_and_source_request_is_isolated() {
    let bytes = || projected(refused(duplicate_names(), 0)).replay_bytes();
    let first = bytes();
    let second = std::thread::spawn(bytes).join().expect("replay thread");
    assert_eq!(first, second);
    let diagnostic = projected(refused(duplicate_names(), 0));
    let mut request = diagnostic.request().clone();
    request.operation.0 += 1;
    assert_eq!(diagnostic.request().operation.0, 77);
}
