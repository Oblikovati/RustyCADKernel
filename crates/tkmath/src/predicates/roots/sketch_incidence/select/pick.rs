// SPDX-License-Identifier: GPL-2.0-only
//! Exact source place of a trim pick.
use super::super::budget::Work;
use super::super::place::{KeyValue, SourcePlace, compare_places};
use super::super::rational_polynomial::ratio;
use super::super::source_frame::SourceFrame;
use super::{SketchSelectionError, SketchSelectionRefusal, SketchTrimPick, refused};
use std::cmp::Ordering;

/// Exact place of a pick; a line projects orthogonally, a conic centrally.
pub(super) fn pick_place(
    frame: &SourceFrame,
    pick: SketchTrimPick,
) -> Result<SourcePlace, SketchSelectionError> {
    let outside = refused(SketchSelectionRefusal::PickOutsideSegment { pick });
    match (frame, pick) {
        (SourceFrame::Line(line), SketchTrimPick::Point(point)) => {
            finite(point).ok_or(outside)?;
            let parameter = line.implicit.projected_parameter(point);
            Ok(SourcePlace::Parameter(KeyValue::Ratio(parameter)))
        }
        (SourceFrame::Conic(conic), SketchTrimPick::Point(point)) => {
            finite(point).ok_or(outside.clone())?;
            let direction: [_; 2] =
                std::array::from_fn(|axis| ratio(point[axis]).subtract(&conic.frame.center[axis]));
            if direction
                .iter()
                .all(|value| value.sign() == Ordering::Equal)
            {
                return Err(outside);
            }
            Ok(conic.frame.exact_place(&direction))
        }
        (SourceFrame::Spline(spline), SketchTrimPick::Parameter(value)) => {
            let [lower, upper] = spline.bounds;
            if !(lower..=upper).contains(&value) {
                return Err(outside);
            }
            Ok(SourcePlace::Parameter(KeyValue::Ratio(ratio(value))))
        }
        _ => Err(refused(SketchSelectionRefusal::PickKind { pick })),
    }
}
fn finite(point: [f64; 2]) -> Option<()> {
    point.iter().all(|value| value.is_finite()).then_some(())
}
/// An open source edits only strictly between its ends.
pub(super) fn require_inside(
    frame: &SourceFrame,
    place: &mut SourcePlace,
    pick: SketchTrimPick,
    work: &mut Work<'_>,
) -> Result<(), SketchSelectionError> {
    let [mut start, mut end] = frame.ends();
    let after_start = compare_places(place, &mut start, work)? == Ordering::Greater;
    let before_end = compare_places(place, &mut end, work)? == Ordering::Less;
    if after_start && before_end {
        return Ok(());
    }
    Err(refused(SketchSelectionRefusal::PickOutsideSegment { pick }))
}
