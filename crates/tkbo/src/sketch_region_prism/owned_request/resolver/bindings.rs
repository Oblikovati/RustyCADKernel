// SPDX-License-Identifier: GPL-2.0-only
//! Materialize existing private interfaces only after complete actual role correspondence.
use super::*;
use crate::sketch_region::OriginalMemberIdentity;
use crate::sketch_region_prism::frozen::{
    lineage::PrismCopyBindings, proof_join::AuthoredClosureContact, *,
};
pub(super) struct InvocationBindings {
    pub publication: PrismPublicationBindings,
    pub copied: PrismCopyBindings,
    pub historical: PrismHistoryBindings,
    pub contact: AuthoredClosureContact,
}
pub(super) fn assemble(
    prepared: &PreparedPolynomialPrism,
    plan: &SingleBoundaryNamePlan,
    roles: &roster::ResolvedStageRoles,
) -> ResolveCheck<InvocationBindings> {
    let (sources, copied) = copy_bindings(prepared, plan, roles)?;
    Ok(InvocationBindings {
        publication: publication_bindings(prepared, plan, roles, sources)?,
        copied,
        historical: materialize::intermediates(plan, roles)?,
        contact: contact_binding(plan, roles)?,
    })
}
fn publication_bindings(
    prepared: &PreparedPolynomialPrism,
    plan: &SingleBoundaryNamePlan,
    roles: &roster::ResolvedStageRoles,
    sources: Vec<PrismSourceBinding>,
) -> ResolveCheck<PrismPublicationBindings> {
    Ok(PrismPublicationBindings {
        operation: operation_name(plan)?,
        selected_region: materialize::authored(
            plan,
            SemanticNameStage::SweepBase,
            SemanticNameRole::BoundedRegion,
        )?,
        originals: original_bindings(prepared, plan, &roles.early.original)?,
        sources,
        directed: output::directed_outputs(prepared, plan, &roles.binding)?,
    })
}
fn original_bindings(
    prepared: &PreparedPolynomialPrism,
    plan: &SingleBoundaryNamePlan,
    rows: &[NamedSourceContext],
) -> ResolveCheck<Vec<PrismOriginalBinding>> {
    let identity =
        require_local(prepared.retained.source.as_ref(), "actual original source")?.identity;
    rows.iter()
        .map(|row| original_binding(plan, identity, row))
        .collect()
}
fn original_binding(
    plan: &SingleBoundaryNamePlan,
    identity: crate::sketch_region::RegionSourceId,
    row: &NamedSourceContext,
) -> ResolveCheck<PrismOriginalBinding> {
    let actual = &row.actual.occurrence;
    Ok(PrismOriginalBinding {
        actual: OriginalMemberIdentity {
            source: identity,
            topology_address: actual.topology_address,
            location: actual.location.clone(),
            orientation: actual.orientation,
        },
        member: materialize::authored(plan, SemanticNameStage::Original, row.member)?,
        directed_use: materialize::authored(plan, SemanticNameStage::Original, row.directed_use)?,
    })
}

fn contact_binding(
    plan: &SingleBoundaryNamePlan,
    roles: &roster::ResolvedStageRoles,
) -> ResolveCheck<AuthoredClosureContact> {
    Ok(AuthoredClosureContact {
        name: materialize::authored(
            plan,
            SemanticNameStage::Request,
            SemanticNameRole::ClosureContact,
        )?,
        actual: roles.binding.region.contact,
        original: roles.binding.region.original.clone(),
    })
}
fn copy_bindings(
    prepared: &PreparedPolynomialPrism,
    plan: &SingleBoundaryNamePlan,
    roles: &roster::ResolvedStageRoles,
) -> ResolveCheck<(Vec<PrismSourceBinding>, PrismCopyBindings)> {
    use PrismHistoryStage as H;
    use SemanticNameStage as S;
    let selected = materialize::historical(plan, H::Selected, S::Selected, &roles.selected)?;
    let sweep = materialize::historical(plan, H::SweepBase, S::SweepBase, &roles.sweep_base)?;
    Ok((
        sweep.members,
        PrismCopyBindings {
            selected_members: selected.members,
            selected_uses: selected_uses(prepared, plan, roles)?,
            copied_uses: copied_uses(prepared, plan, roles)?,
        },
    ))
}
fn selected_uses(
    prepared: &PreparedPolynomialPrism,
    plan: &SingleBoundaryNamePlan,
    roles: &roster::ResolvedStageRoles,
) -> ResolveCheck<Vec<PrismDirectedBinding>> {
    let candidate = resolve_certified_candidate(
        current::candidates(prepared)?,
        &roles.binding.region.source_candidate.face,
    )?;
    let live = current::live_contexts(prepared, &candidate.face, PrismHistoryStage::Selected)?;
    materialize::directed(plan, SemanticNameStage::Selected, &roles.selected, &live)
}
fn copied_uses(
    prepared: &PreparedPolynomialPrism,
    plan: &SingleBoundaryNamePlan,
    roles: &roster::ResolvedStageRoles,
) -> ResolveCheck<Vec<PrismDirectedBinding>> {
    let copy = require_local(
        current::receipt(prepared)?.copied.as_ref(),
        "actual selected copy",
    )?;
    let live = current::live_contexts(prepared, &copy.base, PrismHistoryStage::SweepBase)?;
    materialize::directed(plan, SemanticNameStage::SweepBase, &roles.sweep_base, &live)
}

fn operation_name(plan: &SingleBoundaryNamePlan) -> ResolveCheck<AuthoredPrismId> {
    materialize::authored(
        plan,
        SemanticNameStage::Request,
        SemanticNameRole::Operation,
    )
}
