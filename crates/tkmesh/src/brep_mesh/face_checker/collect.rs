use super::*;
use rustycad_tkernel::ncollection::{
    DynamicArray, EBTree, IncrementalAllocatorObject, UBTreeFiller,
};
pub(super) fn wire(checker: &BRepMeshFaceChecker, index: i32) -> Result<(), Failure> {
    let wire = checker.face.get_wire(index)?;
    let allocator = IncrementalAllocatorObject::default();
    let segments = shared_collection(DynamicArray::with_allocator(
        wire.edges_nb() as usize,
        Some(allocator.resource().clone()),
    ));
    let tree = shared_collection(EBTree::with_allocator(allocator.resource().clone()));
    let collections = checker.wires.snapshot();
    *collections
        .segments
        .write()
        .change_value(index)
        .map_err(array_failure)? = segments.clone();
    *collections
        .trees
        .write()
        .change_value(index)
        .map_err(array_failure)? = tree.clone();
    fill_segments(checker, &wire, &segments, &tree)
}
fn fill_segments(
    checker: &BRepMeshFaceChecker,
    wire: &crate::imesh_data::WireHandle,
    segments: &Handle<FaceSegments>,
    tree: &Handle<FaceBoundsTree>,
) -> Result<(), Failure> {
    let temporary = IncrementalAllocatorObject::default();
    let mut tree = tree.write();
    let mut filler = UBTreeFiller::new(&mut *tree, Some(temporary.resource().clone()), true);
    for edge_index in 0..wire.edges_nb() {
        let pointer = wire.get_edge(edge_index)?;
        let curve = pointer.upgrade().get_pcurve_for(
            &checker.face.downgrade(),
            wire.get_edge_orientation(edge_index)?,
        )?;
        append_segments(pointer, &curve, &mut segments.write(), &mut filler)?;
    }
    filler.fill();
    Ok(())
}

fn append_segments(
    pointer: crate::imesh_data::EdgePtr,
    curve: &crate::imesh_data::PCurveHandle,
    segments: &mut DynamicArray<FaceSegment>,
    filler: &mut UBTreeFiller<
        '_,
        i32,
        rustycad_tkmath::bnd::BoundingBox2,
        EBTree<i32, rustycad_tkmath::bnd::BoundingBox2>,
    >,
) -> Result<(), Failure> {
    for index in 1..curve.parameters_nb() {
        let first = CurvePointReference::new(curve.downgrade(), index - 1);
        let last = CurvePointReference::new(curve.downgrade(), index);
        let segment = FaceSegment::new(pointer.clone(), Some(first), Some(last));
        filler.add(segments.length(), segment.bounds()?);
        segments.append(segment);
    }
    Ok(())
}
