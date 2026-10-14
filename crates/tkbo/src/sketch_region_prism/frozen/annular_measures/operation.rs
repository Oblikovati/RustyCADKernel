// SPDX-License-Identifier: GPL-2.0-only
use super::*;

/// Measure the complete sealed annular domain; `inspect_annular_measures(join, options, cancel)`.
/// The owned scalar reports confer no ordered body validity or physical-volume interpretation.
pub(in crate::sketch_region_prism) fn inspect_annular_measures(
    join: &AnnularFrozenDomain,
    requested: AnnularMeasureOptions,
    mut cancelled: impl FnMut() -> bool + Send,
) -> Result<AnnularPrismMeasures, Box<AnnularMeasureFailure>> {
    let boundaries =
        admit(join, requested).map_err(|cause| early_failure(join, requested, cause))?;
    let mut work = MeasureWork {
        join,
        requested,
        boundaries,
        phase: AnnularMeasurePhase::Admission,
        entity: None,
        partial: Default::default(),
    };
    if let Err(cause) = construct(&mut work, &mut cancelled) {
        return Err(work.failure(cause));
    }
    Ok(work.complete())
}
fn admit(
    join: &AnnularFrozenDomain,
    requested: AnnularMeasureOptions,
) -> MeasureCheck<[AnnularBoundaryDirection; 2]> {
    if requested.version != 2 {
        return Err(AnnularMeasureCause::Version {
            actual: requested.version,
            expected: 2,
        });
    }
    validate_polynomial_side_options(requested.side_area)
        .map_err(AnnularMeasureCause::OriginalSidePolicy)?;
    count("all four bound annular faces", join.faces().len(), 4)?;
    let keys: std::collections::BTreeSet<_> =
        join.faces().iter().map(|f| &f.chart.occurrence).collect();
    count("four distinct full named face paths", keys.len(), 4)?;
    annular_boundary_directions(join).map_err(AnnularMeasureCause::Direction)
}
fn construct(work: &mut MeasureWork<'_>, cancelled: &mut impl FnMut() -> bool) -> MeasureCheck<()> {
    poll(cancelled)?;
    caps::measure_caps(work, cancelled)?;
    caps::measure_sweep(work, cancelled)?;
    sides::measure_sides(work, cancelled)?;
    total(work, cancelled)?;
    work.phase = AnnularMeasurePhase::Commit;
    poll(cancelled)
}
fn total(work: &mut MeasureWork<'_>, cancelled: &mut impl FnMut() -> bool) -> MeasureCheck<()> {
    work.phase = AnnularMeasurePhase::Total;
    work.entity = None;
    count(
        "four completed named face measures",
        work.partial.faces.len(),
        4,
    )?;
    let components: Vec<_> = work
        .partial
        .faces
        .iter()
        .map(|row| row.measure.enclosure())
        .collect();
    let report = sum_nonnegative_enclosures(&components, work.requested.total_area, cancelled)
        .map_err(AnnularMeasureCause::Total)?;
    work.partial.total = Some(report);
    Ok(())
}
fn poll(cancelled: &mut impl FnMut() -> bool) -> MeasureCheck<()> {
    if cancelled() {
        return Err(AnnularMeasureCause::Cancelled);
    }
    Ok(())
}
fn early_failure(
    join: &AnnularFrozenDomain,
    requested: AnnularMeasureOptions,
    cause: AnnularMeasureCause,
) -> Box<AnnularMeasureFailure> {
    Box::new(AnnularMeasureFailure {
        operation: join.projection().operation,
        phase: AnnularMeasurePhase::Admission,
        entity: None,
        requested,
        binding: join.replay_bytes().into(),
        partial: Default::default(),
        cause,
    })
}
