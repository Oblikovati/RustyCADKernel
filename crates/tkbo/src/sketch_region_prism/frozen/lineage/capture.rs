// SPDX-License-Identifier: GPL-2.0-only
//! Reproject historical operands after the existing freeze entry's final external callback.
use super::*;

/// Inspect observed copy/sweep lineage; example: `inspect_prism_lineage(&receipt,&names,&copy_names,&mut progress)`.
/// Historical query geometry is unproven; this never completes validation or a valid body.
/// Inputs are operation-local: concurrent alias writes during inspection are unsupported.
pub(crate) fn inspect_prism_lineage(
    receipt: &PrismReceipt,
    publication: &PrismPublicationBindings,
    authored: &PrismCopyBindings,
    progress: &mut impl FreezeProgress,
) -> PrismLineageOutcome {
    let inputs = LineageInputs {
        receipt,
        publication,
        authored,
    };
    let mut result = empty(publication.operation);
    if let Err(finding) = capture_lineage(&inputs, &mut result) {
        return declined(result, finding);
    }
    freeze_then_finish(&inputs, result, progress)
}
fn freeze_then_finish(
    inputs: &LineageInputs<'_>,
    result: PrismLineageInspection,
    progress: &mut impl FreezeProgress,
) -> PrismLineageOutcome {
    let original = result.lineage_bytes();
    let projection = match freeze_prism(inputs.receipt, inputs.publication, progress) {
        PrismFreezeOutcome::Captured(projection) => projection,
        PrismFreezeOutcome::Cancelled { operation, phase } => {
            return PrismLineageOutcome::Cancelled { operation, phase };
        }
        PrismFreezeOutcome::Incomplete(error) | PrismFreezeOutcome::Failed(error) => {
            return declined(
                result,
                inputs.failure(
                    CopyLineagePhase::Frozen,
                    CopyLineageCause::FreezeFailure(Box::new(error)),
                ),
            );
        }
    };
    finish(inputs, result, original, projection)
}
fn finish(
    inputs: &LineageInputs<'_>,
    mut result: PrismLineageInspection,
    before: Vec<u8>,
    projection: Box<FrozenPrismProjection>,
) -> PrismLineageOutcome {
    let mut current = empty(inputs.publication.operation);
    if let Err(finding) = capture_lineage(inputs, &mut current) {
        return declined(result, finding);
    }
    if before != current.lineage_bytes() {
        return declined(
            result,
            inputs.failure(
                CopyLineagePhase::Commit,
                CopyLineageCause::Changed {
                    operand: "selected/copied contexts, observations, relations or retirement",
                },
            ),
        );
    }
    result.projection = Some(projection);
    PrismLineageOutcome::Inspected(Box::new(result))
}
fn capture_lineage(
    inputs: &LineageInputs<'_>,
    result: &mut PrismLineageInspection,
) -> LineageCheck<()> {
    admit_lineage(inputs)?;
    result.members = members::capture_members(inputs)?;
    result.member_coverage = CopyLineageCoverage::Complete;
    result.directed_coverage = CopyLineageCoverage::Incomplete;
    let contexts = directed::bind_contexts(inputs)?;
    result.directed = directed::pair_contexts(inputs, &contexts)?;
    result.contexts = directed::owned_contexts(inputs, &contexts)?;
    result.sweep = sweep::capture_sweep(inputs, &contexts)?;
    result.retired = sweep::capture_retirement(inputs)?;
    result.directed_coverage = CopyLineageCoverage::Complete;
    Ok(())
}
fn empty(operation: AuthoredPrismId) -> PrismLineageInspection {
    PrismLineageInspection {
        operation,
        member_coverage: CopyLineageCoverage::Incomplete,
        directed_coverage: CopyLineageCoverage::Blocked,
        members: Vec::new(),
        contexts: Vec::new(),
        directed: Vec::new(),
        sweep: Vec::new(),
        retired: Vec::new(),
        findings: Vec::new(),
        projection: None,
    }
}
fn declined(
    mut result: PrismLineageInspection,
    finding: CopyLineageFinding,
) -> PrismLineageOutcome {
    let native = native_cause(&finding.cause);
    if matches!(
        finding.phase,
        CopyLineagePhase::Members | CopyLineagePhase::Frozen | CopyLineagePhase::Commit
    ) {
        result.member_coverage = CopyLineageCoverage::Incomplete;
        result.directed_coverage = CopyLineageCoverage::Blocked;
    }
    if matches!(
        finding.phase,
        CopyLineagePhase::Directed | CopyLineagePhase::Sweep
    ) {
        result.directed_coverage = CopyLineageCoverage::Incomplete;
    }
    result.findings.push(finding);
    if native {
        return PrismLineageOutcome::Failed(Box::new(result));
    }
    PrismLineageOutcome::Incomplete(Box::new(result))
}

fn native_cause(cause: &CopyLineageCause) -> bool {
    match cause {
        CopyLineageCause::Frozen(cause) => cause.native_failure(),
        CopyLineageCause::FreezeFailure(failure) => failure.cause.native_failure(),
        _ => false,
    }
}

fn admit_lineage(inputs: &LineageInputs<'_>) -> LineageCheck<()> {
    super::super::admission::admit_receipt(inputs.receipt)
        .map_err(|e| inputs.frozen(CopyLineagePhase::Frozen, e))?;
    let counts = [
        ("copy observations", inputs.observations()?.len()),
        ("selected members", inputs.authored.selected_members.len()),
        ("copied members", inputs.publication.sources.len()),
        ("selected uses", inputs.authored.selected_uses.len()),
        ("copied uses", inputs.authored.copied_uses.len()),
        ("output uses", inputs.publication.directed.len()),
    ];
    for (quantity, actual) in counts {
        inputs.limit(quantity, actual)?;
    }
    Ok(())
}
