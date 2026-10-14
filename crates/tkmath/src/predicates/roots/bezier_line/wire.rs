// SPDX-License-Identifier: GPL-2.0-only
mod captured;
mod endpoint;
pub use captured::{
    CaptureWorkReceipt, CapturedBezierWindingProof, CapturedBezierWireFailure,
    CapturedBezierWireInput, CapturedBezierWireOccurrence, CapturedBezierWireResult,
    CapturedBezierWireStore, CapturedWindingEvidence,
};
mod error;
mod types;
mod walk;
use super::{BezierLineQuery, winding::WindingQuery};
pub use error::BezierWireError;
use std::collections::BTreeSet;
pub use types::{BezierWireLocation, BezierWireQuery, BezierWireResult, BezierWireSegment};
/// Certify exact closure and classify the ray origin by the complete nonzero winding rule.
///
/// Self-crossing paths are allowed; this does not certify a valid face.
/// ```
/// use rustycad_tkmath::predicates::roots::{RootIsolationOptions, bezier_line::*};
/// let controls = [[0.,0.],[1.,1.],[0.,0.]];
/// let segments = [BezierWireSegment { id: 7, controls: &controls, weights: None,
///     interval: [0.,1.], reversed: false }];
/// let query = BezierWireQuery { segments: &segments, line: [[2.,0.],[3.,0.]],
///     options: RootIsolationOptions::new(0.001) };
/// assert_eq!(classify_bezier_wire(query, || false)?.location(), &BezierWireLocation::Outside);
/// # Ok::<(), BezierWireError>(())
/// ```
pub fn classify_bezier_wire(
    query: BezierWireQuery<'_>,
    mut cancelled: impl FnMut() -> bool,
) -> Result<BezierWireResult, BezierWireError> {
    validate_ids(query.segments, &mut cancelled)?;
    let mut walk = walk::WireWalk::default();
    for segment in query.segments {
        poll(segment.id, &mut cancelled)?;
        append_segment(&mut walk, query, *segment, &mut cancelled)?;
    }
    walk.finish()
}
fn validate_ids(
    segments: &[BezierWireSegment<'_>],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<(), BezierWireError> {
    if segments.is_empty() {
        return Err(BezierWireError::Empty);
    }
    let mut ids = BTreeSet::new();
    for segment in segments {
        poll(segment.id, cancelled)?;
        if !ids.insert(segment.id) {
            return Err(BezierWireError::DuplicateId { id: segment.id });
        }
    }
    Ok(())
}
fn append_segment(
    walk: &mut walk::WireWalk,
    query: BezierWireQuery<'_>,
    segment: BezierWireSegment<'_>,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<(), BezierWireError> {
    let request = segment_request(query, segment, walk.used);
    let (prepared, mut endpoints) =
        WindingQuery::prepare_captured(request, cancelled, endpoint::capture).map_err(|cause| {
            BezierWireError::Segment {
                id: segment.id,
                cause,
            }
        })?;
    if segment.reversed {
        endpoints.reverse();
    }
    walk.join(segment.id, endpoints)?;
    let result = prepared
        .evaluate(cancelled)
        .map_err(|cause| BezierWireError::Segment {
            id: segment.id,
            cause,
        })?;
    walk.append(segment.id, segment.reversed, result)
}
fn poll(id: u64, cancelled: &mut impl FnMut() -> bool) -> Result<(), BezierWireError> {
    if cancelled() {
        return Err(BezierWireError::Cancelled { id });
    }
    Ok(())
}

fn segment_request<'a>(
    query: BezierWireQuery<'_>,
    segment: BezierWireSegment<'a>,
    used: u16,
) -> BezierLineQuery<'a> {
    let mut options = query.options;
    options.max_bisections -= used;
    BezierLineQuery {
        controls: segment.controls,
        weights: segment.weights,
        line: query.line,
        interval: segment.interval,
        options,
    }
}
