// SPDX-License-Identifier: GPL-2.0-only
//! The finishing callback runs before invocation-local owners leave scope; no self-reference is needed.
use super::super::{
    OwnedPolynomialPrismRequest, PreparedPolynomialPrism, pipeline::prepare_with_admission,
    resolver::*,
};
use super::{admission::*, context::*, projection, result::*, scoped_failure::*};

/// Run one actual owned-input operation; `with_owned_prism_operation(request, progress, finish)`.
/// `finish` may return owned success or diagnostics, but cannot return a scoped local failure.
pub(crate) fn with_owned_prism_operation<R>(
    request: OwnedPolynomialPrismRequest,
    progress: &mut dyn CompletePrismProgress,
    finish: impl for<'a> FnOnce(Result<OwnedCompletePrism, ScopedPrismFailure<'a>>) -> R,
) -> R {
    let mut context = CompletePrismContext::new(progress);
    match prepare_with_admission(request, &mut context, admit_complete_names) {
        Ok(prepared) => run_prepared(&prepared, &mut context, finish),
        Err(cause) => finish(Err(refusal(
            ScopedPrismCause::Preparation(cause),
            &mut context,
        ))),
    }
}
fn run_prepared<R>(
    prepared: &PreparedPolynomialPrism,
    context: &mut CompletePrismContext<'_>,
    finish: impl for<'a> FnOnce(Result<OwnedCompletePrism, ScopedPrismFailure<'a>>) -> R,
) -> R {
    let result = admitted_invocation(prepared, context);
    match result {
        Ok(invocation) => run_invocation(&invocation, context, finish),
        Err(cause) => finish(Err(refusal(
            ScopedPrismCause::Resolution {
                prepared,
                cause: Box::new(cause),
            },
            context,
        ))),
    }
}
fn admitted_invocation<'a>(
    prepared: &'a PreparedPolynomialPrism,
    context: &mut CompletePrismContext<'_>,
) -> Result<ResolvedPrismInvocation<'a>, ResolutionCause> {
    let plan = prepared
        .retained
        .plan
        .clone()
        .ok_or(ResolutionCause::Missing {
            required: "V2 dictionary admitted before construction",
        })?;
    resolve_invocation(prepared, plan, &mut || {
        context.poll(CompletePrismPhase::Resolution, None)
    })
}
fn run_invocation<R>(
    invocation: &ResolvedPrismInvocation<'_>,
    context: &mut CompletePrismContext<'_>,
    finish: impl for<'a> FnOnce(Result<OwnedCompletePrism, ScopedPrismFailure<'a>>) -> R,
) -> R {
    match invocation.publish(context) {
        Ok(invoked) => finish(
            projection::project(invoked, context)
                .map_err(|cause| refusal(ScopedPrismCause::Projection(cause), context)),
        ),
        Err(cause) => finish(Err(refusal(ScopedPrismCause::Publication(cause), context))),
    }
}
fn refusal<'a>(
    cause: ScopedPrismCause<'a>,
    context: &mut CompletePrismContext<'_>,
) -> ScopedPrismFailure<'a> {
    ScopedPrismFailure {
        cause,
        callback: context.stop.take(),
    }
}
