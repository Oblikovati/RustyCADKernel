//! Reconstruction of the second overlap edge after replacement.
use super::*;

#[allow(clippy::too_many_arguments)] // Mirrors native's explicit wire replacement inputs.
pub(super) fn split_second_overlap(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    face: &Face,
    index: i32,
    points: &SegmentPoints,
    candidates: &CandidatePoints,
    endpoint_keys: (i32, i32),
    boxes: &mut Vec<EdgeBox>,
    counts: &mut RepairCounts<'_>,
) -> Result<(i32, i32), Failure> {
    let mut inserted = 0;
    let mut segment_index = index;
    if endpoint_keys.0 == 0 && endpoint_keys.1 > 0 {
        inserted += split_if_success(
            tool,
            wire,
            face,
            index,
            points.second_parameters.0,
            &candidates.first,
            candidates.first_tolerance,
            boxes,
            counts,
        )?;
        return Ok((inserted, segment_index));
    }
    if endpoint_keys.0 > 0 && endpoint_keys.1 == 0 {
        inserted += split_if_success(
            tool,
            wire,
            face,
            index,
            points.second_parameters.1,
            &candidates.last,
            candidates.last_tolerance,
            boxes,
            counts,
        )?;
        if inserted > 0 {
            segment_index += 1;
        }
        return Ok((inserted, segment_index));
    }
    if endpoint_keys.0 > 0 || endpoint_keys.1 > 0 {
        return Ok((inserted, segment_index));
    }
    let first_inserted = split_if_success(
        tool,
        wire,
        face,
        index,
        points.second_parameters.0,
        &candidates.first,
        candidates.first_tolerance,
        boxes,
        counts,
    )?;
    let current = wire.edge(index)?;
    let (_, first_range, last_range) =
        ShapeAnalysisEdge::new().pcurve_on_face(&current, face, false)?;
    let outside = (first_range - points.second_parameters.1)
        * (last_range - points.second_parameters.1)
        > 0.0;
    // Natively the edge after the first split is num2 + dnum1 + dnum2, so it moves only
    // when that split happened; the segment then is that edge.
    let next_index = if outside {
        index + first_inserted
    } else {
        index
    };
    let second_inserted = split_if_success(
        tool,
        wire,
        face,
        next_index,
        points.second_parameters.1,
        &candidates.last,
        candidates.last_tolerance,
        boxes,
        counts,
    )?;
    inserted += first_inserted + second_inserted;
    if second_inserted > 0 {
        segment_index = if outside { next_index } else { index + 1 };
    }
    Ok((inserted, segment_index))
}
