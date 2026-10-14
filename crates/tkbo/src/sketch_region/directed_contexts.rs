// SPDX-License-Identifier: GPL-2.0-only
//! Shared legacy traversal; child slots locate uses but never supply persistent identities.
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::standard::Failure;

#[derive(Clone)]
pub(crate) struct RegionDirectedContext {
    pub occurrence: Shape,
    pub ancestors: Vec<Shape>,
    pub child_slots: Vec<usize>,
}
pub(crate) enum RegionContextFailure {
    Native(Failure),
    RepeatedAncestor { member: Shape },
}
/// Retain cumulative actual uses, including repeated children; `strict_region_contexts(&root)`.
/// Legacy cycle detection includes placement; bounded admission must precede untrusted traversal.
pub(crate) fn strict_region_contexts(
    body: &Shape,
) -> Result<Vec<RegionDirectedContext>, RegionContextFailure> {
    let mut contexts = Vec::new();
    collect_contexts(body, &mut Vec::new(), &mut Vec::new(), &mut contexts)?;
    Ok(contexts)
}
fn collect_contexts(
    shape: &Shape,
    ancestors: &mut Vec<Shape>,
    slots: &mut Vec<usize>,
    contexts: &mut Vec<RegionDirectedContext>,
) -> Result<(), RegionContextFailure> {
    if ancestors.iter().any(|parent| parent.is_same(shape)) {
        return Err(RegionContextFailure::RepeatedAncestor {
            member: shape.clone(),
        });
    }
    contexts.push(context_record(shape, ancestors, slots));
    ancestors.push(shape.clone());
    collect_children(shape, ancestors, slots, contexts)?;
    ancestors.pop();
    Ok(())
}
fn collect_children(
    shape: &Shape,
    ancestors: &mut Vec<Shape>,
    slots: &mut Vec<usize>,
    contexts: &mut Vec<RegionDirectedContext>,
) -> Result<(), RegionContextFailure> {
    let mut iterator = ShapeIterator::new(shape, true, true);
    let mut index = 0;
    while iterator.more() {
        slots.push(index);
        let child = iterator.value().map_err(RegionContextFailure::Native)?;
        collect_contexts(child, ancestors, slots, contexts)?;
        slots.pop();
        index += 1;
        iterator.next();
    }
    Ok(())
}
fn context_record(shape: &Shape, ancestors: &[Shape], slots: &[usize]) -> RegionDirectedContext {
    RegionDirectedContext {
        occurrence: shape.clone(),
        ancestors: ancestors.to_vec(),
        child_slots: slots.to_vec(),
    }
}
