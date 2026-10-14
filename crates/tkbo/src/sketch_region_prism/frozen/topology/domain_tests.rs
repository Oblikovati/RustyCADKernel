// SPDX-License-Identifier: GPL-2.0-only
use super::super::*;
use super::{
    domain_fixture::*, domain_policy_fixture::*, execution::*, model::*,
    tests::NamedTopologyProgress, types::*,
};
use rustycad_tkg3d::top_abs::Orientation;

fn inspect(
    source: &FrozenPrismProjection,
    domains: &NamedAnnularAssumptions,
    progress: &mut NamedTopologyProgress,
) -> (TopologyResult<FrozenPrismKey>, TopologyCheckpoint) {
    let mut work = TopologyWork::new(progress);
    let result = check_incidence(source, domains, &mut work);
    (result, work.current)
}
fn accepted(source: &FrozenPrismProjection, domains: &NamedAnnularAssumptions) {
    let (result, point) = inspect(source, domains, &mut NamedTopologyProgress::default());
    match result {
        Ok(root) => assert_eq!(root.kind, 2),
        Err(CheckFailure::Invalid(w) | CheckFailure::Unsupported(w)) => panic!("{point:?}: {w:?}"),
        Err(CheckFailure::Cancelled) => panic!("unexpected fixture cancellation at {point:?}"),
    }
    assert_eq!(point.invariant, TopologyInvariant::FaceDomainEuler);
}
fn refused(
    source: &FrozenPrismProjection,
    domains: &NamedAnnularAssumptions,
    invariant: TopologyInvariant,
) {
    let (result, point) = inspect(source, domains, &mut NamedTopologyProgress::default());
    assert!(result.is_err());
    assert_eq!(point.invariant, invariant);
}
#[test]
fn annular_domain_fixture_passes_shared_incidence_with_weighted_euler_zero() {
    let source = annular_complex();
    let domains = NamedAnnularAssumptions::declared(&source, false);
    assert_eq!(source.graph.counts, [4, 6, 12, 6, 4, 1, 1]);
    assert_eq!(
        i64::from(source.graph.counts[0]) - i64::from(source.graph.counts[1])
            + i64::from(source.graph.counts[4]),
        2
    );
    accepted(&source, &domains);
    super::tests::rejected(&source, TopologyInvariant::ClosedContainment);
    assert!(!source.is_valid_body());
}
#[test]
fn explicit_radial_cuts_preserve_annular_surface_characteristic() {
    let source = cut_annular_complex();
    let domains = NamedAnnularAssumptions::declared(&source, true);
    assert_eq!(source.graph.counts, [4, 8, 16, 4, 4, 1, 1]);
    accepted(&source, &domains);
    super::tests::rejected(&source, TopologyInvariant::SphereEuler);
}
#[test]
fn missing_duplicate_foreign_and_disk_hole_assumptions_are_refused() {
    let source = annular_complex();
    let original = NamedAnnularAssumptions::declared(&source, false);
    let face = authored_face(&source, 0);
    for mode in 0..4 {
        let mut domains = original.clone();
        corrupt_domain(&mut domains, face, mode);
        refused(&source, &domains, TopologyInvariant::ClosedContainment);
    }
}
fn corrupt_domain(domains: &mut NamedAnnularAssumptions, face: &FrozenPrismUse, mode: u8) {
    if mode == 0 {
        domains.faces.remove(&face.key);
        return;
    }
    let mut outer = face.children[0].clone();
    if mode == 1 {
        outer.0[3].1 = AuthoredPrismId(u64::MAX);
    }
    let replacement = if mode == 3 {
        AssumedParameterDomain::Disk(outer)
    } else {
        AssumedParameterDomain::Annulus {
            outer,
            inner: face.children[0].clone(),
        }
    };
    domains.faces.insert(face.key.clone(), replacement);
}
#[test]
fn unused_extra_domain_cannot_be_accepted_by_equal_counts() {
    let source = annular_complex();
    let mut domains = NamedAnnularAssumptions::declared(&source, false);
    let face = authored_face(&source, 0);
    let mut foreign = face.key.clone();
    foreign.0[2].1 = AuthoredPrismId(u64::MAX);
    domains
        .faces
        .insert(foreign, domains.faces[&face.key].clone());
    refused(&source, &domains, TopologyInvariant::FaceDomainEuler);
}
#[test]
fn changed_source_bytes_cannot_rebind_assumptions_to_equal_topology() {
    let mut source = annular_complex();
    let domains = NamedAnnularAssumptions::declared(&source, false);
    source.nodes[0].raw_tolerance = Some(-0.0);
    refused(&source, &domains, TopologyInvariant::FaceDomainEuler);
    let (outcome, _) = inspect(&source, &domains, &mut NamedTopologyProgress::default());
    assert!(matches!(outcome, Err(CheckFailure::Unsupported(_))));
}
#[test]
fn annular_faces_still_require_opposite_shared_edge_uses() {
    let mut source = annular_complex();
    let domains = NamedAnnularAssumptions::declared(&source, false);
    let face = authored_face(&source, 0).key.clone();
    for row in &mut source.uses {
        if row.key.0.starts_with(&face.0) {
            row.orientation = super::combinatorial_tests::reverse(row.orientation);
        }
    }
    refused(&source, &domains, TopologyInvariant::OppositeEdgeUses);
}
#[test]
fn pinching_outer_and_inner_vertices_still_fails_the_shared_link_check() {
    let mut source = annular_complex();
    let domains = NamedAnnularAssumptions::declared(&source, false);
    pinch_vertices(&mut source);
    refused(&source, &domains, TopologyInvariant::CyclicVertexLinks);
}
fn pinch_vertices(source: &mut FrozenPrismProjection) {
    let old = fixture_vertex(source, 902);
    let new = fixture_vertex(source, 900);
    source.nodes.retain(|node| node.key != old);
    for row in &mut source.uses {
        replace_vertex(&mut row.key, old, new);
        for child in &mut row.children {
            replace_vertex(child, old, new);
        }
    }
    replace_endpoints(source, old, new);
    source.graph.counts[0] -= 1;
}
fn fixture_vertex(source: &FrozenPrismProjection, identity: u64) -> FrozenPrismKey {
    source
        .nodes
        .iter()
        .find(|n| n.key.kind == 7 && n.key.source.0 == identity)
        .expect("authored fixture vertex; expected declared identity")
        .key
}
fn replace_endpoints(source: &mut FrozenPrismProjection, old: FrozenPrismKey, new: FrozenPrismKey) {
    for (_, first, last) in &mut source.graph.endpoints {
        if *first == old {
            *first = new;
        }
        if *last == old {
            *last = new;
        }
    }
}
fn replace_vertex(path: &mut FrozenUseKey, old: FrozenPrismKey, new: FrozenPrismKey) {
    for (key, _) in &mut path.0 {
        if *key == old {
            *key = new;
        }
    }
}
#[test]
fn every_annular_incidence_checkpoint_can_cancel_or_fail_without_success() {
    let source = annular_complex();
    let original = source.replay_bytes();
    let domains = NamedAnnularAssumptions::declared(&source, false);
    let mut progress = NamedTopologyProgress::default();
    assert!(inspect(&source, &domains, &mut progress).0.is_ok());
    for point in progress.seen {
        refusal_at_checkpoint(&source, &domains, point);
        assert_eq!(source.replay_bytes(), original);
    }
}
fn refusal_at_checkpoint(
    source: &FrozenPrismProjection,
    domains: &NamedAnnularAssumptions,
    point: TopologyCheckpoint,
) {
    let mut cancel = NamedTopologyProgress {
        cancel: Some(point),
        ..Default::default()
    };
    let (cancelled, observed) = inspect(source, domains, &mut cancel);
    assert!(matches!(cancelled, Err(CheckFailure::Cancelled)));
    assert_eq!(observed, point);
    callback_failure_at_checkpoint(source, domains, point);
}
fn callback_failure_at_checkpoint(
    source: &FrozenPrismProjection,
    domains: &NamedAnnularAssumptions,
    point: TopologyCheckpoint,
) {
    let mut fail = NamedTopologyProgress {
        fail: Some(point),
        ..Default::default()
    };
    let (failed, observed) = inspect(source, domains, &mut fail);
    assert!(matches!(
        failed,
        Err(CheckFailure::Unsupported(TopologyWitness {
            actual: TopologyActual::Callback(_),
            ..
        }))
    ));
    assert_eq!(observed, point);
}
#[test]
fn annular_fixture_checks_are_thread_independent_and_immutable() {
    let source = annular_complex();
    let original = source.replay_bytes();
    let domains = NamedAnnularAssumptions::declared(&source, false);
    std::thread::scope(|scope| {
        let threads: Vec<_> = (0..4)
            .map(|_| scope.spawn(|| accepted(&source, &domains)))
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
    });
    assert_eq!(source.replay_bytes(), original);
    assert!(source.uses.iter().all(|row| matches!(
        row.orientation,
        Orientation::Forward | Orientation::Reversed
    )));
}
