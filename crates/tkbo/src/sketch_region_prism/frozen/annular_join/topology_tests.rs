// SPDX-License-Identifier: GPL-2.0-only
//! Privileged adversarial fixture mutations cannot create production domain authority.
use super::super::topology::*;
use super::operation_tests::joined;
use super::*;
use rustycad_tkernel::standard::Failure;

struct TopologyCallback<F>(F);
impl<F: FnMut(TopologyCheckpoint) -> Result<bool, Failure> + Send> TopologyProgress
    for TopologyCallback<F>
{
    fn checkpoint(&mut self, checkpoint: TopologyCheckpoint) -> Result<bool, Failure> {
        (self.0)(checkpoint)
    }
}
#[test]
fn same_count_duplicate_cap_wire_role_cannot_replace_exact_domain_bijection() {
    let mut joined = joined();
    let domain = joined
        .faces
        .iter_mut()
        .find_map(|face| match &mut face.domain {
            NamedAnnularDomain::Cap { outer, inner } => Some((outer, inner)),
            _ => None,
        })
        .unwrap();
    domain.1.occurrence = domain.0.occurrence.clone();
    let outcome = certify_annular_frozen_topology(&joined, &mut TopologyCallback(|_| Ok(false)));
    let AnnularTopologyOutcome::Failed(failure) = outcome else {
        panic!("typed wire-domain mismatch")
    };
    assert_eq!(
        failure.checkpoint.invariant,
        TopologyInvariant::ClosedContainment
    );
    assert!(failure.witness.occurrence.is_some());
}
#[test]
fn exact_annular_topology_can_cancel_at_domain_euler_and_final_commit() {
    let joined = joined();
    for invariant in [
        TopologyInvariant::FaceDomainEuler,
        TopologyInvariant::Commit,
    ] {
        let outcome = certify_annular_frozen_topology(
            &joined,
            &mut TopologyCallback(|point: TopologyCheckpoint| Ok(point.invariant == invariant)),
        );
        let AnnularTopologyOutcome::Cancelled {
            operation,
            checkpoint,
        } = outcome
        else {
            panic!("explicit cancellation")
        };
        assert_eq!(operation, AuthoredPrismId(77));
        assert_eq!(checkpoint.invariant, invariant);
    }
}
#[test]
fn complete_join_read_views_keep_domain_proof_raw_vertices_and_parent_multiplicity() {
    let joined = joined();
    assert_eq!(joined.embedding().incidences().len(), 10);
    assert_eq!(joined.parents().len(), 48);
    assert_eq!(joined.raw_vertices().len(), 24);
    assert_eq!(joined.name_census().requested, tests::options());
    assert_eq!(
        joined.name_census().requested,
        joined.name_census().effective
    );
    assert_eq!(
        joined
            .faces()
            .iter()
            .map(|f| f.domain.boundary_uses().len())
            .sum::<usize>(),
        6
    );
    assert!(joined.parents().iter().all(|row| !row.preserved.is_empty()
        || !row.modified.is_empty()
        || !row.generated.is_empty()));
}
