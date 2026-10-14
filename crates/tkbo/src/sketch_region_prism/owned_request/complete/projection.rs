// SPDX-License-Identifier: GPL-2.0-only
//! Pure projections reuse the one accepted publication; no fresh capture or detached map issuance.
use super::super::{
    names::{body_map::*, reservations::*},
    resolver::ResolvedPrismPublication,
};
use super::{context::*, result::*};
use crate::sketch_region_prism::{frozen::joined_volume::*, publication::body_view::*};
pub(crate) enum CompleteProjectionCause {
    Callback,
    Names(Box<PrismSemanticMapFailure>),
    Reservations(PrismReservationFailure),
    Body(Box<PrismBodyProjectionFailure>),
    Volume(Box<JoinedPrismVolumeFailure>),
}
pub(crate) struct ProjectionRefusal {
    pub(super) invoked: ResolvedPrismPublication,
    pub cause: CompleteProjectionCause,
}
pub(super) fn project(
    invoked: ResolvedPrismPublication,
    context: &mut CompletePrismContext<'_>,
) -> Result<OwnedCompletePrism, Box<ProjectionRefusal>> {
    match children(&invoked, context) {
        Ok(parts) => {
            let (request, publication) = invoked.into_parts();
            Ok(OwnedCompletePrism {
                request,
                publication,
                parts,
            })
        }
        Err(cause) => Err(Box::new(ProjectionRefusal { invoked, cause })),
    }
}
fn children(
    invoked: &ResolvedPrismPublication,
    context: &mut CompletePrismContext<'_>,
) -> Result<CompletePrismParts, CompleteProjectionCause> {
    poll(context, CompletePrismPhase::Names)?;
    let names = issue_resolved_prism_names(invoked).map_err(CompleteProjectionCause::Names)?;
    let (reservations, body) = body_parts(invoked, &names, context)?;
    let volume = physical_volume(invoked, context)?;
    poll(context, CompletePrismPhase::Commit)?;
    Ok(CompletePrismParts {
        names,
        reservations,
        body,
        volume,
    })
}
fn body_parts(
    invoked: &ResolvedPrismPublication,
    names: &ResolvedPrismNames,
    context: &mut CompletePrismContext<'_>,
) -> Result<(PrismReservationCoverage, OwnedNamedPrismBody), CompleteProjectionCause> {
    poll(context, CompletePrismPhase::Reservations)?;
    let reservations =
        inspect_reservations(invoked, names).map_err(CompleteProjectionCause::Reservations)?;
    poll(context, CompletePrismPhase::Body)?;
    let body =
        project_named_prism_body(invoked.result(), names).map_err(CompleteProjectionCause::Body)?;
    Ok((reservations, body))
}
fn physical_volume(
    invoked: &ResolvedPrismPublication,
    context: &mut CompletePrismContext<'_>,
) -> Result<JoinedPrismVolume, CompleteProjectionCause> {
    let result = invoked.result();
    interpret_joined_prism_volume(
        result.join(),
        result.validation().orientation(),
        result.measures(),
        1,
        || context.poll(CompletePrismPhase::Volume, None),
    )
    .map_err(|cause| CompleteProjectionCause::Volume(Box::new(cause)))
}
fn poll(
    context: &mut CompletePrismContext<'_>,
    phase: CompletePrismPhase,
) -> Result<(), CompleteProjectionCause> {
    if context.poll(phase, None) {
        return Err(CompleteProjectionCause::Callback);
    }
    Ok(())
}
