// SPDX-License-Identifier: GPL-2.0-only
//! Count nested borrowed rosters before legacy inspectors allocate correspondence/fault vectors.
use super::work::*;
use crate::sketch_region::ArrangementRelation;
use crate::sketch_region_candidates::CandidateSegment;
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_source_conformance::{CapturedSpaceRepresentation, OriginalCurveCapture};

pub(super) fn curve<'a, const D: usize>(
    curve: &CapturedMetricCurve<D>,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    work.units("borrowed metric curve", 1)?;
    if let CapturedMetricCurve::Bezier { poles, weights } = curve {
        work.units("borrowed metric pole", poles.len())?;
        work.units(
            "borrowed metric weight",
            weights.as_ref().map_or(0, Vec::len),
        )?;
    }
    Ok(())
}
pub(super) fn space<'a>(
    space: &CapturedSpaceRepresentation,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    work.units("borrowed space descriptor", 1)?;
    curve(&space.curve, work)
}
pub(super) fn originals<'a>(
    rows: &[OriginalCurveCapture],
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    work.units("borrowed original capture", rows.len())?;
    for row in rows {
        if let Ok(capture) = &row.representation {
            space(capture, work)?;
        }
    }
    Ok(())
}
pub(super) fn segments<'a>(
    rows: &[CandidateSegment],
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    work.units("borrowed segment", rows.len())?;
    for row in rows {
        work.units("borrowed segment parent", row.parents.len())?;
        work.units("borrowed segment control", row.controls.len())?;
        work.units(
            "borrowed segment weight",
            row.weights.as_ref().map_or(0, Vec::len),
        )?;
    }
    Ok(())
}
pub(super) fn relations<'a>(
    rows: &[ArrangementRelation],
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    work.units("borrowed relation", rows.len())?;
    for row in rows {
        for parents in [&row.preserved, &row.modified, &row.generated] {
            work.units("borrowed relation kinded parent", parents.len())?;
        }
    }
    Ok(())
}
