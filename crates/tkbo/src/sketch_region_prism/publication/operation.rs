// SPDX-License-Identifier: GPL-2.0-only
//! A prospective result remains local until the final callback-free source and history checks.
use super::{history, progress::*, replay, request::*, result::*, stage_error::*, stages};
use crate::sketch_region_prism::frozen::proof_join::JoinedPrismProof;

/// Assemble one named operation; `publish_named_prism(&request, &mut progress)` never returns a partial body.
pub(crate) fn publish_named_prism<'a>(
    request: &PrismPublicationRequest<'a>,
    progress: &mut dyn PrismPublicationProgress,
) -> PublicationCheck<'a, PublishedNamedPrism> {
    publish_with_version(request, progress, 1)
}
/// Retain complete downstream callback diagnostics; `publish_named_prism_v2(&request, &mut progress)` requires version 2.
pub(crate) fn publish_named_prism_v2<'a>(
    request: &PrismPublicationRequest<'a>,
    progress: &mut dyn PrismPublicationProgress,
) -> PublicationCheck<'a, PublishedNamedPrism> {
    publish_with_version(request, progress, 2)
}
fn publish_with_version<'a>(
    request: &PrismPublicationRequest<'a>,
    progress: &mut dyn PrismPublicationProgress,
    version: u32,
) -> PublicationCheck<'a, PublishedNamedPrism> {
    admit(request, version)?;
    let mut callbacks = callbacks_for_version(progress, version);
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
fn callbacks_for_version(
    progress: &mut dyn PrismPublicationProgress,
    version: u32,
) -> PublicationCallbacks<'_> {
    if version == 2 {
        return PublicationCallbacks::retaining_children(progress);
    }
    PublicationCallbacks::new(progress)
}
fn admit<'a>(request: &PrismPublicationRequest<'a>, version: u32) -> PublicationCheck<'a, ()> {
    if request.requested.version != version {
        return Err(request.failure(
            PrismPublicationPhase::Admission,
            PrismPublicationClass::Incomplete,
            PrismPublicationCause::UnsupportedVersion {
                actual: request.requested.version,
                expected: version,
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
    capture_children(request, join, callbacks)
}
fn capture_children<'a>(
    request: &PrismPublicationRequest<'a>,
    join: JoinedPrismProof,
    callbacks: &mut PublicationCallbacks<'_>,
) -> PublicationCheck<'a, PublishedNamedPrism> {
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
