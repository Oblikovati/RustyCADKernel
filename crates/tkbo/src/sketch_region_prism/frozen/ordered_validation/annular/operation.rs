// SPDX-License-Identifier: GPL-2.0-only
use super::*;
/// Check topology, geometry, self-intersection and tolerance in order; `validate_annular_prism(join, options, progress)`.
/// Complete validation is a private prerequisite, not successful publication or complete history.
pub(in crate::sketch_region_prism) fn validate_annular_prism(
    join: &AnnularFrozenDomain,
    requested: AnnularValidationOptions,
    progress: &mut dyn AnnularValidationProgress,
) -> Result<AnnularValidationCertificate, Box<AnnularValidationFailure>> {
    let mut context = AnnularValidationContext::new(progress);
    let mut partial = AnnularValidationPartial::default();
    if let Err(stop) = validate(join, requested, &mut context, &mut partial) {
        return Err(refused(join, requested, context, partial, stop));
    }
    Ok(AnnularValidationCertificate {
        source: join.replay_bytes().into(),
        requested,
        trace: context.trace,
        proofs: partial,
        historical: join.projection().coverage().clone(),
    })
}
fn validate(
    join: &AnnularFrozenDomain,
    requested: AnnularValidationOptions,
    context: &mut AnnularValidationContext<'_>,
    partial: &mut AnnularValidationPartial,
) -> ValidationCheck<()> {
    admit(requested, context)?;
    partial.topology = Some(checks::topology(join, context)?);
    partial.orientation = Some(checks::orientation(join, context)?);
    partial.conformance = Some(checks::conformance(join, context)?);
    partial.intersection = Some(checks::intersection(join, context)?);
    complete_tolerance(join, requested.stored_cap, context, partial)
}
fn complete_tolerance(
    join: &AnnularFrozenDomain,
    cap: StoredToleranceCap,
    context: &mut AnnularValidationContext<'_>,
    partial: &mut AnnularValidationPartial,
) -> ValidationCheck<()> {
    let geometry = partial
        .conformance
        .as_ref()
        .expect("geometry stage completed before tolerance");
    partial.tolerance = Some(checks::tolerance(join, geometry, cap, context)?);
    context.checkpoint(AnnularValidationCheck::Commit)?;
    context.passed(OrderedPrismStage::Tolerance);
    Ok(())
}
fn admit(
    requested: AnnularValidationOptions,
    context: &mut AnnularValidationContext<'_>,
) -> ValidationCheck<()> {
    if requested.version != 2 {
        return Err(ValidationStop::incomplete(
            AnnularValidationCause::Version {
                actual: requested.version,
                expected: 2,
            },
        ));
    }
    context.checkpoint(AnnularValidationCheck::Admission)
}
fn refused(
    join: &AnnularFrozenDomain,
    requested: AnnularValidationOptions,
    mut context: AnnularValidationContext<'_>,
    partial: AnnularValidationPartial,
    stop: ValidationStop,
) -> Box<AnnularValidationFailure> {
    context.refused(stop.class);
    Box::new(AnnularValidationFailure {
        operation: join.projection().operation,
        requested,
        checkpoint: context.current,
        class: stop.class,
        trace: context.trace,
        cause: stop.cause,
        source: join.replay_bytes().into(),
        partial,
    })
}
