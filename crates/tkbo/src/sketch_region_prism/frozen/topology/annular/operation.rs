// SPDX-License-Identifier: GPL-2.0-only
use super::*;

/// Certify the existing finite incidence with proved annular domains; `certify_annular_frozen_topology(joined,progress)`.
/// Geometry, orientation, tolerance and complete history remain separate ordered publication gates.
pub(in crate::sketch_region_prism) fn certify_annular_frozen_topology(
    joined: &AnnularFrozenDomain,
    progress: &mut dyn TopologyProgress,
) -> AnnularTopologyOutcome {
    let mut work = TopologyWork::new(progress);
    let checked = certify(joined, &mut work);
    finish(joined.projection().operation, work.current, checked)
}
fn certify(
    joined: &AnnularFrozenDomain,
    work: &mut TopologyWork<'_>,
) -> TopologyResult<AnnularTopologyCertificate> {
    let source = joined.projection();
    let root = check_incidence(source, &policy::BoundAnnularPolicy(joined), work)?;
    work.poll(TopologyInvariant::Commit, 0)?;
    Ok(AnnularTopologyCertificate {
        source: joined.replay_bytes().into(),
        operation: source.operation,
        root,
        counts: source.graph.counts,
    })
}
fn finish(
    operation: AuthoredPrismId,
    checkpoint: TopologyCheckpoint,
    checked: TopologyResult<AnnularTopologyCertificate>,
) -> AnnularTopologyOutcome {
    let failure = |witness| TopologyFailure {
        operation,
        checkpoint,
        witness,
    };
    match checked {
        Ok(certificate) => AnnularTopologyOutcome::Certified(Box::new(certificate)),
        Err(CheckFailure::Invalid(witness)) => AnnularTopologyOutcome::Failed(failure(witness)),
        Err(CheckFailure::Unsupported(witness)) => {
            AnnularTopologyOutcome::Incomplete(failure(witness))
        }
        Err(CheckFailure::Cancelled) => AnnularTopologyOutcome::Cancelled {
            operation,
            checkpoint,
        },
    }
}
