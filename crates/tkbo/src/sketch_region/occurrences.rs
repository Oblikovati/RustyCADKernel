// SPDX-License-Identifier: GPL-2.0-only
//! Physical history uses IsSame, while occurrence coverage retains direction and placement.
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::standard::{Failure, exceptions};

pub(super) fn equal_occurrences(root: &Shape) -> Result<Vec<Shape>, Failure> {
    let mut ordered = Vec::new();
    collect_occurrences(root, &mut ordered, &mut Vec::new())?;
    Ok(ordered)
}
fn collect_occurrences(
    shape: &Shape,
    ordered: &mut Vec<Shape>,
    ancestors: &mut Vec<usize>,
) -> Result<(), Failure> {
    let identity = shape.tshape_address();
    if ancestors.contains(&identity) {
        return Err(exceptions::construction_error(&format!(
            "topology={identity:#x} repeats an ancestor; expected an acyclic source graph"
        )));
    }
    if ordered.iter().any(|known| known.is_equal(shape)) {
        return Ok(());
    }
    ordered.push(shape.clone());
    ancestors.push(identity);
    let result = collect_children(shape, ordered, ancestors);
    ancestors.pop();
    result
}
fn collect_children(
    shape: &Shape,
    ordered: &mut Vec<Shape>,
    ancestors: &mut Vec<usize>,
) -> Result<(), Failure> {
    let mut children = ShapeIterator::new(shape, true, true);
    while children.more() {
        collect_occurrences(children.value()?, ordered, ancestors)?;
        children.next();
    }
    Ok(())
}
