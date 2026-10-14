// SPDX-License-Identifier: GPL-2.0-only
//! Exact identities are checked in addition to portable operands; equal geometry is not identity.
use super::*;

pub(in crate::sketch_region_prism) fn rebind_annular_prism_v2<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    receipt: &PrismReceipt,
    binding: &AnnularPrismBinding,
) -> AnnularBindingResult<'a, ()> {
    check(request, receipt, binding)
        .map_err(|cause| attributed(request, receipt, AnnularBindingStage::Commit, cause))
}
pub(super) fn check<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    receipt: &PrismReceipt,
    binding: &AnnularPrismBinding,
) -> AnnularCheck<'a> {
    operation::preflight(request, receipt)?;
    operation::region_rebind(request, &binding.region)?;
    let fresh = operation::capture(
        request,
        &binding.region,
        receipt,
        &mut operation::CallbackFree,
    )?;
    same_contexts(&binding.operands, &fresh)?;
    holds(
        "all current annular binding operands and policies",
        replay::encode(&binding.region, &fresh)? == binding.replay,
    )?;
    Ok(())
}
fn same_contexts(old: &AnnularPrismOperands, new: &AnnularPrismOperands) -> AnnularCheck<'static> {
    holds(
        "same hierarchy budget and exact census",
        old.hierarchy == new.hierarchy,
    )?;
    holds(
        "same complete original copy and native image parent classes",
        old.parents.copies == new.parents.copies
            && old.parents.images == new.parents.images
            && old.parents.captured == new.parents.captured,
    )?;
    holds(
        "same complete current body context census",
        old.body_contexts == new.body_contexts,
    )?;
    require(
        "same complete copied context census",
        old.copy.members.len(),
        new.copy.members.len(),
    )?;
    for (first, second) in old.copy.members.iter().zip(&new.copy.members) {
        holds(
            "same authored copied context association",
            first.key == second.key
                && first.selected == second.selected
                && first.copied == second.copied,
        )?;
    }
    require(
        "same complete actual chart universe",
        old.charts.len(),
        new.charts.len(),
    )?;
    for (first, second) in old.charts.iter().zip(&new.charts) {
        holds(
            "same source-output-role chart and trim contexts",
            first.identity == second.identity && first.trims == second.trims,
        )?;
        holds(
            "same complete original chart parents",
            first.chart.parents == second.chart.parents,
        )?;
    }
    Ok(())
}
