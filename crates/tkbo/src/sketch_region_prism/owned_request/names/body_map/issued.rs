// SPDX-License-Identifier: GPL-2.0-only
//! Production issuance requires the sealed actual resolver/publication flow, never detached tokens plus bytes.
use super::{
    super::{body_keys::*, *},
    checks, expansion,
    types::*,
};
use crate::sketch_region_prism::frozen::{FrozenPrismKey, FrozenUseKey};
use crate::sketch_region_prism::owned_request::resolver::ResolvedPrismPublication;
/// Expand actual resolver-authored publication; `issue_resolved_prism_names(&invoked_result)`.
/// The sealed input has no constructor accepting a detached publication and caller dictionary.
pub(in crate::sketch_region_prism::owned_request) fn issue_resolved_prism_names(
    publication: &ResolvedPrismPublication,
) -> MapCheck<ResolvedPrismNames> {
    let projection = publication.result().join().projection();
    let members: Vec<_> = projection.nodes().iter().map(|row| row.key).collect();
    let uses: Vec<_> = projection
        .uses()
        .iter()
        .map(|row| row.key.clone())
        .collect();
    let plan = publication.plan();
    let domain = NameMapDomain {
        members: &members,
        uses: &uses,
        publication: publication.result().replay_bytes(),
    };
    checked_projection(plan, domain)
}
fn checked_projection(
    plan: &SingleBoundaryNamePlan,
    domain: NameMapDomain<'_>,
) -> MapCheck<ResolvedPrismNames> {
    let members = member_rows(plan, domain.members)?;
    let uses = use_rows(plan, domain.uses)?;
    checks::checked(plan.clone(), domain, members, uses)
}

fn member_rows(
    plan: &SingleBoundaryNamePlan,
    keys: &[FrozenPrismKey],
) -> MapCheck<Vec<SemanticMemberBinding>> {
    keys.iter()
        .map(|&frozen| {
            let semantic = expansion::member(plan, frozen)?;
            let output_slot = expansion::output(plan, frozen, semantic.source_member)?;
            Ok(SemanticMemberBinding {
                frozen,
                semantic,
                output_slot,
            })
        })
        .collect()
}
fn use_rows(
    plan: &SingleBoundaryNamePlan,
    keys: &[FrozenUseKey],
) -> MapCheck<Vec<SemanticUseBinding>> {
    keys.iter()
        .map(|frozen| {
            Ok(SemanticUseBinding {
                frozen: frozen.clone(),
                semantic: expansion::occurrence(plan, frozen)?,
            })
        })
        .collect()
}
