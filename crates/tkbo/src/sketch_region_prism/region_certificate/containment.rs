// SPDX-License-Identifier: GPL-2.0-only
//! Bind actual face/wire/edge containment to the previously captured generated contexts.
use super::{PairedContactTransaction, types::*};
use crate::sketch_region::{PartitionOccurrenceIdentity, equal_occurrences};
use crate::sketch_region_candidates::{
    GeneratedAreaSnapshot, GeneratedMemberSnapshot, GeneratedTopologyObservation,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn generated_universe(bridge: &PairedContactTransaction) -> RegionCheck {
    let copy = need(bridge.mapped.copied.as_ref(), "candidate copy receipt")?;
    let failures = copy.observation_failures.len();
    require("copy observation faults", failures, 0)?;
    let GeneratedTopologyObservation::Observed(areas) = &copy.generated_topology else {
        return Err(RegionCertificateCause::UnavailableCandidates);
    };
    let batch =
        super::borrowed_batch(bridge).ok_or(RegionCertificateCause::UnavailableCandidates)?;
    require(
        "complete generated face count",
        areas.len(),
        batch.candidates.len(),
    )?;
    for candidate in &batch.candidates {
        bind_area(&candidate.face, areas)?;
    }
    Ok(())
}
fn bind_area(shape: &Shape, areas: &[GeneratedAreaSnapshot]) -> RegionCheck {
    let key = PartitionOccurrenceIdentity::capture(shape);
    let rows: Vec<_> = areas
        .iter()
        .filter(|area| {
            area.members
                .iter()
                .any(|row| row.kind == ShapeEnum::Face && row.occurrence == key)
        })
        .collect();
    require("one captured generated face identity", rows.len(), 1)?;
    let members = equal_occurrences(shape).map_err(RegionCertificateCause::Mapped)?;
    let expected = rows[0].members.len();
    let actual = members.len();
    require("generated member count", actual, expected)?;
    for member in &members {
        bind_member(member, &rows[0].members)?;
    }
    Ok(())
}
fn bind_member(shape: &Shape, captured: &[GeneratedMemberSnapshot]) -> RegionCheck {
    let key = PartitionOccurrenceIdentity::capture(shape);
    let rows: Vec<_> = captured
        .iter()
        .filter(|row| row.occurrence == key)
        .collect();
    require("one captured generated member", rows.len(), 1)?;
    let descendants = equal_occurrences(shape).map_err(RegionCertificateCause::Mapped)?;
    let keys: Vec<_> = descendants
        .iter()
        .skip(1)
        .map(PartitionOccurrenceIdentity::capture)
        .collect();
    holds(
        "generated/current containment identity",
        super::identity_multiset::same_multiset(&rows[0].descendants, &keys),
    )
}
