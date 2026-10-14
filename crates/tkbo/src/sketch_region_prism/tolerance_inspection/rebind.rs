// SPDX-License-Identifier: GPL-2.0-only
//! Final comparison has no callbacks; raw metadata and actual endpoint points need their own replay.
use super::super::PrismReceipt;
use super::{
    census::capture_census, census_types::*, inspection_types::*, metadata::capture_metadata,
};
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_source_conformance::same_space_representation;

pub(super) fn rebind_local_capture(
    receipt: &PrismReceipt,
    captured: &PrismToleranceInspection,
) -> InspectionCheck<()> {
    let current = capture_census(receipt, &captured.binding)
        .map_err(|cause| census_failure(ToleranceInspectionPhase::Commit, cause))?;
    compare_census(&captured.census, &current)?;
    let metadata = capture_metadata(receipt, &current, captured.requested).map_err(at_commit)?;
    compare_metadata(&captured.metadata, &metadata)
}
fn compare_census(
    before: &PrismToleranceCensus,
    current: &PrismToleranceCensus,
) -> InspectionCheck<()> {
    require_current(
        before.contexts == current.contexts
            && before.entities == current.entities
            && before.trims == current.trims
            && before.history == current.history,
        None,
        "complete directed census/history",
    )?;
    compare_endpoints(&before.endpoints, &current.endpoints)
}
fn compare_endpoints(
    before: &[ToleranceEndpointCensus],
    current: &[ToleranceEndpointCensus],
) -> InspectionCheck<()> {
    let same_count = before.len() == current.len();
    require_current(same_count, None, "physical endpoint count")?;
    for expected in before {
        let actual = current
            .iter()
            .find(|row| row.edge == expected.edge && row.vertex == expected.vertex);
        require_current(
            actual.is_some_and(|row| same_endpoint(expected, row)),
            Some(&expected.edge),
            "endpoint point/parameter/curve/context operands",
        )?;
    }
    Ok(())
}
fn same_endpoint(before: &ToleranceEndpointCensus, current: &ToleranceEndpointCensus) -> bool {
    before.parameter.to_bits() == current.parameter.to_bits()
        && before.point.map(f64::to_bits) == current.point.map(f64::to_bits)
        && before.contexts == current.contexts
        && same_space_representation(&before.space, &current.space)
}
fn compare_metadata(
    before: &[ToleranceMetadataEvidence],
    current: &[ToleranceMetadataEvidence],
) -> InspectionCheck<()> {
    require_current(before.len() == current.len(), None, "metadata entity count")?;
    for expected in before {
        let actual = current.iter().find(|row| row.entity == expected.entity);
        let same = actual.is_some_and(|row| {
            row.observation.raw.to_bits() == expected.observation.raw.to_bits()
                && row.observation.effective.to_bits() == expected.observation.effective.to_bits()
        });
        require_current(same, Some(&expected.entity), "raw/effective metadata bits")?;
    }
    Ok(())
}
fn require_current(
    holds: bool,
    entity: Option<&PartitionOccurrenceIdentity>,
    quantity: &'static str,
) -> InspectionCheck<()> {
    if holds {
        return Ok(());
    }
    Err(failure(
        ToleranceInspectionPhase::Commit,
        ToleranceInspectionCause::Stale {
            entity: entity.cloned(),
            quantity,
        },
    ))
}

fn at_commit(mut error: ToleranceInspectionFailure) -> ToleranceInspectionFailure {
    error.phase = ToleranceInspectionPhase::Commit;
    error
}
