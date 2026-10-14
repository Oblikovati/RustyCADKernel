// SPDX-License-Identifier: GPL-2.0-only
//! These local rows label actual operands; publication must still prove complete history.
use super::*;
#[derive(Clone)]
pub(super) struct NamedSourceContext {
    pub actual: RegionObservedContext,
    pub member: SemanticNameRole,
    pub directed_use: SemanticNameRole,
}
pub(super) enum ResolutionCause {
    PlanVersion {
        actual: u32,
        expected: u32,
    },
    PlanMismatch {
        actual: Box<SingleBoundaryNamePlan>,
        expected: Box<SingleBoundaryNamePlan>,
    },
    NativeParameter {
        actual: f64,
        expected: f64,
    },
    Names(SemanticNameFailure),
    History(Box<PrismHistoryCause>),
    Missing {
        required: &'static str,
    },
    Count {
        required: &'static str,
        actual: usize,
        expected: usize,
    },
}
pub(super) type ResolveCheck<T> = Result<T, ResolutionCause>;
impl From<PrismHistoryCause> for ResolutionCause {
    fn from(cause: PrismHistoryCause) -> Self {
        Self::History(Box::new(cause))
    }
}
pub(super) fn require_count(
    required: &'static str,
    actual: usize,
    expected: usize,
) -> ResolveCheck<()> {
    if actual != expected {
        return Err(ResolutionCause::Count {
            required,
            actual,
            expected,
        });
    }
    Ok(())
}
pub(super) fn require_local<T>(value: Option<T>, required: &'static str) -> ResolveCheck<T> {
    value.ok_or(ResolutionCause::Missing { required })
}
pub(super) fn complete_context_coverage(
    named: &[NamedSourceContext],
    actual: &[RegionObservedContext],
) -> ResolveCheck<()> {
    require_count("complete named context domain", named.len(), actual.len())?;
    for row in actual {
        require_count(
            "unique actual context in retained domain",
            actual.iter().filter(|other| *other == row).count(),
            1,
        )?;
        require_count(
            "one semantic role per actual context",
            named.iter().filter(|name| &name.actual == row).count(),
            1,
        )?;
    }
    Ok(())
}
