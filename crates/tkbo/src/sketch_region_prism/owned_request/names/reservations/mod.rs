// SPDX-License-Identifier: GPL-2.0-only
//! Reservation usage is projected from accepted history and the sealed actual map, never assumed from tokens.
mod capture;
mod classification;
mod replay;
mod types;
use super::{body_map::ResolvedPrismNames, *};
use crate::sketch_region_prism::owned_request::resolver::ResolvedPrismPublication;
pub(crate) use types::*;

/// Inspect every reservation after accepted publication; `inspect_reservations(&invoked, &names)`.
pub(in crate::sketch_region_prism::owned_request) fn inspect_reservations(
    invoked: &ResolvedPrismPublication,
    names: &ResolvedPrismNames,
) -> Result<PrismReservationCoverage, PrismReservationFailure> {
    inspect_bound(invoked, names).map_err(|cause| PrismReservationFailure {
        operation: invoked.plan().identities().operation,
        cause,
    })
}
fn inspect_bound(
    invoked: &ResolvedPrismPublication,
    names: &ResolvedPrismNames,
) -> ReservationCheck<PrismReservationCoverage> {
    if names.plan() != invoked.plan() || !names.binds(invoked.result()) {
        return Err(PrismReservationCause::ForeignMap);
    }
    let applications = capture::applications(invoked, names)?;
    let rows = classification::classify(invoked.plan(), &applications)?;
    Ok(PrismReservationCoverage::new(names, rows))
}

#[cfg(test)]
mod tests;
