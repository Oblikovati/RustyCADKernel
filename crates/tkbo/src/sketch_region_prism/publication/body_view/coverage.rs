// SPDX-License-Identifier: GPL-2.0-only
//! Exact domains, not matching totals: every definition and complete directed path occurs once.
use super::error::*;
use crate::sketch_region_prism::{frozen::*, owned_request::names::body_map::ResolvedPrismNames};
use std::collections::BTreeSet;

pub(super) fn domains(
    names: &ResolvedPrismNames,
    members: &[FrozenPrismNode],
    uses: &[FrozenPrismUse],
) -> BodyCheck<()> {
    let actual_members: Vec<_> = members.iter().map(|row| row.key).collect();
    let expected_members: Vec<_> = names
        .member_bindings()
        .iter()
        .map(|row| row.frozen)
        .collect();
    exact_keys(
        names,
        "complete unique named member domain",
        &actual_members,
        &expected_members,
    )?;
    uses_domain(names, uses)
}
fn uses_domain(names: &ResolvedPrismNames, uses: &[FrozenPrismUse]) -> BodyCheck<()> {
    let actual_uses: Vec<_> = uses.iter().map(|row| &row.key).collect();
    let expected_uses: Vec<_> = names.use_bindings().iter().map(|row| &row.frozen).collect();
    exact_keys(
        names,
        "complete unique named occurrence domain",
        &actual_uses,
        &expected_uses,
    )
}
pub(super) fn exact_keys<T: Ord>(
    names: &ResolvedPrismNames,
    required: &'static str,
    actual: &[T],
    expected: &[T],
) -> BodyCheck<()> {
    let left: BTreeSet<_> = actual.iter().collect();
    let right: BTreeSet<_> = expected.iter().collect();
    count(names, required, left.len(), actual.len())?;
    count(names, required, right.len(), expected.len())?;
    count(names, required, actual.len(), expected.len())?;
    count(
        names,
        required,
        left.intersection(&right).count(),
        expected.len(),
    )
}
