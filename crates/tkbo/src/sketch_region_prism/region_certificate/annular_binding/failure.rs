// SPDX-License-Identifier: GPL-2.0-only
//! Native, source and chart causes remain typed; correspondence alone does not complete a body.
use super::*;
use rustycad_tkernel::standard::Failure;
pub(in crate::sketch_region_prism) struct AnnularBindingFailure<'a> {
    pub operation: u64,
    pub selected_candidate: u64,
    pub stage: AnnularBindingStage,
    pub requested_region: TwoLoopRegionOptions,
    pub operation_limits: super::super::super::frozen::FrozenPrismLimits,
    pub cause: Box<AnnularBindingCause<'a>>,
}
pub(in crate::sketch_region_prism) type AnnularBindingResult<'a, T> =
    Result<T, Box<AnnularBindingFailure<'a>>>;
pub(super) fn attributed<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    receipt: &PrismReceipt,
    stage: AnnularBindingStage,
    cause: Box<AnnularBindingCause<'a>>,
) -> Box<AnnularBindingFailure<'a>> {
    Box::new(AnnularBindingFailure {
        operation: request.identities.operation,
        selected_candidate: request.identities.selected,
        stage,
        requested_region: request.requested,
        operation_limits: super::super::super::frozen::capture_prism_limits(receipt),
        cause,
    })
}
pub(in crate::sketch_region_prism) enum AnnularBindingCause<'a> {
    Region(Box<TwoLoopRegionFailure<'a>>),
    Chart(Box<super::super::current_binding::PrismBindingFailure>),
    History(Box<super::super::super::frozen::complete_history::PrismHistoryCause>),
    Native(Failure),
    Capture(super::super::super::surface_charts::PrismSurfaceCause),
    Body(Box<PrismCause>),
    Count {
        required: &'static str,
        actual: usize,
        expected: usize,
    },
    Cancelled(AnnularBindingStage),
    Callback {
        stage: AnnularBindingStage,
        cause: Failure,
    },
    Stopped {
        callback: Box<AnnularBindingCause<'a>>,
        observed: Option<Box<AnnularBindingCause<'a>>>,
    },
}
pub(super) type AnnularCheck<'a, T = ()> = Result<T, Box<AnnularBindingCause<'a>>>;
pub(super) fn require(
    required: &'static str,
    actual: usize,
    expected: usize,
) -> AnnularCheck<'static> {
    if actual != expected {
        return Err(Box::new(AnnularBindingCause::Count {
            required,
            actual,
            expected,
        }));
    }
    Ok(())
}
pub(super) fn holds(required: &'static str, actual: bool) -> AnnularCheck<'static> {
    require(required, usize::from(actual), 1)
}
impl From<rustycad_tkernel::standard::Failure> for Box<AnnularBindingCause<'_>> {
    fn from(cause: Failure) -> Self {
        Box::new(AnnularBindingCause::Native(cause))
    }
}
impl From<super::super::super::surface_charts::PrismSurfaceCause> for Box<AnnularBindingCause<'_>> {
    fn from(cause: super::super::super::surface_charts::PrismSurfaceCause) -> Self {
        Box::new(AnnularBindingCause::Capture(cause))
    }
}
