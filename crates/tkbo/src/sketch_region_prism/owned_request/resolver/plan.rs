// SPDX-License-Identifier: GPL-2.0-only
//! V2 is explicitly supplied before lookup; no geometry result can extend the vocabulary.
use super::*;
pub(super) fn admit_plan(
    prepared: &PreparedPolynomialPrism,
    supplied: &SingleBoundaryNamePlan,
) -> ResolveCheck<()> {
    if supplied.version() != 2 {
        return Err(ResolutionCause::PlanVersion {
            actual: supplied.version(),
            expected: 2,
        });
    }
    let expected = expected_plan(prepared)?;
    if supplied != &expected {
        return Err(ResolutionCause::PlanMismatch {
            actual: Box::new(supplied.clone()),
            expected: Box::new(expected),
        });
    }
    Ok(())
}

fn expected_plan(prepared: &PreparedPolynomialPrism) -> ResolveCheck<SingleBoundaryNamePlan> {
    let request = &prepared.retained.request;
    require_count(
        "one authored boundary for semantic resolution",
        request.sources.len(),
        1,
    )?;
    let source = require_local(request.sources.first(), "one authored boundary")?;
    SingleBoundaryNamePlan::v2(source.names(request.operation), request.selection)
        .map_err(ResolutionCause::Names)
}
