// SPDX-License-Identifier: GPL-2.0-only
//! Existing observed sweep roles connect copied uses to the same frozen publication keys.
use super::super::super::{PrismImageBinding, PrismImageRole};
use super::*;
use crate::sketch_region::ArrangementRelation;

pub(super) fn capture_sweep(
    inputs: &LineageInputs<'_>,
    copied: &directed::BoundCopyContexts,
) -> LineageCheck<Vec<CopySweepObservation>> {
    let bound = super::super::bindings::bind_projection(inputs.receipt, inputs.publication)
        .map_err(|e| inputs.frozen(CopyLineagePhase::Sweep, e))?;
    capture_bound_sweep(inputs, copied, &bound)
}
pub(super) fn capture_bound_sweep(
    inputs: &LineageInputs<'_>,
    copied: &directed::BoundCopyContexts,
    bound: &super::super::bindings::BoundProjection<'_>,
) -> LineageCheck<Vec<CopySweepObservation>> {
    let mut result = Vec::new();
    for binding in &bound.diagnostic.bindings {
        result.push(sweep_row(inputs, copied, bound, binding)?);
    }
    result.sort();
    if result.windows(2).any(|rows| rows[0] == rows[1]) {
        return Err(inputs.count(
            CopyLineagePhase::Sweep,
            "one native query/role/output observation",
            2,
            1,
        ));
    }
    Ok(result)
}
fn sweep_row(
    inputs: &LineageInputs<'_>,
    copied: &directed::BoundCopyContexts,
    bound: &super::super::bindings::BoundProjection<'_>,
    binding: &PrismImageBinding,
) -> LineageCheck<CopySweepObservation> {
    let query = query_index(inputs, copied, &binding.native.query)?;
    let key = names::member(inputs, &binding.native.query, CopyLineageStage::Copied)?;
    check_sweep_parents(inputs, binding).map_err(|e| names::attributed(e, key))?;
    let output = output_index(inputs, bound, &binding.context)?;
    Ok(CopySweepObservation {
        copied: copied.copied_keys[query].clone(),
        output: bound.use_names[output].clone(),
        role: binding.native.role.into(),
        parents: names::parents(inputs, &binding.native.returned)?,
    })
}
fn query_index(
    inputs: &LineageInputs<'_>,
    copied: &directed::BoundCopyContexts,
    query: &Shape,
) -> LineageCheck<usize> {
    let queries: Vec<_> = copied
        .copied
        .iter()
        .enumerate()
        .filter(|(_, c)| c.occurrence.is_equal(query))
        .map(|(i, _)| i)
        .collect();
    count(
        inputs,
        "one actual copied directed context for native sweep query",
        queries.len(),
    )?;
    Ok(queries[0])
}
fn output_index(
    inputs: &LineageInputs<'_>,
    bound: &super::super::bindings::BoundProjection<'_>,
    context: &PrismBodyContext,
) -> LineageCheck<usize> {
    let outputs: Vec<_> = bound
        .contexts
        .iter()
        .enumerate()
        .filter(|(_, c)| super::super::identities::same_context(c, context))
        .map(|(i, _)| i)
        .collect();
    count(
        inputs,
        "one current output context for native sweep observation",
        outputs.len(),
    )?;
    Ok(outputs[0])
}
fn check_sweep_parents(
    inputs: &LineageInputs<'_>,
    binding: &PrismImageBinding,
) -> LineageCheck<()> {
    let copied = copied_relation(inputs, &binding.native.query)?;
    let output = &binding.native.returned.output;
    let expected = match binding.native.role {
        PrismImageRole::Generated => {
            super::super::super::history::generated_relation(output, copied)
        }
        PrismImageRole::First | PrismImageRole::Last => {
            crate::sketch_region_candidates::composed_copy_relation(copied, output)
        }
    };
    check_parents(inputs, &binding.native.returned, &expected)
}
fn copied_relation<'a>(
    inputs: &'a LineageInputs<'_>,
    shape: &Shape,
) -> LineageCheck<&'a ArrangementRelation> {
    let copied = inputs
        .receipt
        .copied
        .as_ref()
        .expect("member phase checked");
    let rows: Vec<_> = copied
        .relations
        .iter()
        .filter(|r| r.output.is_equal(shape))
        .collect();
    count(inputs, "one copied relation for sweep query", rows.len())?;
    Ok(rows[0])
}
fn check_parents(
    inputs: &LineageInputs<'_>,
    actual: &ArrangementRelation,
    expected: &ArrangementRelation,
) -> LineageCheck<()> {
    members::compare_parents(inputs, actual, expected).map_err(|mut finding| {
        finding.phase = CopyLineagePhase::Sweep;
        finding
    })
}
pub(super) fn capture_retirement(
    inputs: &LineageInputs<'_>,
) -> LineageCheck<Vec<CopyRetirementObservation>> {
    let diagnostic = inputs
        .receipt
        .diagnostic
        .as_ref()
        .ok_or_else(|| inputs.count(CopyLineagePhase::Sweep, "sweep diagnostic", 0, 1))?;
    let mut result = Vec::new();
    for row in &diagnostic.native_retired {
        let key = names::member(inputs, &row.output, CopyLineageStage::Copied)?;
        result.push(retirement_row(inputs, row).map_err(|e| names::attributed(e, key))?);
    }
    result.sort();
    if result.windows(2).any(|rows| rows[0] == rows[1]) {
        return Err(inputs.count(
            CopyLineagePhase::Sweep,
            "one native retirement per copied member",
            2,
            1,
        ));
    }
    Ok(result)
}
fn retirement_row(
    inputs: &LineageInputs<'_>,
    row: &ArrangementRelation,
) -> LineageCheck<CopyRetirementObservation> {
    let matches = inputs
        .observations()?
        .iter()
        .filter(|o| o.copied_image.is_equal(&row.output))
        .count();
    count(
        inputs,
        "one copied intermediate for native retirement",
        matches,
    )?;
    check_parents(inputs, row, copied_relation(inputs, &row.output)?)?;
    Ok(CopyRetirementObservation {
        copied: names::occurrence(inputs, &row.output, CopyLineageStage::Copied)?,
        parents: names::parents(inputs, row)?,
    })
}
fn count(inputs: &LineageInputs<'_>, required: &'static str, actual: usize) -> LineageCheck<()> {
    inputs.require(CopyLineagePhase::Sweep, required, actual, 1)
}
