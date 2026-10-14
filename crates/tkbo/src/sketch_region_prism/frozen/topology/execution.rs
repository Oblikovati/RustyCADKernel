// SPDX-License-Identifier: GPL-2.0-only
use super::super::{AuthoredPrismId, FrozenPrismKey, FrozenPrismProjection};
use super::{
    admission, containment, incidence, links, model::*, pairing, traversal_binding, types::*, wires,
};

pub(super) struct TopologyWork<'a> {
    pub(super) current: TopologyCheckpoint,
    progress: &'a mut dyn TopologyProgress,
}
impl TopologyWork<'_> {
    pub(super) fn poll(
        &mut self,
        invariant: TopologyInvariant,
        completed: usize,
    ) -> TopologyResult<()> {
        self.current = TopologyCheckpoint {
            invariant,
            completed,
        };
        if self
            .progress
            .checkpoint(self.current)
            .map_err(callback_failure)?
        {
            return Err(CheckFailure::Cancelled);
        }
        Ok(())
    }
}
/// Certify closed combinatorial incidence, never embedded or outward solid validity.
/// Example: `certify_frozen_topology(&projection, &mut progress)`.
pub(crate) fn certify_frozen_topology(
    source: &FrozenPrismProjection,
    progress: &mut dyn TopologyProgress,
) -> FrozenTopologyOutcome {
    let mut work = TopologyWork {
        current: TopologyCheckpoint {
            invariant: TopologyInvariant::Admission,
            completed: 0,
        },
        progress,
    };
    let checked = certify(source, &mut work);
    finish(source.operation, work.current, checked)
}
fn certify(
    source: &FrozenPrismProjection,
    work: &mut TopologyWork<'_>,
) -> TopologyResult<FrozenTopologyCertificate> {
    admission::check(source, work)?;
    let mut index = admission::index(source, work)?;
    let root = containment::check(&index, work)?;
    incidence::check(source, &mut index, work)?;
    traversal_binding::check(source, &mut index, work)?;
    let corners = wires::check(&index, work)?;
    let adjacency = pairing::check(&index, work)?;
    links::check(&index, &corners, work)?;
    pairing::connected(&adjacency, work)?;
    pairing::euler(source.graph.counts, work)?;
    work.poll(TopologyInvariant::Commit, 0)?;
    Ok(completed(source, root))
}
fn completed(source: &FrozenPrismProjection, root: FrozenPrismKey) -> FrozenTopologyCertificate {
    FrozenTopologyCertificate {
        source: source.replay_bytes().into_boxed_slice(),
        operation: source.operation,
        root,
        counts: source.graph.counts,
    }
}

fn finish(
    operation: AuthoredPrismId,
    checkpoint: TopologyCheckpoint,
    checked: TopologyResult<FrozenTopologyCertificate>,
) -> FrozenTopologyOutcome {
    let failure = |witness| make_failure(operation, checkpoint, witness);
    match checked {
        Ok(report) => FrozenTopologyOutcome::Certified(Box::new(report)),
        Err(CheckFailure::Invalid(witness)) => FrozenTopologyOutcome::Failed(failure(witness)),
        Err(CheckFailure::Unsupported(witness)) => {
            FrozenTopologyOutcome::Incomplete(failure(witness))
        }
        Err(CheckFailure::Cancelled) => FrozenTopologyOutcome::Cancelled {
            operation,
            checkpoint,
        },
    }
}
fn callback_failure(cause: rustycad_tkernel::standard::Failure) -> CheckFailure {
    let actual = TopologyActual::Callback(Box::new(cause));
    CheckFailure::Unsupported(witness(None, actual, "successful progress callback"))
}

fn make_failure(
    operation: AuthoredPrismId,
    checkpoint: TopologyCheckpoint,
    witness: TopologyWitness,
) -> TopologyFailure {
    TopologyFailure {
        operation,
        checkpoint,
        witness,
    }
}
