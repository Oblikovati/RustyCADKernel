// SPDX-License-Identifier: GPL-2.0-only
use super::{
    classification::orientation_stop, errors::*, joined_orientation::*, topology::*,
    work::OrderedWork, *,
};
use crate::sketch_region_prism::{exact_tolerance::*, region_certificate::embedding::*};

pub(super) fn topology(
    join: &JoinedPrismProof,
    work: &mut OrderedWork<'_>,
) -> OrderedCheck<Box<FrozenTopologyCertificate>> {
    let checked = match certify_frozen_topology(join.projection(), work) {
        FrozenTopologyOutcome::Certified(proof) => Ok(proof),
        FrozenTopologyOutcome::Failed(cause) => Err(OrderedStop {
            class: OrderedPrismFailureClass::Failed,
            cause: OrderedPrismCause::Topology(Box::new(cause)),
        }),
        FrozenTopologyOutcome::Incomplete(cause) => Err(OrderedStop::incomplete(
            OrderedPrismCause::Topology(Box::new(cause)),
        )),
        FrozenTopologyOutcome::Cancelled { .. } => Err(OrderedStop::cancelled()),
    };
    let proof = work.accept(checked)?;
    binding(proof.binds(join.projection()), "exact topology")?;
    work.passed(OrderedPrismStage::Topology);
    Ok(proof)
}
pub(super) fn orientation(
    join: &JoinedPrismProof,
    work: &mut OrderedWork<'_>,
) -> OrderedCheck<JoinedPrismOrientationCertificate> {
    let mut commit = false;
    let checked =
        inspect_joined_prism_orientation(join, JoinedPrismOrientationOptions::v1(), || {
            let cancelled = work.poll(OrderedPrismCheck::Orientation { commit });
            commit = true;
            cancelled
        })
        .map_err(orientation_stop);
    let proof = work.accept(checked)?;
    binding(proof.matches_join(join), "named face and trim orientation")?;
    Ok(proof)
}
pub(super) fn conformance(
    join: &JoinedPrismProof,
    work: &mut OrderedWork<'_>,
) -> OrderedCheck<JoinedPrismConformance> {
    let checked = certify_joined_prism_conformance(join, work).map_err(|cause| OrderedStop {
        class: cause.classification().into(),
        cause: OrderedPrismCause::Conformance(cause),
    });
    let proof = work.accept(checked)?;
    binding(
        proof.binds(join),
        "raw endpoint and original closure conformance",
    )?;
    work.passed(OrderedPrismStage::Geometry);
    Ok(proof)
}
pub(super) fn self_intersection(
    join: &JoinedPrismProof,
    work: &mut OrderedWork<'_>,
) -> OrderedCheck<JoinedPrismIntersectionCertificate> {
    let checked = check_prism_embedding_table(join.embedding(), |completed| {
        work.poll(OrderedPrismCheck::Incidence { completed })
    })
    .map_err(|cause| OrderedStop {
        class: match cause {
            PrismEmbeddingTableFailure::Cancelled { .. } => OrderedPrismFailureClass::Cancelled,
            _ => OrderedPrismFailureClass::Incomplete,
        },
        cause: OrderedPrismCause::SelfIntersection(Box::new(cause)),
    });
    work.accept(checked)?;
    work.passed(OrderedPrismStage::SelfIntersection);
    Ok(JoinedPrismIntersectionCertificate::checked(join))
}
pub(super) fn tolerance(
    join: &JoinedPrismProof,
    geometry: &JoinedPrismConformance,
    cap: StoredToleranceCap,
    work: &mut OrderedWork<'_>,
) -> OrderedCheck<JoinedPrismTolerance> {
    let checked =
        certify_joined_prism_tolerances(join, geometry, cap, work).map_err(|cause| OrderedStop {
            class: cause.classification().into(),
            cause: OrderedPrismCause::Tolerance(cause),
        });
    let proof = work.accept(checked)?;
    binding(proof.binds(join), "stored tolerance metadata")?;
    work.checkpoint(OrderedPrismCheck::Commit)?;
    work.passed(OrderedPrismStage::Tolerance);
    Ok(proof)
}
pub(super) fn binding(matches: bool, proof: &'static str) -> OrderedCheck<()> {
    if !matches {
        return Err(OrderedStop::incomplete(OrderedPrismCause::Binding {
            proof,
        }));
    }
    Ok(())
}
