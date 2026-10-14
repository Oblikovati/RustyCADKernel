// SPDX-License-Identifier: GPL-2.0-only
//! Reuse historical observation projection against the caller's already-bound body.
use super::super::bindings::BoundProjection;
use super::*;

/// Project lineage without callbacks or another freeze; `project_prism_lineage(&bound,&projection,&copy_names)`.
/// The caller owns current geometry/evidence validation; this only checks shared output identity.
pub(in crate::sketch_region_prism::frozen) fn project_prism_lineage(
    bound: &BoundProjection<'_>,
    projection: &FrozenPrismProjection,
    authored: &PrismCopyBindings,
) -> PrismLineageOutcome {
    let inputs = LineageInputs {
        receipt: bound.receipt,
        publication: bound.bindings,
        authored,
    };
    let mut result = capture::empty(bound.bindings.operation);
    if let Err(finding) = capture_bound(&inputs, bound, projection, &mut result) {
        return capture::declined(result, finding);
    }
    PrismLineageOutcome::Inspected(Box::new(result))
}
fn capture_bound(
    inputs: &LineageInputs<'_>,
    bound: &BoundProjection<'_>,
    projection: &FrozenPrismProjection,
    result: &mut PrismLineageInspection,
) -> LineageCheck<()> {
    same_output_identity(inputs, bound, projection)?;
    let contexts = capture::capture_contexts(inputs, result)?;
    result.sweep = sweep::capture_bound_sweep(inputs, &contexts, bound)?;
    capture::finish_observations(inputs, result)
}
fn same_output_identity(
    inputs: &LineageInputs<'_>,
    bound: &BoundProjection<'_>,
    projection: &FrozenPrismProjection,
) -> LineageCheck<()> {
    let mut expected = bound.use_names.iter().collect::<Vec<_>>();
    let mut actual = projection.uses().iter().map(|u| &u.key).collect::<Vec<_>>();
    expected.sort();
    actual.sort();
    if projection.operation != bound.bindings.operation
        || projection.region != bound.bindings.selected_region
        || actual != expected
    {
        return Err(inputs.failure(
            CopyLineagePhase::Frozen,
            CopyLineageCause::Changed {
                operand: "same-operation selected region and complete frozen directed keys",
            },
        ));
    }
    Ok(())
}
