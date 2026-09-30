use super::selector::SegmentSelector;
use super::*;
struct WirePair {
    index: i32,
    same: bool,
    current: Handle<FaceSegments>,
    target: Handle<FaceSegments>,
    tree: Handle<FaceBoundsTree>,
}
pub(super) fn process(checker: &BRepMeshFaceChecker, index: i32) -> Result<(), Failure> {
    let collections = checker.wires.snapshot();
    let current = collections
        .segments
        .read()
        .value(index)
        .map_err(array_failure)?
        .clone();
    let mut selector = SegmentSelector::new(2.0 * checker.face.get_deflection());
    for target_index in index..checker.face.wires_nb() {
        let pair = wire_pair(&collections, index, current.clone(), target_index)?;
        selector.set_segments(&pair.target);
        check_segments(&collections, &pair, &mut selector)?;
    }
    Ok(())
}
fn wire_pair(
    collections: &WireCollections,
    index: i32,
    current: Handle<FaceSegments>,
    target_index: i32,
) -> Result<WirePair, Failure> {
    let target = collections
        .segments
        .read()
        .value(target_index)
        .map_err(array_failure)?
        .clone();
    let tree = collections
        .trees
        .read()
        .value(target_index)
        .map_err(array_failure)?
        .clone();
    Ok(WirePair {
        index,
        same: index == target_index,
        current,
        target,
        tree,
    })
}
fn check_segments(
    collections: &WireCollections,
    pair: &WirePair,
    selector: &mut SegmentSelector,
) -> Result<(), Failure> {
    for segment_index in 0..pair.current.read().length() {
        let segment = pair
            .current
            .read()
            .value(segment_index)
            .map_err(array_failure)?
            .clone();
        selector.reset(&segment, if pair.same { segment_index } else { -1 })?;
        let count = pair.tree.read().select(selector);
        if let Some(error) = selector.failure.take() {
            return Err(error);
        }
        if count != 0 {
            record_intersections(collections, pair, &segment, selector)?;
        }
    }
    Ok(())
}
fn record_intersections(
    collections: &WireCollections,
    pair: &WirePair,
    segment: &FaceSegment,
    selector: &SegmentSelector,
) -> Result<(), Failure> {
    let intersections = intersection_set(collections, pair.index)?;
    intersections.write().add(segment.edge_ptr.clone());
    for selected in 0..selector.indices.length() {
        let index = *selector.indices.value(selected).map_err(array_failure)?;
        let edge = pair
            .target
            .read()
            .value(index)
            .map_err(array_failure)?
            .edge_ptr
            .clone();
        intersections.write().add(edge);
    }
    Ok(())
}
fn intersection_set(
    collections: &WireCollections,
    index: i32,
) -> Result<Handle<FaceEdgeSet>, Failure> {
    let mut intersections = collections
        .intersections
        .read()
        .value(index)
        .map_err(array_failure)?
        .clone();
    if intersections.is_null() {
        intersections = new_edge_set();
        *collections
            .intersections
            .write()
            .change_value(index)
            .map_err(array_failure)? = intersections.clone();
    }
    Ok(intersections)
}
