// SPDX-License-Identifier: GPL-2.0-only
use super::{checks, errors::*, work::OrderedWork, *};

/// Validate one already joined body in the required order, without publishing an operation.
/// Example: `validate_joined_prism(&join, OrderedPrismValidationOptions::v1(cap), &mut progress)`.
pub(crate) fn validate_joined_prism(
    join: &JoinedPrismProof,
    requested: OrderedPrismValidationOptions,
    progress: &mut dyn OrderedPrismValidationProgress,
) -> OrderedPrismValidationOutcome {
    let mut work = OrderedWork::new(progress);
    match validate(join, requested, &mut work) {
        Ok(proof) => OrderedPrismValidationOutcome::CompleteValidation(Box::new(proof)),
        Err(stop) => refused(join, requested, work, stop),
    }
}
fn validate(
    join: &JoinedPrismProof,
    requested: OrderedPrismValidationOptions,
    work: &mut OrderedWork<'_>,
) -> OrderedCheck<OrderedPrismValidationCertificate> {
    admit(requested, work)?;
    let topology = checks::topology(join, work)?;
    let orientation = checks::orientation(join, work)?;
    let conformance = checks::conformance(join, work)?;
    let intersection = checks::self_intersection(join, work)?;
    let tolerance = checks::tolerance(join, &conformance, requested.stored_cap, work)?;
    Ok(assemble(
        join,
        requested,
        work.trace.clone(),
        (topology, orientation, conformance, intersection, tolerance),
    ))
}
type CompletedProofs = (
    Box<topology::FrozenTopologyCertificate>,
    joined_orientation::JoinedPrismOrientationCertificate,
    crate::sketch_region_prism::exact_tolerance::JoinedPrismConformance,
    JoinedPrismIntersectionCertificate,
    crate::sketch_region_prism::exact_tolerance::JoinedPrismTolerance,
);
fn assemble(
    join: &JoinedPrismProof,
    requested: OrderedPrismValidationOptions,
    trace: OrderedPrismStageTrace,
    proofs: CompletedProofs,
) -> OrderedPrismValidationCertificate {
    let (topology, orientation, conformance, intersection, tolerance) = proofs;
    OrderedPrismValidationCertificate {
        source: join.replay_bytes().into(),
        requested,
        trace,
        topology,
        orientation,
        conformance,
        intersection,
        tolerance,
        historical: join.projection().coverage().clone(),
    }
}

fn admit(requested: OrderedPrismValidationOptions, work: &mut OrderedWork<'_>) -> OrderedCheck<()> {
    if requested.version != 1 {
        return Err(OrderedStop::incomplete(OrderedPrismCause::Version {
            actual: requested.version,
            expected: 1,
        }));
    }
    work.checkpoint(OrderedPrismCheck::Admission)
}
fn refused(
    join: &JoinedPrismProof,
    requested: OrderedPrismValidationOptions,
    mut work: OrderedWork<'_>,
    stop: OrderedStop,
) -> OrderedPrismValidationOutcome {
    work.trace
        .record_prism_stage(work.current.stage, stop.class.into())
        .expect("internal ordered validator must refuse only its next stage");
    let failure = Box::new(OrderedPrismValidationFailure {
        operation: join.projection().operation,
        requested,
        checkpoint: work.current,
        class: stop.class,
        trace: work.trace,
        cause: stop.cause,
        source: join.replay_bytes().into(),
    });
    failure_outcome(failure)
}
fn failure_outcome(failure: Box<OrderedPrismValidationFailure>) -> OrderedPrismValidationOutcome {
    match failure.class {
        OrderedPrismFailureClass::Failed => OrderedPrismValidationOutcome::Failed(failure),
        OrderedPrismFailureClass::Incomplete => OrderedPrismValidationOutcome::Incomplete(failure),
        OrderedPrismFailureClass::Cancelled => OrderedPrismValidationOutcome::Cancelled(failure),
    }
}
