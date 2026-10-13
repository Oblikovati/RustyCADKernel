// SPDX-License-Identifier: GPL-2.0-only
//! Enumerate mapped context requirements explicitly, including missing entries.
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn checked_occurrences(shape: &Shape) -> Result<Vec<Shape>, Failure> {
    let mut found = Vec::new();
    append_occurrences(shape, &mut found)?;
    Ok(found)
}

fn append_occurrences(shape: &Shape, found: &mut Vec<Shape>) -> Result<(), Failure> {
    if found.iter().any(|entry| entry.is_equal(shape)) {
        return Ok(());
    }
    found.push(shape.clone());
    let mut children = ShapeIterator::new(shape, true, true);
    while let Ok(child) = children.value() {
        append_occurrences(child, found)?;
        children.next();
    }
    Ok(())
}

pub(super) fn required_contexts(shape: &Shape) -> Result<Vec<Shape>, Failure> {
    // Native contextual loops check one IsSame entity, not both orientations.
    let mut found = Vec::new();
    for kind in required_kinds(shape.shape_type()?) {
        for subshape in Explorer::new(shape, *kind, ShapeEnum::Shape)? {
            if !found.iter().any(|entry: &Shape| entry.is_same(&subshape)) {
                found.push(subshape);
            }
        }
    }
    Ok(found)
}

fn required_kinds(kind: ShapeEnum) -> &'static [ShapeEnum] {
    match kind {
        ShapeEnum::Edge => &[ShapeEnum::Vertex],
        ShapeEnum::Face => &[ShapeEnum::Wire, ShapeEnum::Edge, ShapeEnum::Vertex],
        ShapeEnum::Solid => &[ShapeEnum::Shell],
        _ => &[],
    }
}
