// SPDX-License-Identifier: GPL-2.0-only
//! This adapter accepts only the complete immutable operation and its exact certified name map.
use super::{coverage, error::*, faces, incidence, members, occurrences, result::*, rows::*};
use crate::sketch_region_prism::{
    owned_request::names::body_map::ResolvedPrismNames, publication::PublishedNamedPrism,
};

/// Project accepted operands without live lookup; `project_named_prism_body(result, names)` refuses a foreign map.
pub(crate) fn project_named_prism_body(
    result: &PublishedNamedPrism,
    names: &ResolvedPrismNames,
) -> BodyCheck<OwnedNamedPrismBody> {
    admit(result, names)?;
    let rows = project_rows(result, names)?;
    Ok(OwnedNamedPrismBody {
        plan: names.plan().clone(),
        root: super::lookup::root_occurrence(names, result.join().projection().uses())?,
        naming_binding: names.replay_bytes().to_vec(),
        rows,
    })
}
fn admit(result: &PublishedNamedPrism, names: &ResolvedPrismNames) -> BodyCheck<()> {
    if names.version() != 1 {
        return Err(failure(
            names,
            PrismBodyProjectionCause::UnsupportedVersion {
                actual: names.version(),
                expected: 1,
            },
        ));
    }
    if !names.binds(result) {
        return Err(failure(names, PrismBodyProjectionCause::PublicationBinding));
    }
    let projection = result.join().projection();
    coverage::domains(names, projection.nodes(), projection.uses())?;
    super::references::check(result, names)
}
fn project_rows(
    result: &PublishedNamedPrism,
    names: &ResolvedPrismNames,
) -> BodyCheck<PrismBodyRows> {
    let projection = result.join().projection();
    Ok(PrismBodyRows {
        members: project_members(result, names)?,
        occurrences: project_occurrences(result, names)?,
        faces: projection
            .faces()
            .iter()
            .map(|row| faces::project(row, names))
            .collect::<BodyCheck<_>>()?,
        endpoints: incidence::endpoints(projection, names)?,
        coedges: incidence::coedges(projection, names)?,
        wires: incidence::wires(projection, names)?,
    })
}
fn project_members(
    result: &PublishedNamedPrism,
    names: &ResolvedPrismNames,
) -> BodyCheck<Vec<PrismBodyMember>> {
    let metadata = result.validation().tolerance().metadata();
    result
        .join()
        .projection()
        .nodes()
        .iter()
        .map(|row| members::project(row, names, metadata))
        .collect()
}
fn project_occurrences(
    result: &PublishedNamedPrism,
    names: &ResolvedPrismNames,
) -> BodyCheck<Vec<PrismBodyOccurrence>> {
    let raw = result.join().raw_vertices();
    result
        .join()
        .projection()
        .uses()
        .iter()
        .map(|row| occurrences::project(row, names, raw))
        .collect()
}
