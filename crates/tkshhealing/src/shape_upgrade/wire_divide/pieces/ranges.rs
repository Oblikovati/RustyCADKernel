//! Transfer old representation ranges before replacing the active pcurve(s).
use super::*;
struct SegmentRanges {
    spatial: (f64, f64),
    planar: (f64, f64),
    same: bool,
}
pub(super) fn set_ranges(
    support: &WireSupport,
    edge: &Edge,
    curves: &DividedCurves,
    piece: &PieceGeometry,
    first: usize,
    index: usize,
) -> Result<(f64, f64), Failure> {
    let mut ranges = segment_ranges(curves, piece, first, index + 1)?;
    transfer_ranges(curves, edge, piece, first, index + 1)?;
    if !support.face.is_null() {
        let (pcurve, start, end) =
            ShapeAnalysisEdge::new().pcurve_on_face(edge, &support.face, false)?;
        if !pcurve.is_null() {
            ranges.same &= (start, end) == ranges.planar;
        }
    }
    construction::update_pcurves(support, edge, piece, curves.seam)?;
    install_ranges(support, edge, piece, &ranges)?;
    if (!curves.was_same_range || !ranges.same) && !BRepTool::degenerated(edge)? {
        BRepBuilder::default().same_range(edge, false)?;
    }
    Ok(if curves.has3 {
        ranges.spatial
    } else {
        ranges.planar
    })
}
fn install_ranges(
    support: &WireSupport,
    edge: &Edge,
    piece: &PieceGeometry,
    ranges: &SegmentRanges,
) -> Result<(), Failure> {
    if !piece.spatial.is_null() {
        ShapeBuildEdge.set_range_3d(edge, ranges.spatial.0, ranges.spatial.1)?;
    }
    if !piece.planar.is_null() {
        BRepBuilder::default().range_on_face(
            edge,
            &support.face,
            ranges.planar.0,
            ranges.planar.1,
        )?;
    }
    Ok(())
}
fn segment_ranges(
    curves: &DividedCurves,
    piece: &PieceGeometry,
    first: usize,
    last: usize,
) -> Result<SegmentRanges, Failure> {
    let mut ranges = SegmentRanges {
        spatial: (0., 0.),
        planar: (0., 0.),
        same: true,
    };
    spatial_range(&mut ranges, curves, piece, first, last)?;
    planar_range(&mut ranges, curves, piece, first, last)?;
    Ok(ranges)
}
fn spatial_range(
    ranges: &mut SegmentRanges,
    curves: &DividedCurves,
    piece: &PieceGeometry,
    first: usize,
    last: usize,
) -> Result<(), Failure> {
    if !piece.spatial.is_null() {
        let (bounded, range) = piece.spatial_bounds(curves, first, last)?;
        ranges.spatial = range;
        if bounded {
            ranges.same = matches_knots(ranges.spatial, &curves.knots3, first, last)?;
        }
    }
    Ok(())
}
fn planar_range(
    ranges: &mut SegmentRanges,
    curves: &DividedCurves,
    piece: &PieceGeometry,
    first: usize,
    last: usize,
) -> Result<(), Failure> {
    if !piece.planar.is_null() {
        let (bounded, range) = piece.planar_bounds(curves, first, last)?;
        ranges.planar = range;
        if bounded {
            ranges.same &= matches_knots(ranges.planar, &curves.knots2, first, last)?;
        }
    }
    Ok(())
}
fn matches_knots(
    bounds: (f64, f64),
    knots: &split_values::Knots,
    first: usize,
    last: usize,
) -> Result<bool, Failure> {
    Ok(bounds.0 == split_values::value(knots, first)?
        && bounds.1 == split_values::value(knots, last)?)
}
fn transfer_ranges(
    curves: &DividedCurves,
    edge: &Edge,
    piece: &PieceGeometry,
    first: usize,
    last: usize,
) -> Result<(), Failure> {
    let planar = piece.spatial.is_null();
    let knots = if planar {
        &curves.knots2
    } else {
        &curves.knots3
    };
    curves.transfer.transfer_range(
        edge,
        split_values::value(knots, first)?,
        split_values::value(knots, last)?,
        planar,
    )
}
