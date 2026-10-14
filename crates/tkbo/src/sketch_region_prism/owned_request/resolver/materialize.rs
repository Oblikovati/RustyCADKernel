// SPDX-License-Identifier: GPL-2.0-only
//! Fixed names label already resolved contexts; existing history still validates every relation.
use super::*;
use crate::sketch_region_prism::PrismBodyContext;
use crate::sketch_region_prism::frozen::{AuthoredPrismId, PrismDirectedBinding};
pub(super) fn authored(
    plan: &SingleBoundaryNamePlan,
    stage: SemanticNameStage,
    role: SemanticNameRole,
) -> ResolveCheck<AuthoredPrismId> {
    let token = SemanticNameSlot { stage, role }.token().value();
    plan.reverse(token).map_err(ResolutionCause::Names)?;
    Ok(AuthoredPrismId(token))
}
pub(super) fn historical(
    plan: &SingleBoundaryNamePlan,
    history_stage: PrismHistoryStage,
    stage: SemanticNameStage,
    rows: &[NamedSourceContext],
) -> ResolveCheck<PrismHistoricalStageBindings> {
    let mut result = PrismHistoricalStageBindings {
        stage: history_stage,
        members: vec![],
        uses: vec![],
    };
    for row in rows {
        append_historical_role(&mut result, plan, stage, row)?;
    }
    Ok(result)
}
pub(super) fn directed(
    plan: &SingleBoundaryNamePlan,
    stage: SemanticNameStage,
    named: &[NamedSourceContext],
    live: &[PrismBodyContext],
) -> ResolveCheck<Vec<PrismDirectedBinding>> {
    complete_context_coverage(named, &current::snapshot_contexts(live)?)?;
    let mut result = Vec::new();
    for row in named {
        result.push(PrismDirectedBinding {
            actual: selected_copy::unique_live(live, &row.actual)?.clone(),
            authored_use: authored(plan, stage, row.directed_use)?,
        });
    }
    Ok(result)
}
pub(super) fn intermediates(
    plan: &SingleBoundaryNamePlan,
    roles: &roster::ResolvedStageRoles,
) -> ResolveCheck<PrismHistoryBindings> {
    let stages = intermediate_rows(roles);
    let intermediate = stages
        .into_iter()
        .map(|(history, stage, rows)| historical(plan, history, stage, rows))
        .collect::<ResolveCheck<Vec<_>>>()?;
    Ok(PrismHistoryBindings {
        complementary_region: authored(
            plan,
            SemanticNameStage::Complement,
            SemanticNameRole::ComplementRegion,
        )?,
        intermediate,
    })
}
fn intermediate_rows(
    roles: &roster::ResolvedStageRoles,
) -> [(PrismHistoryStage, SemanticNameStage, &[NamedSourceContext]); 4] {
    use PrismHistoryStage as H;
    use SemanticNameStage as S;
    [
        (H::InputCopy, S::InputCopy, &roles.early.input_copy),
        (H::Partition, S::Partition, &roles.early.partition),
        (
            H::CandidateCopy,
            S::CandidateCopy,
            &roles.early.candidate_copy,
        ),
        (H::Complement, S::Complement, &roles.complement),
    ]
}

fn append_historical_role(
    result: &mut PrismHistoricalStageBindings,
    plan: &SingleBoundaryNamePlan,
    stage: SemanticNameStage,
    row: &NamedSourceContext,
) -> ResolveCheck<()> {
    append_historical_member(
        result,
        &row.actual.occurrence,
        authored(plan, stage, row.member)?,
    )?;
    result.uses.push(PrismHistoricalUseBinding {
        actual: row.actual.clone(),
        authored_use: authored(plan, stage, row.directed_use)?,
    });
    Ok(())
}
