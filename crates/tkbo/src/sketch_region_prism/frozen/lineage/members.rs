// SPDX-License-Identifier: GPL-2.0-only
//! Exact member observations compose lineage; equal parent sets never establish identity.
use super::*;
use crate::sketch_region::ArrangementRelation;

pub(super) fn capture_members(
    inputs: &LineageInputs<'_>,
) -> LineageCheck<Vec<CopyMemberObservation>> {
    let observations = inputs.observations()?;
    check_domain(inputs, observations)?;
    let mut result = Vec::new();
    for observation in observations {
        let key = names::member(
            inputs,
            &observation.selected.output,
            CopyLineageStage::Selected,
        )?;
        result.push(capture_member(inputs, observation).map_err(|e| names::attributed(e, key))?);
    }
    result.sort();
    Ok(result)
}
fn check_domain(
    inputs: &LineageInputs<'_>,
    observations: &[PrismCopyObservation],
) -> LineageCheck<()> {
    let selected = inputs
        .receipt
        .input
        .selected
        .as_ref()
        .ok_or_else(|| inputs.count(CopyLineagePhase::Members, "selected candidate", 0, 1))?;
    check_relation_counts(inputs, observations.len(), selected.relations.len())?;
    super::super::domain::check_count(observations.len())
        .map_err(|e| inputs.frozen(CopyLineagePhase::Members, e))?;
    for stage in [CopyLineageStage::Selected, CopyLineageStage::Copied] {
        check_membership(inputs, &names::contexts(inputs, stage)?, stage)?;
    }
    check_authored_members(inputs, &selected.relations)
}
fn check_relation_counts(
    inputs: &LineageInputs<'_>,
    actual: usize,
    selected: usize,
) -> LineageCheck<()> {
    let copied = inputs
        .receipt
        .copied
        .as_ref()
        .expect("checked observations");
    count(inputs, "selected relation count", actual, selected)?;
    count(
        inputs,
        "copied relation count",
        actual,
        copied.relations.len(),
    )
}
fn check_authored_members(
    inputs: &LineageInputs<'_>,
    rows: &[ArrangementRelation],
) -> LineageCheck<()> {
    check_binding_duplicates(inputs, &inputs.authored.selected_members)?;
    check_binding_duplicates(inputs, &inputs.publication.sources)?;
    for binding in &inputs.authored.selected_members {
        let used = rows.iter().any(|row| {
            super::super::identities::same_definition(
                &binding.actual,
                &crate::sketch_region::PartitionOccurrenceIdentity::capture(&row.output),
            )
        });
        count(
            inputs,
            "authored selected member used by actual selected relation",
            usize::from(used),
            1,
        )?;
    }
    Ok(())
}
fn check_binding_duplicates(
    inputs: &LineageInputs<'_>,
    bindings: &[PrismSourceBinding],
) -> LineageCheck<()> {
    for binding in bindings {
        let actual = bindings
            .iter()
            .filter(|other| other.actual == binding.actual)
            .count();
        count(
            inputs,
            "one authored binding per actual oriented member",
            actual,
            1,
        )?;
    }
    Ok(())
}
fn check_membership(
    inputs: &LineageInputs<'_>,
    contexts: &[PrismBodyContext],
    stage: CopyLineageStage,
) -> LineageCheck<()> {
    for observation in inputs.observations()? {
        let shape = observed_shape(observation, stage);
        let key = names::member(inputs, shape, stage)?;
        check_observation_membership(inputs, contexts, shape, stage)
            .map_err(|e| names::attributed(e, key))?;
    }
    check_context_coverage(inputs, contexts, stage)
}
fn check_context_coverage(
    inputs: &LineageInputs<'_>,
    contexts: &[PrismBodyContext],
    stage: CopyLineageStage,
) -> LineageCheck<()> {
    for context in contexts {
        let covered = inputs
            .observations()?
            .iter()
            .any(|o| observed_shape(o, stage).is_same(&context.occurrence));
        let key = names::member(inputs, &context.occurrence, stage)?;
        count(
            inputs,
            "copy observation covers actual member",
            usize::from(covered),
            1,
        )
        .map_err(|e| names::attributed(e, key))?;
    }
    Ok(())
}
fn observed_shape(observation: &PrismCopyObservation, stage: CopyLineageStage) -> &Shape {
    match stage {
        CopyLineageStage::Selected => &observation.selected.output,
        CopyLineageStage::Copied => &observation.copied_image,
    }
}
fn check_observation_membership(
    inputs: &LineageInputs<'_>,
    contexts: &[PrismBodyContext],
    shape: &Shape,
    stage: CopyLineageStage,
) -> LineageCheck<()> {
    let belongs = contexts.iter().any(|c| c.occurrence.is_equal(shape));
    count(
        inputs,
        "observed member belongs to current operation root",
        usize::from(belongs),
        1,
    )?;
    let rows = inputs
        .observations()?
        .iter()
        .filter(|o| observed_shape(o, stage).is_equal(shape))
        .count();
    count(inputs, "one observation per oriented member", rows, 1)
}
fn capture_member(
    inputs: &LineageInputs<'_>,
    observed: &PrismCopyObservation,
) -> LineageCheck<CopyMemberObservation> {
    check_images(inputs, observed)?;
    let selected = inputs
        .receipt
        .input
        .selected
        .as_ref()
        .expect("checked domain");
    let copied = inputs.receipt.copied.as_ref().expect("checked domain");
    let original = relation(inputs, &selected.relations, &observed.selected.output)?;
    let actual = relation(inputs, &copied.relations, &observed.copied_image)?;
    compare_parents(inputs, original, &observed.selected)?;
    let expected = crate::sketch_region_candidates::composed_copy_relation(
        &observed.selected,
        &observed.copied_image,
    );
    compare_parents(inputs, actual, &expected)?;
    owned_member_record(inputs, observed, actual)
}
pub(super) fn owned_member_record(
    inputs: &LineageInputs<'_>,
    observed: &PrismCopyObservation,
    actual: &ArrangementRelation,
) -> LineageCheck<CopyMemberObservation> {
    Ok(CopyMemberObservation {
        selected: names::occurrence(
            inputs,
            &observed.selected.output,
            CopyLineageStage::Selected,
        )?,
        native_image: names::occurrence(inputs, &observed.native_image, CopyLineageStage::Copied)?,
        copied: names::occurrence(inputs, &observed.copied_image, CopyLineageStage::Copied)?,
        selected_parents: names::parents(inputs, &observed.selected)?,
        copied_parents: names::parents(inputs, actual)?,
    })
}
fn check_images(inputs: &LineageInputs<'_>, observed: &PrismCopyObservation) -> LineageCheck<()> {
    let same = observed.native_image.is_same(&observed.copied_image);
    count(
        inputs,
        "native image and oriented copy share exact definition/placement",
        usize::from(same),
        1,
    )?;
    let oriented = observed.selected.output.orientation() == observed.copied_image.orientation();
    count(
        inputs,
        "copied query orientation preserved",
        usize::from(oriented),
        1,
    )?;
    check_bijection(inputs, observed)
}
fn check_bijection(
    inputs: &LineageInputs<'_>,
    observed: &PrismCopyObservation,
) -> LineageCheck<()> {
    for other in inputs.observations()? {
        let same_query = observed.selected.output.is_same(&other.selected.output);
        let same_copy = observed.copied_image.is_same(&other.copied_image);
        count(
            inputs,
            "one-to-one actual member copy map",
            usize::from(same_query == same_copy),
            1,
        )?;
    }
    Ok(())
}
fn relation<'a>(
    inputs: &LineageInputs<'_>,
    rows: &'a [ArrangementRelation],
    member: &Shape,
) -> LineageCheck<&'a ArrangementRelation> {
    let found: Vec<_> = rows
        .iter()
        .filter(|row| row.output.is_equal(member))
        .collect();
    count(
        inputs,
        "one actual oriented member relation",
        found.len(),
        1,
    )?;
    Ok(found[0])
}
pub(super) fn compare_parents(
    inputs: &LineageInputs<'_>,
    actual: &ArrangementRelation,
    expected: &ArrangementRelation,
) -> LineageCheck<()> {
    for (class, (actual, expected)) in names::parents(inputs, actual)?
        .into_iter()
        .zip(names::parents(inputs, expected)?)
        .enumerate()
    {
        if actual != expected {
            return Err(inputs.failure(
                CopyLineagePhase::Members,
                CopyLineageCause::Parents {
                    class: CopyParentClass::ALL[class],
                    actual,
                    expected,
                },
            ));
        }
    }
    Ok(())
}
fn count(
    inputs: &LineageInputs<'_>,
    required: &'static str,
    actual: usize,
    expected: usize,
) -> LineageCheck<()> {
    inputs.require(CopyLineagePhase::Members, required, actual, expected)
}
