// SPDX-License-Identifier: GPL-2.0-only
//! Provisional actual stage roles are kept local until complete publication and final rebind.
use super::*;
use crate::sketch_region_prism::region_certificate::current_binding::CurrentPrismBinding;
pub(super) struct EarlyRoles {
    pub original: Vec<NamedSourceContext>,
    pub input_copy: Vec<NamedSourceContext>,
    pub partition: Vec<NamedSourceContext>,
    pub candidate_copy: Vec<NamedSourceContext>,
}
pub(super) struct ResolvedStageRoles {
    pub early: EarlyRoles,
    pub binding: CurrentPrismBinding,
    pub selected: Vec<NamedSourceContext>,
    pub complement: Vec<NamedSourceContext>,
    pub sweep_base: Vec<NamedSourceContext>,
}
pub(super) fn resolve_stages(
    prepared: &PreparedPolynomialPrism,
    plan: &SingleBoundaryNamePlan,
    cancelled: &mut impl FnMut() -> bool,
) -> ResolveCheck<ResolvedStageRoles> {
    plan::admit_plan(prepared, plan)?;
    let early = resolve_early(prepared)?;
    let binding = current::binding(prepared, cancelled)?;
    let (selected, complement) = resolve_regions(prepared, &binding, &early)?;
    let sweep_base = selected_copy::sweep_base_contexts(prepared, &binding, &selected)?;
    Ok(ResolvedStageRoles {
        early,
        binding,
        selected,
        complement,
        sweep_base,
    })
}
fn resolve_early(prepared: &PreparedPolynomialPrism) -> ResolveCheck<EarlyRoles> {
    let initial = operands::initial_copy(prepared)?;
    let original = source::original_contexts(prepared, &initial.source)?;
    let input_copy = copied::copied_contexts(initial, &original)?;
    let partition = partition::partition_contexts(prepared, &original, &input_copy)?;
    let candidate_copy = copied::copied_contexts(operands::candidate_copy(prepared)?, &partition)?;
    Ok(EarlyRoles {
        original,
        input_copy,
        partition,
        candidate_copy,
    })
}

fn resolve_regions(
    prepared: &PreparedPolynomialPrism,
    binding: &CurrentPrismBinding,
    early: &EarlyRoles,
) -> ResolveCheck<(Vec<NamedSourceContext>, Vec<NamedSourceContext>)> {
    let selected = generated::generated_contexts(
        prepared,
        binding,
        &early.candidate_copy,
        PrismHistoryStage::Selected,
    )?;
    let complement = generated::generated_contexts(
        prepared,
        binding,
        &early.candidate_copy,
        PrismHistoryStage::Complement,
    )?;
    Ok((selected, complement))
}
