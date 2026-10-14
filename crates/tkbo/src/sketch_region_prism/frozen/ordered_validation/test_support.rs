// SPDX-License-Identifier: GPL-2.0-only
pub(super) use super::super::proof_join::operation_tests::JoinFixture;
use super::*;
pub(super) use crate::sketch_region_prism::exact_tolerance::StoredToleranceCap;
use rustycad_tkernel::standard::Failure;

#[derive(Default)]
pub(super) struct NamedValidationProgress {
    pub points: Vec<OrderedPrismCheckpoint>,
    pub stop: Option<usize>,
    pub fail: bool,
}
impl OrderedPrismValidationProgress for NamedValidationProgress {
    fn checkpoint(&mut self, point: OrderedPrismCheckpoint) -> Result<bool, Failure> {
        self.points.push(point);
        let stop = self.stop == Some(point.ordinal);
        if stop && self.fail {
            return Err(Failure::new(
                "owned callback failure at observed checkpoint",
            ));
        }
        Ok(stop)
    }
}
pub(super) fn options() -> OrderedPrismValidationOptions {
    let cap = StoredToleranceCap::refuse(1.).unwrap();
    OrderedPrismValidationOptions::v1(cap)
}
pub(super) fn complete(join: &JoinedPrismProof) -> Box<OrderedPrismValidationCertificate> {
    let outcome = validate_joined_prism(join, options(), &mut NamedValidationProgress::default());
    let OrderedPrismValidationOutcome::CompleteValidation(proof) = outcome else {
        panic!("expected complete ordered validation")
    };
    proof
}
pub(super) fn failure(
    outcome: OrderedPrismValidationOutcome,
) -> Box<OrderedPrismValidationFailure> {
    match outcome {
        OrderedPrismValidationOutcome::Failed(f)
        | OrderedPrismValidationOutcome::Incomplete(f)
        | OrderedPrismValidationOutcome::Cancelled(f) => f,
        OrderedPrismValidationOutcome::CompleteValidation(_) => {
            panic!("expected typed refusal without partial certificate")
        }
    }
}
