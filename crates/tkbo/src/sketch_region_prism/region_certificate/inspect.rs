// SPDX-License-Identifier: GPL-2.0-only
//! One Jordan component follows from complete closure-only coverage on a nondegenerate plane.
use super::super::surface_charts::CapturedPrismSources;
use super::types::{RegionCheck, holds, need, require};
use super::*;
use rustycad_tkg3d::top_abs::Orientation;

pub(super) fn inspect(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
) -> RegionCheck<RegionCertificate> {
    let original = source::source_hypotheses(bridge, sources)?;
    let (candidate, complement) = candidate_universe(bridge, receipt)?;
    let contact = history::current_history(bridge, sources, &candidate)?;
    let copied = rebind::copied_boundary(receipt, &candidate)?;
    Ok(RegionCertificate {
        original,
        census_occurrence: bridge.census.inputs.segments[0].occurrence,
        contact,
        copied,
        boundary_reversed: reversed_boundary(&candidate),
        source_candidate: candidate,
        complementary_candidate: complement,
        selected_winding: 1,
    })
}
fn candidate_universe(
    bridge: &PairedContactTransaction,
    receipt: &PrismReceipt,
) -> RegionCheck<(CapturedPrismSources, CapturedPrismSources)> {
    let batch = borrowed_batch(bridge).ok_or(RegionCertificateCause::UnavailableCandidates)?;
    require_universe(batch)?;
    containment::generated_universe(bridge)?;
    let index = classified_winding(receipt)?;
    let selected = need(receipt.input.selected.as_ref(), "selected candidate")?;
    holds(
        "actual selected candidate identity",
        selected.face.is_equal(&batch.candidates[index].face),
    )?;
    let actual = boundary::candidate_boundary(selected, &bridge.census)?;
    boundary::declared_plane(receipt, &actual)?;
    let complement = boundary::candidate_boundary(&batch.candidates[1 - index], &bridge.census)?;
    opposite_domains(&actual, &complement)?;
    Ok((actual, complement))
}
fn require_universe(batch: &CandidateBatch) -> RegionCheck {
    require(
        "two directed Jordan candidate domains",
        batch.candidates.len(),
        2,
    )?;
    require("no unrepresented source", batch.unrepresented.len(), 0)?;
    require("no deleted source", batch.input_deleted.len(), 0)
}

fn opposite_domains(first: &CapturedPrismSources, second: &CapturedPrismSources) -> RegionCheck {
    let a = &first.boundaries[0].occurrence;
    let b = &second.boundaries[0].occurrence;
    holds(
        "distinct generated face occurrences",
        first.face != second.face,
    )?;
    holds(
        "complement reuses physical boundary",
        history::same_physical(a, b),
    )?;
    let directions = [a.orientation, b.orientation];
    let opposite =
        directions.contains(&Orientation::Forward) && directions.contains(&Orientation::Reversed);
    holds("opposite boundary traversals", opposite)?;
    let facing = first.face.orientation == second.face.orientation;
    holds("same directed face orientation", facing)
}

fn classified_winding(receipt: &PrismReceipt) -> RegionCheck<usize> {
    let classification = need(receipt.classification.as_ref(), "exact winding selection")?;
    let CandidateClassification::Incomplete {
        selection,
        captured,
        ..
    } = classification;
    let SeedSelection::Unique { candidate } = selection else {
        return Err(RegionCertificateCause::UnavailableCandidates);
    };
    require("two winding totals", captured.candidate_windings.len(), 2)?;
    holds("selected candidate index", *candidate < 2)?;
    let windings = &captured.candidate_windings;
    holds(
        "bounded Jordan winding and complement",
        windings[*candidate] == 1 && windings[1 - *candidate] == -1,
    )?;
    Ok(*candidate)
}

fn reversed_boundary(candidate: &CapturedPrismSources) -> bool {
    let direction = candidate.boundaries[0].occurrence.orientation;
    direction == Orientation::Reversed
}
