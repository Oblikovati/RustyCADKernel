// SPDX-License-Identifier: GPL-2.0-only
//! Extra or omitted operands cannot disappear while projecting accepted rows.
use super::{coverage::exact_keys, error::*};
use crate::sketch_region_prism::{
    exact_tolerance::JoinedToleranceMetadata, frozen::*,
    owned_request::names::body_map::ResolvedPrismNames, publication::PublishedNamedPrism,
};

pub(super) fn check(result: &PublishedNamedPrism, names: &ResolvedPrismNames) -> BodyCheck<()> {
    let projection = result.join().projection();
    metadata(
        projection,
        result.validation().tolerance().metadata(),
        names,
    )?;
    vertices(result, names)?;
    faces(projection, names)?;
    incidence(projection, names)
}
fn member_keys(projection: &FrozenPrismProjection, kind: u8) -> Vec<FrozenPrismKey> {
    projection
        .nodes()
        .iter()
        .filter(|row| row.key.kind == kind)
        .map(|row| row.key)
        .collect()
}
fn use_keys(projection: &FrozenPrismProjection, kind: u8) -> Vec<&FrozenUseKey> {
    projection
        .uses()
        .iter()
        .filter(|row| row.key.0.last().is_some_and(|(key, _)| key.kind == kind))
        .map(|row| &row.key)
        .collect()
}
fn metadata(
    projection: &FrozenPrismProjection,
    rows: &[JoinedToleranceMetadata],
    names: &ResolvedPrismNames,
) -> BodyCheck<()> {
    let expected = tolerance_member_keys(projection);
    let actual: Vec<_> = rows.iter().map(|row| row.entity).collect();
    exact_keys(
        names,
        "complete tolerance member coverage",
        &actual,
        &expected,
    )?;
    for row in rows {
        metadata_contexts(projection, row, names)?;
    }
    Ok(())
}
fn metadata_contexts(
    projection: &FrozenPrismProjection,
    row: &JoinedToleranceMetadata,
    names: &ResolvedPrismNames,
) -> BodyCheck<()> {
    let contexts: Vec<_> = projection
        .uses()
        .iter()
        .filter(|item| item.key.0.last().is_some_and(|(key, _)| *key == row.entity))
        .map(|item| &item.key)
        .collect();
    let actual: Vec<_> = row.contexts.iter().collect();
    exact_keys(
        names,
        "complete tolerance occurrence coverage",
        &actual,
        &contexts,
    )
}
fn vertices(result: &PublishedNamedPrism, names: &ResolvedPrismNames) -> BodyCheck<()> {
    let expected = use_keys(result.join().projection(), 7);
    let actual: Vec<_> = result
        .join()
        .raw_vertices()
        .iter()
        .map(|row| &row.occurrence)
        .collect();
    exact_keys(
        names,
        "complete raw local vertex occurrence coverage",
        &actual,
        &expected,
    )
}
fn faces(projection: &FrozenPrismProjection, names: &ResolvedPrismNames) -> BodyCheck<()> {
    let expected = member_keys(projection, 4);
    let actual: Vec<_> = projection.faces().iter().map(|row| row.key).collect();
    exact_keys(names, "complete face surface coverage", &actual, &expected)?;
    let trims = trim_keys(projection);
    let coedges: Vec<_> = projection
        .incidence()
        .coedges
        .iter()
        .map(|row| &row.occurrence)
        .collect();
    exact_keys(
        names,
        "complete directed face trim coverage",
        &trims,
        &coedges,
    )
}
fn incidence(projection: &FrozenPrismProjection, names: &ResolvedPrismNames) -> BodyCheck<()> {
    endpoints(projection, names)?;
    let coedges: Vec<_> = projection
        .incidence()
        .coedges
        .iter()
        .map(|row| &row.occurrence)
        .collect();
    exact_keys(
        names,
        "complete directed coedge coverage",
        &coedges,
        &use_keys(projection, 6),
    )?;
    wires(projection, names, &coedges)
}
fn wires(
    projection: &FrozenPrismProjection,
    names: &ResolvedPrismNames,
    coedges: &[&FrozenUseKey],
) -> BodyCheck<()> {
    let rows = projection
        .wire_traversals()
        .ok_or_else(|| missing_traversal(names))?;
    let wires: Vec<_> = rows.iter().map(|row| &row.occurrence).collect();
    exact_keys(
        names,
        "complete ordered wire coverage",
        &wires,
        &use_keys(projection, 5),
    )?;
    let actual: Vec<_> = rows.iter().flat_map(|row| &row.coedges).collect();
    exact_keys(names, "complete ordered coedge coverage", &actual, coedges)
}

fn endpoints(projection: &FrozenPrismProjection, names: &ResolvedPrismNames) -> BodyCheck<()> {
    let endpoints: Vec<_> = projection
        .incidence()
        .endpoints
        .iter()
        .map(|row| row.0)
        .collect();
    exact_keys(
        names,
        "complete edge endpoint coverage",
        &endpoints,
        &member_keys(projection, 6),
    )
}
fn tolerance_member_keys(projection: &FrozenPrismProjection) -> Vec<FrozenPrismKey> {
    [4, 6, 7]
        .into_iter()
        .flat_map(|kind| member_keys(projection, kind))
        .collect()
}
fn trim_keys(projection: &FrozenPrismProjection) -> Vec<&FrozenUseKey> {
    projection
        .faces()
        .iter()
        .flat_map(|row| row.trims.iter().map(|trim| &trim.occurrence))
        .collect()
}
