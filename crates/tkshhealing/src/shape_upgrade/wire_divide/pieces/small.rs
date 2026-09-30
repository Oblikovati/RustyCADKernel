//! Small segments retain native saved-parameter and vertex-location mutation order.
use super::geometry::PieceSamples;
use super::*;
pub(super) fn next_vertex<T: WireDivider + ?Sized>(
    tool: &T,
    support: &mut WireSupport,
    curves: &DividedCurves,
    small: &Handle<ShapeUpgradeFixSmallCurves>,
    cursor: &mut PieceCursor,
    piece: &mut PieceGeometry,
    index: usize,
) -> Result<Option<Vertex>, Failure> {
    // The native single/degenerate path leaves V null; it does not substitute V2.
    if curves.count() == 1 || cursor.vertices.degenerate {
        return Ok(Some(Vertex::default()));
    }
    let samples = geometry::samples(support, piece, curves, index)?;
    let tolerance = tool.wire_divide_base().min_tolerance();
    if samples.end.distance(&cursor.vertices.first_point) < tolerance
        && samples.end.distance(&samples.middle) < tolerance
        && !support.face.is_null()
    {
        skip_piece(support, curves, cursor, &samples, index)?;
        return Ok(None);
    }
    if cursor.small_count != 0 {
        repair_small(small, cursor, piece, &samples, tolerance)?;
    }
    segment_vertex(curves, cursor, &samples, index)
}
fn segment_vertex(
    curves: &DividedCurves,
    cursor: &mut PieceCursor,
    samples: &PieceSamples,
    index: usize,
) -> Result<Option<Vertex>, Failure> {
    if index == curves.count() {
        return Ok(Some(cursor.vertices.last.clone()));
    }
    let mut vertex = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut vertex, samples.end, cursor.vertices.tolerance)?;
    cursor.vertices.first_point = samples.end;
    Ok(Some(vertex))
}
fn skip_piece(
    support: &WireSupport,
    curves: &DividedCurves,
    cursor: &mut PieceCursor,
    samples: &PieceSamples,
    index: usize,
) -> Result<(), Failure> {
    if cursor.small_count == 0 {
        cursor.saved_first = samples.first;
        cursor.saved_index = index;
    }
    cursor.small_count += 1;
    if index == curves.count() {
        let replacement = cursor
            .vertices
            .first
            .oriented(cursor.vertices.last.orientation());
        support
            .context
            .replace(&cursor.vertices.last, &replacement)?;
    }
    Ok(())
}
fn repair_small(
    small: &Handle<ShapeUpgradeFixSmallCurves>,
    cursor: &mut PieceCursor,
    piece: &mut PieceGeometry,
    samples: &PieceSamples,
    tolerance: f64,
) -> Result<(), Failure> {
    let large = samples.end.distance(&samples.start) > tolerance
        || samples.end.distance(&samples.middle) > tolerance;
    if large || !approximate(small, cursor, piece, samples.last)? {
        move_first_vertex(cursor, samples.start)?;
    }
    cursor.small_count = 0;
    Ok(())
}
fn approximate(
    small: &Handle<ShapeUpgradeFixSmallCurves>,
    cursor: &mut PieceCursor,
    piece: &mut PieceGeometry,
    mut last: f64,
) -> Result<bool, Failure> {
    let mut spatial = Handle::default();
    let mut planar = Handle::default();
    let mut reversed = Handle::default();
    let approximated = small.cast::<dyn FixSmallCurves>().approx(
        &mut spatial,
        &mut planar,
        &mut reversed,
        &mut cursor.saved_first,
        &mut last,
    )?;
    if approximated {
        piece.spatial = spatial;
        piece.planar = planar;
        piece.reversed = reversed;
    }
    Ok(approximated)
}
fn move_first_vertex(cursor: &PieceCursor, start: Point3) -> Result<(), Failure> {
    let first = cursor.vertices.first_point;
    let middle = Point3::new(
        0.5 * (first.x() + start.x()),
        0.5 * (first.y() + start.y()),
        0.5 * (first.z() + start.z()),
    );
    BRepBuilder::default().update_vertex_point(&cursor.vertices.first, middle, 0.)
}
