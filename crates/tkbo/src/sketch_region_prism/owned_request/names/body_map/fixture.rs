// SPDX-License-Identifier: GPL-2.0-only
//! Test-only explicit tables prove expansion/coverage, not upstream native correspondence.
use super::{
    super::{body_keys::*, *},
    checks,
    types::*,
};
use crate::sketch_region_prism::publication::PublishedNamedPrism;
/// Fixture-only checked rows; `fixture_bound_names(plan, result, members, uses)` assigns nothing.
/// Production issuance remains unavailable until the actual resolver proves semantic correspondence.
pub(crate) fn fixture_bound_names(
    plan: SingleBoundaryNamePlan,
    result: &PublishedNamedPrism,
    members: Vec<SemanticMemberBinding>,
    uses: Vec<SemanticUseBinding>,
) -> MapCheck<ResolvedPrismNames> {
    let projection = result.join().projection();
    let actual_members: Vec<_> = projection.nodes().iter().map(|row| row.key).collect();
    let actual_uses: Vec<_> = projection
        .uses()
        .iter()
        .map(|row| row.key.clone())
        .collect();
    let domain = NameMapDomain {
        members: &actual_members,
        uses: &actual_uses,
        publication: result.replay_bytes(),
    };
    checks::checked(plan, domain, members, uses)
}
/// Pure symbolic fixture only; `fixture_unbound_symbolic_names()` cannot bind an actual publication.
pub(crate) fn fixture_unbound_symbolic_names() -> ResolvedPrismNames {
    super::test_fixture::names()
}
