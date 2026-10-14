// SPDX-License-Identifier: GPL-2.0-only
use super::{fixture::*, *};
use crate::sketch_region_prism::frozen::FrozenSweepRole;

#[test]
fn complete_actual_publication_preserves_native_semantics_for_all_eight_reversals() {
    for variant in 0..8 {
        let actual = resolved_prism_fixture(reversal_request(variant));
        assert_eq!(actual.publication.join().projection().nodes().len(), 13);
        assert_eq!(actual.names.use_bindings().len(), 26);
        assert_seam_anchors(&actual.names);
    }
}
fn reversal_request(variant: u8) -> super::super::OwnedPolynomialPrismRequest {
    let mut request = super::super::test_fixture::request();
    request.sources[0].reversed_traversal = variant & 1 != 0;
    if variant & 2 != 0 {
        request.sources[0].world_poles.reverse();
    }
    if variant & 4 != 0 {
        request.extrusion[2] = -2.;
    }
    request
}
fn assert_seam_anchors(names: &body_map::ResolvedPrismNames) {
    use SemanticNameRole::*;
    for (seam, anchor) in [(SeamStartUse, StartUse), (SeamEndUse, EndUse)] {
        let endpoints: Vec<_> = names
            .use_bindings()
            .iter()
            .filter(|row| {
                row.semantic.0.last().is_some_and(|s| s.member.kind == 7)
                    && row.semantic.0.iter().any(|s| s.directed_use.role == seam)
            })
            .collect();
        assert_endpoint_anchors(&endpoints, anchor);
    }
}

fn assert_endpoint_anchors(endpoints: &[&SemanticUseBinding], anchor: SemanticNameRole) {
    assert_eq!(endpoints.len(), 2);
    let mut roles = Vec::new();
    for endpoint in endpoints {
        let last = endpoint.semantic.0.last().unwrap();
        assert_eq!(last.directed_use.role, anchor);
        roles.push(last.member.sweep_role);
    }
    assert!(roles.contains(&FrozenSweepRole::First));
    assert!(roles.contains(&FrozenSweepRole::Last));
}
