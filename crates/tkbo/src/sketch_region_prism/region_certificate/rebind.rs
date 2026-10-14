// SPDX-License-Identifier: GPL-2.0-only
//! Rebind to the actual isolated copy and its composed history before removing any gap.
use super::super::{PrismCopiedSelection, PrismReceipt, surface_charts::CapturedPrismSources};
use super::{boundary::*, types::*};
use crate::sketch_region::{ArrangementRelation, equal_occurrences};

pub(super) fn copied_boundary(
    receipt: &PrismReceipt,
    candidate: &CapturedPrismSources,
) -> RegionCheck<CapturedPrismSources> {
    let copied = need(receipt.copied.as_ref(), "isolated copied selection")?;
    let actual = capture_boundary(&copied.base)?;
    copied_snapshots(copied, &actual)?;
    holds(
        "source/copy exact geometry",
        same_boundary_geometry(candidate, &actual),
    )?;
    holds(
        "source/copy physical separation",
        separated(candidate, &actual),
    )?;
    copied_history(copied, receipt)?;
    embedded_support(receipt, &actual)?;
    Ok(actual)
}
fn copied_snapshots(copied: &PrismCopiedSelection, actual: &CapturedPrismSources) -> RegionCheck {
    let frozen = copied
        .chart_sources
        .as_ref()
        .map_err(|cause| RegionCertificateCause::Capture(cause.clone()))?;
    holds(
        "pre-sweep/current copied geometry",
        same_boundary_geometry(frozen, actual),
    )?;
    let same = frozen.face == actual.face
        && frozen.boundaries[0].occurrence == actual.boundaries[0].occurrence;
    holds("pre-sweep/current copied identities", same)
}
fn separated(first: &CapturedPrismSources, second: &CapturedPrismSources) -> bool {
    first.face.topology_address != second.face.topology_address
        && first.boundaries[0].occurrence.topology_address
            != second.boundaries[0].occurrence.topology_address
}

fn embedded_support(receipt: &PrismReceipt, actual: &CapturedPrismSources) -> RegionCheck {
    let chart = need(receipt.affine_chart.as_ref(), "copied affine support")?;
    let vector = receipt.limits.requested.extrusion;
    holds(
        "requested/current affine extrusion",
        chart.extrusion == [vector.x(), vector.y(), vector.z()],
    )?;
    let proof = chart
        .decision
        .as_ref()
        .map_err(|_| RegionCertificateCause::UnavailableCandidates)?;
    let same = chart.source == actual.face && same_support(&proof.support, &actual.support);
    holds("copied affine chart identity", same)?;
    holds(
        "nondegenerate support and requested sweep",
        nonzero(&proof.signed_jacobian),
    )
}
fn nonzero(value: &rustycad_tkmath::predicates::roots::DyadicBound) -> bool {
    // Exact zero refuses only this support/sweep hypothesis, never all planar region operations.
    value.compare_f64(0.).is_ok_and(|order| !order.is_eq())
}

fn copied_history(copied: &PrismCopiedSelection, receipt: &PrismReceipt) -> RegionCheck {
    let selected = need(receipt.input.selected.as_ref(), "selected candidate")?;
    let members = equal_occurrences(&copied.base).map_err(RegionCertificateCause::Mapped)?;
    require(
        "copied occurrence history coverage",
        copied.relations.len(),
        members.len(),
    )?;
    require(
        "copy/source relation bijection",
        copied.relations.len(),
        selected.relations.len(),
    )?;
    bind_copy_members(&members, &copied.relations, &selected.relations)
}
fn bind_copy_members(
    members: &[rustycad_tkbrep::topods::Shape],
    copied: &[ArrangementRelation],
    sources: &[ArrangementRelation],
) -> RegionCheck {
    for member in members {
        let rows: Vec<_> = copied
            .iter()
            .filter(|row| row.output.is_equal(member))
            .collect();
        require("one composed copy relation", rows.len(), 1)?;
        bind_parent_relation(rows[0], sources)?;
    }
    Ok(())
}

fn bind_parent_relation(
    actual: &ArrangementRelation,
    sources: &[ArrangementRelation],
) -> RegionCheck {
    let kind = actual
        .output
        .shape_type()
        .map_err(RegionCertificateCause::Mapped)?;
    let orientation = actual.output.orientation();
    let matches = sources
        .iter()
        .filter(|old| {
            old.output.shape_type().is_ok_and(|value| value == kind)
                && old.output.orientation() == orientation
                && same_composed_parents(old, actual)
        })
        .count();
    require("unique composed parent relation", matches, 1)
}
fn same_composed_parents(source: &ArrangementRelation, copied: &ArrangementRelation) -> bool {
    let modified: Vec<_> = source.preserved.iter().chain(&source.modified).collect();
    copied.preserved.is_empty()
        && modified.len() == copied.modified.len()
        && modified
            .iter()
            .all(|parent| copied.modified.contains(parent))
        && source.generated == copied.generated
}
