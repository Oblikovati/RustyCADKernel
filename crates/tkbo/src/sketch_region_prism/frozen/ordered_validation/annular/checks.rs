// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_prism::region_certificate::two_loop::TwoLoopRefusalClass;
pub(super) fn topology(
    join: &AnnularFrozenDomain,
    context: &mut AnnularValidationContext<'_>,
) -> ValidationCheck<Box<AnnularTopologyCertificate>> {
    let result = match certify_annular_frozen_topology(join, context) {
        AnnularTopologyOutcome::Certified(proof) => Ok(proof),
        AnnularTopologyOutcome::Failed(cause) => Err(ValidationStop {
            class: OrderedPrismFailureClass::Failed,
            cause: AnnularValidationCause::Topology(Box::new(cause)),
        }),
        AnnularTopologyOutcome::Incomplete(cause) => Err(ValidationStop::incomplete(
            AnnularValidationCause::Topology(Box::new(cause)),
        )),
        AnnularTopologyOutcome::Cancelled { .. } => Err(ValidationStop::cancelled()),
    };
    let proof = context.accept(result)?;
    binding(proof.binds(join), "whole annular topology binding")?;
    context.passed(OrderedPrismStage::Topology);
    Ok(proof)
}
pub(super) fn orientation(
    join: &AnnularFrozenDomain,
    context: &mut AnnularValidationContext<'_>,
) -> ValidationCheck<AnnularOrientationCertificate> {
    let mut commit = false;
    let result =
        inspect_annular_orientation(join, AnnularOrientationOptions { version: 2 }, || {
            let stopped = context.poll(AnnularValidationCheck::Orientation { commit });
            commit = true;
            stopped
        })
        .map_err(direction_stop);
    let proof = context.accept(result)?;
    binding(
        proof.binds(join),
        "whole annular face/trim direction binding",
    )?;
    Ok(proof)
}
fn direction_stop(cause: Box<AnnularDirectionFailure>) -> ValidationStop {
    ValidationStop {
        class: direction_class(cause.classification()),
        cause: AnnularValidationCause::Orientation(cause),
    }
}
fn direction_class(class: TwoLoopRefusalClass) -> OrderedPrismFailureClass {
    match class {
        TwoLoopRefusalClass::Failed => OrderedPrismFailureClass::Failed,
        TwoLoopRefusalClass::Cancelled => OrderedPrismFailureClass::Cancelled,
        TwoLoopRefusalClass::Incomplete => OrderedPrismFailureClass::Incomplete,
    }
}
pub(super) fn conformance(
    join: &AnnularFrozenDomain,
    context: &mut AnnularValidationContext<'_>,
) -> ValidationCheck<AnnularPrismConformance> {
    let result = certify_annular_prism_conformance(join, context).map_err(|cause| ValidationStop {
        class: cause.classification().into(),
        cause: AnnularValidationCause::Conformance(cause),
    });
    let proof = context.accept(result)?;
    binding(proof.binds(join), "whole annular geometry binding")?;
    context.passed(OrderedPrismStage::Geometry);
    Ok(proof)
}
pub(super) fn intersection(
    join: &AnnularFrozenDomain,
    context: &mut AnnularValidationContext<'_>,
) -> ValidationCheck<AnnularIntersectionCertificate> {
    let result = check_bound_annular_table(join.embedding(), |completed| {
        context.poll(AnnularValidationCheck::Incidence { completed })
    })
    .map_err(|cause| ValidationStop {
        class: if matches!(cause, AnnularTableFailure::Cancelled { .. }) {
            OrderedPrismFailureClass::Cancelled
        } else {
            OrderedPrismFailureClass::Incomplete
        },
        cause: AnnularValidationCause::SelfIntersection(Box::new(cause)),
    });
    context.accept(result)?;
    context.passed(OrderedPrismStage::SelfIntersection);
    Ok(intersection_certificate(join))
}
fn intersection_certificate(join: &AnnularFrozenDomain) -> AnnularIntersectionCertificate {
    let domains = join.embedding().domains();
    AnnularIntersectionCertificate {
        source: join.replay_bytes().into(),
        domains: [
            &domains.first,
            &domains.last,
            &domains.outer,
            &domains.inner,
        ]
        .map(Clone::clone),
        incidences: join.embedding().incidences().clone(),
        embedding: join.embedding().replay_bytes().into(),
    }
}
pub(super) fn tolerance(
    join: &AnnularFrozenDomain,
    geometry: &AnnularPrismConformance,
    cap: StoredToleranceCap,
    context: &mut AnnularValidationContext<'_>,
) -> ValidationCheck<AnnularPrismTolerance> {
    let result = certify_annular_prism_tolerances(join, geometry, cap, context).map_err(|cause| {
        ValidationStop {
            class: cause.classification().into(),
            cause: AnnularValidationCause::Tolerance(cause),
        }
    });
    let proof = context.accept(result)?;
    binding(proof.binds(join), "whole annular tolerance binding")?;
    Ok(proof)
}
