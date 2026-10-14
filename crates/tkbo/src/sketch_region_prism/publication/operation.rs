// SPDX-License-Identifier: GPL-2.0-only
//! A prospective result remains local until the final callback-free source and history checks.
use super::{history, progress::*, replay, request::*, result::*, stage_error::*, stages};

/// Assemble one named operation; `publish_named_prism(&request, &mut progress)` never returns a partial body.
pub(crate) fn publish_named_prism<'a>(
    request: &PrismPublicationRequest<'a>,
    progress: &mut dyn PrismPublicationProgress,
) -> PublicationCheck<'a, PublishedNamedPrism> {
    admit(request)?;
    let mut callbacks = PublicationCallbacks::new(progress);
    let mut result = construct(request, &mut callbacks)?;
    result.replay = replay::encode(&result);
    request.callback(PrismPublicationPhase::Commit, &mut callbacks)?;
    request
        .source
        .rebind(&result.join)
        .map_err(|cause| request.stage_failure(cause))?;
    history::reproject(request, &result)?;
    Ok(result)
}
fn admit<'a>(request: &PrismPublicationRequest<'a>) -> PublicationCheck<'a, ()> {
    if request.requested.version != 1 {
        return Err(request.failure(
            PrismPublicationPhase::Admission,
            PrismPublicationClass::Incomplete,
            PrismPublicationCause::UnsupportedVersion {
                actual: request.requested.version,
                expected: 1,
            },
        ));
    }
    Ok(())
}
fn construct<'a>(
    request: &PrismPublicationRequest<'a>,
    callbacks: &mut PublicationCallbacks<'_>,
) -> PublicationCheck<'a, PublishedNamedPrism> {
    let join = request
        .source
        .compose(request.requested.join, callbacks)
        .map_err(|cause| request.stage_failure(cause))?;
    let validation = stages::validation(&join, request.requested.validation, callbacks)
        .map_err(|cause| request.stage_failure(cause))?;
    let history = history::capture(request, &join, callbacks)?;
    let measures = stages::measures(&join, request.requested.measures, callbacks)
        .map_err(|cause| request.stage_failure(cause))?;
    Ok(PublishedNamedPrism {
        join,
        validation,
        history,
        measures,
        requested: request.requested,
        replay: Vec::new(),
    })
}
