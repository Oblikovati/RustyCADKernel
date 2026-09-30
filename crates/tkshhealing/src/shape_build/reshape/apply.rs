// Rust translation of ShapeBuild_ReShape::Apply(shape, until) and applyImpl of
// ShapeBuild_ReShape.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The recursive `Apply` of a ShapeBuild re-shaper.
use super::ShapeBuildReShape;
use crate::shape_build::ShapeBuildEdge;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{ReShape, ReplacementKind};
use rustycad_tkbrep::topods::{Edge, Shape, ShapeIterator};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

fn bits(status: ShapeExtendStatus) -> i32 {
    ShapeExtend::encode_status(status)
}

/// `applyImpl(theShape, theUntil, theInFlight)`: `in_flight` holds the TShapes being
/// processed up the call stack, which breaks a replacement by a compound that contains
/// the shape (cyclic containment, distinct from cycles in the replacement map).
pub(super) fn apply_impl(
    reshape: &ShapeBuildReShape,
    shape: &Shape,
    until: ShapeEnum,
    in_flight: &mut Vec<usize>,
) -> Result<Shape, Failure> {
    reshape.set_status(bits(ShapeExtendStatus::Ok));
    if shape.is_null() {
        return Ok(shape.clone());
    }
    // Apply direct replacement
    let replaced = reshape.value(shape)?;
    if replaced.is_null() {
        reshape.set_status(bits(ShapeExtendStatus::Done2));
        return Ok(replaced);
    }
    let address = shape.tshape_address();
    if in_flight.contains(&address) {
        return Ok(replaced);
    }
    if is_replaced(reshape, &replaced, shape) {
        in_flight.push(address);
        let result = apply_impl(reshape, &replaced, until, in_flight);
        remove_address(in_flight, address);
        let result = result?;
        reshape.set_status(reshape.status_bits() | bits(ShapeExtendStatus::Done1));
        return Ok(result);
    }
    let kind = shape.shape_type()?;
    if kind >= until {
        return Ok(replaced); // stop criterion
    }
    if kind == ShapeEnum::Vertex || kind == ShapeEnum::Shape {
        return Ok(shape.clone());
    }
    rebuild(reshape, shape, kind, until, in_flight)
}

/// Whether the direct replacement is another shape: another TShape with the location
/// mode (IsPartner), another shape (IsSame) without it.
fn is_replaced(reshape: &ShapeBuildReShape, replaced: &Shape, shape: &Shape) -> bool {
    if reshape.mode_consider_location() {
        return !replaced.is_partner(shape);
    }
    !replaced.is_same(shape)
}

/// `theInFlight.Remove(theShape.TShape())`.
fn remove_address(in_flight: &mut Vec<usize>, address: usize) {
    if let Some(position) = in_flight.iter().position(|&a| a == address) {
        in_flight.remove(position);
    }
}

/// The rebuilt shape and the local status of the sub-shape loop.
struct Rebuilt {
    result: Shape,
    modified: bool,
    status: i32,
}

/// The second half of `applyImpl`: an empty copy of the shape (FORWARD, to protect
/// INTERNAL and EXTERNAL shapes) filled with the applied sub-shapes; the shape itself
/// when nothing changed, else the copy, recorded as the replacement of the shape.
fn rebuild(
    reshape: &ShapeBuildReShape,
    shape: &Shape,
    kind: ShapeEnum,
    until: ShapeEnum,
    in_flight: &mut Vec<usize>,
) -> Result<Shape, Failure> {
    let mut result = shape.empty_copied()?;
    let orientation = shape.orientation();
    result.set_orientation(Orientation::Forward);
    let address = shape.tshape_address();
    in_flight.push(address);
    let rebuilt = rebuild_children(reshape, shape, kind, until, in_flight, result);
    remove_address(in_flight, address);
    let Rebuilt {
        mut result,
        modified,
        status,
    } = rebuilt?;
    if !modified {
        return Ok(shape.clone());
    }
    // Restore range on edge broken by EmptyCopied()
    if kind == ShapeEnum::Edge {
        ShapeBuildEdge.copy_ranges(
            &Edge::unchecked(result.clone()),
            &Edge::unchecked(shape.clone()),
            0.0,
            1.0,
        )?;
    } else if kind == ShapeEnum::Wire || kind == ShapeEnum::Shell {
        result.set_closed(BRepTool::is_closed(&result)?)?;
    }
    result.set_orientation(orientation);
    reshape.set_status(status);
    let replacement = if result.is_null() {
        ReplacementKind::Remove
    } else {
        ReplacementKind::Modify
    };
    reshape.replace_with_kind(shape, &result, replacement)?;
    Ok(result)
}

/// The loop over the sub-shapes (iterated without their cumulated orientation).
fn rebuild_children(
    reshape: &ShapeBuildReShape,
    shape: &Shape,
    kind: ShapeEnum,
    until: ShapeEnum,
    in_flight: &mut Vec<usize>,
    result: Shape,
) -> Result<Rebuilt, Failure> {
    let mut rebuilt = Rebuilt {
        result,
        modified: false,
        status: reshape.status_bits(),
    };
    let mut it = ShapeIterator::new(shape, false, true);
    while it.more() {
        let sub = it.value()?.clone();
        it.next();
        let applied = apply_impl(reshape, &sub, until, in_flight)?;
        add_applied(reshape, &mut rebuilt, kind, &sub, &applied)?;
    }
    Ok(rebuilt)
}

/// One sub-shape of the loop: the flags, then the applied shape added as is (in a
/// compound, or when of the same type), or its items of the sub-shape's type.
fn add_applied(
    reshape: &ShapeBuildReShape,
    rebuilt: &mut Rebuilt,
    kind: ShapeEnum,
    sub: &Shape,
    applied: &Shape,
) -> Result<(), Failure> {
    if applied.is_not_equal(sub) {
        if ShapeExtend::decode_status(reshape.status_bits(), ShapeExtendStatus::Done4) {
            rebuilt.status |= bits(ShapeExtendStatus::Done4);
        }
        rebuilt.modified = true;
    }
    if applied.is_null() {
        rebuilt.status |= bits(ShapeExtendStatus::Done4);
        return Ok(());
    }
    rebuilt.status |= bits(ShapeExtendStatus::Done3);
    let builder = BRepBuilder::default();
    let sub_kind = sub.shape_type()?;
    if kind == ShapeEnum::Compound || applied.shape_type()? == sub_kind {
        // Fix for SAMTECH bug OCC322 about absence internal vertices after sewing
        return builder.add(&mut rebuilt.result, applied);
    }
    let mut nb_items = 0;
    let mut items = ShapeIterator::new(applied, true, true);
    while items.more() {
        let item = items.value()?.clone();
        items.next();
        nb_items += 1;
        if item.shape_type()? == sub_kind {
            builder.add(&mut rebuilt.result, &item)?;
        } else {
            rebuilt.status |= bits(ShapeExtendStatus::Fail1);
        }
    }
    if nb_items == 0 {
        rebuilt.status |= bits(ShapeExtendStatus::Fail1);
    }
    Ok(())
}
