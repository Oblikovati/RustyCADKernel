// SPDX-License-Identifier: GPL-2.0-only
//! Complete exact table coverage and ancestry; no constructor is exported to production yet.
use super::{
    super::{body_keys::*, *},
    expansion,
    types::*,
};
use crate::sketch_region_prism::frozen::{FrozenPrismKey, FrozenUseKey};
pub(super) fn checked(
    plan: SingleBoundaryNamePlan,
    domain: NameMapDomain<'_>,
    members: Vec<SemanticMemberBinding>,
    uses: Vec<SemanticUseBinding>,
) -> MapCheck<ResolvedPrismNames> {
    require_nonempty(&plan, &domain)?;
    member_coverage(&plan, domain.members, &members)?;
    use_coverage(&plan, domain.uses, &uses)?;
    check_members(&plan, &members)?;
    check_uses(&plan, &members, &uses)?;
    Ok(assemble(plan, domain.publication, members, uses))
}
fn assemble(
    plan: SingleBoundaryNamePlan,
    publication: &[u8],
    mut members: Vec<SemanticMemberBinding>,
    mut uses: Vec<SemanticUseBinding>,
) -> ResolvedPrismNames {
    members.sort_by_key(|row| row.frozen);
    uses.sort_by(|a, b| a.frozen.cmp(&b.frozen));
    let envelope = SemanticReplayEnvelope::new(plan, publication);
    let replay = super::replay::encode(&envelope, &members, &uses);
    ResolvedPrismNames {
        envelope,
        members,
        uses,
        replay,
    }
}
fn multiplicity<T: Clone + Eq>(domain: &[T], rows: &[T]) -> Option<(T, usize, usize)> {
    for key in domain.iter().chain(rows) {
        let expected = domain.iter().filter(|value| *value == key).count();
        let actual = rows.iter().filter(|value| *value == key).count();
        if expected != 1 || actual != 1 {
            return Some((key.clone(), actual, expected));
        }
    }
    None
}
fn member_coverage(
    plan: &SingleBoundaryNamePlan,
    domain: &[FrozenPrismKey],
    rows: &[SemanticMemberBinding],
) -> MapCheck<()> {
    let keys: Vec<_> = rows.iter().map(|row| row.frozen).collect();
    if let Some((key, actual, domain)) = multiplicity(domain, &keys) {
        return Err(failure(
            plan,
            PrismSemanticMapCause::MemberMultiplicity {
                key,
                actual,
                domain,
                expected: 1,
            },
        ));
    }
    Ok(())
}
fn use_coverage(
    plan: &SingleBoundaryNamePlan,
    domain: &[FrozenUseKey],
    rows: &[SemanticUseBinding],
) -> MapCheck<()> {
    let keys: Vec<_> = rows.iter().map(|row| row.frozen.clone()).collect();
    if let Some((key, actual, domain)) = multiplicity(domain, &keys) {
        return Err(failure(
            plan,
            PrismSemanticMapCause::UseMultiplicity {
                key,
                actual,
                domain,
                expected: 1,
            },
        ));
    }
    Ok(())
}
fn check_members(plan: &SingleBoundaryNamePlan, rows: &[SemanticMemberBinding]) -> MapCheck<()> {
    for (index, row) in rows.iter().enumerate() {
        check_member(plan, row)?;
        if let Some(other) = rows[..index]
            .iter()
            .find(|other| other.semantic == row.semantic)
        {
            return Err(failure(
                plan,
                PrismSemanticMapCause::MemberCollision {
                    first: other.frozen,
                    second: row.frozen,
                },
            ));
        }
    }
    Ok(())
}
fn check_member(plan: &SingleBoundaryNamePlan, row: &SemanticMemberBinding) -> MapCheck<()> {
    if row.semantic != expansion::member(plan, row.frozen)? {
        return Err(failure(
            plan,
            PrismSemanticMapCause::MemberExpansion { key: row.frozen },
        ));
    }
    check_output(plan, row)
}
fn check_output(plan: &SingleBoundaryNamePlan, row: &SemanticMemberBinding) -> MapCheck<()> {
    let expected = expansion::output(plan, row.frozen, row.semantic.source_member)?;
    if row.output_slot != expected {
        return Err(failure(
            plan,
            PrismSemanticMapCause::OutputSlot {
                key: row.frozen,
                actual: row.output_slot,
                expected,
            },
        ));
    }
    Ok(())
}
fn check_uses(
    plan: &SingleBoundaryNamePlan,
    members: &[SemanticMemberBinding],
    rows: &[SemanticUseBinding],
) -> MapCheck<()> {
    for (index, row) in rows.iter().enumerate() {
        check_use(plan, row, members, rows)?;
        if let Some(other) = rows[..index]
            .iter()
            .find(|other| other.semantic == row.semantic)
        {
            return Err(use_collision(plan, &other.frozen, &row.frozen));
        }
    }
    Ok(())
}
fn use_collision(
    plan: &SingleBoundaryNamePlan,
    first: &FrozenUseKey,
    second: &FrozenUseKey,
) -> Box<PrismSemanticMapFailure> {
    failure(
        plan,
        PrismSemanticMapCause::UseCollision {
            first: first.clone(),
            second: second.clone(),
        },
    )
}
fn check_use(
    plan: &SingleBoundaryNamePlan,
    row: &SemanticUseBinding,
    members: &[SemanticMemberBinding],
    uses: &[SemanticUseBinding],
) -> MapCheck<()> {
    if row.semantic != expansion::occurrence(plan, &row.frozen)? {
        return Err(failure(
            plan,
            PrismSemanticMapCause::UseExpansion {
                key: row.frozen.clone(),
            },
        ));
    }
    for (depth, (key, _)) in row.frozen.0.iter().enumerate() {
        require_member(plan, *key, members)?;
        require_ancestor(plan, &row.frozen, depth, uses)?;
    }
    Ok(())
}
fn require_member(
    plan: &SingleBoundaryNamePlan,
    key: FrozenPrismKey,
    members: &[SemanticMemberBinding],
) -> MapCheck<()> {
    if !members.iter().any(|member| member.frozen == key) {
        return Err(failure(
            plan,
            PrismSemanticMapCause::MemberMultiplicity {
                key,
                actual: 0,
                domain: 1,
                expected: 1,
            },
        ));
    }
    Ok(())
}
fn require_ancestor(
    plan: &SingleBoundaryNamePlan,
    key: &FrozenUseKey,
    depth: usize,
    uses: &[SemanticUseBinding],
) -> MapCheck<()> {
    let parent = FrozenUseKey(key.0[..depth + 1].to_vec());
    if !uses.iter().any(|candidate| candidate.frozen == parent) {
        return Err(failure(
            plan,
            PrismSemanticMapCause::MissingAncestor {
                key: key.clone(),
                parent,
            },
        ));
    }
    Ok(())
}
fn require_nonempty(plan: &SingleBoundaryNamePlan, domain: &NameMapDomain<'_>) -> MapCheck<()> {
    if domain.members.is_empty() || domain.uses.is_empty() {
        return Err(failure(
            plan,
            PrismSemanticMapCause::EmptyDomain {
                members: domain.members.len(),
                uses: domain.uses.len(),
            },
        ));
    }
    Ok(())
}
