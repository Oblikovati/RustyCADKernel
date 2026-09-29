// Created on: 1998-06-03
// Created by: data exchange team
// Copyright (c) 1998-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of BRepTools_ReShape::Status, Apply, applyImpl and the static
// CopyRanges and EncodeStatus (ModelingData/TKBRep/BRepTools/BRepTools_ReShape.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Applying the recorded replacements to a shape and its sub-shapes.
use super::{ReShape, ReplacementKind};
use crate::brep::access::edge_tshape;
use crate::brep::update_curves::as_gcurve;
use crate::brep::{BRepBuilder, BRepTool};
use crate::topods::{self, Shape, ShapeIterator};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// `NCollection_Map<occ::handle<TopoDS_TShape>>` of `applyImpl`: the TShapes being
/// processed up the call stack (the cycle guard).
#[derive(Default)]
pub struct ApplyGuard {
    in_flight: Vec<usize>,
}

impl ApplyGuard {
    fn contains(&self, shape: &Shape) -> bool {
        self.in_flight.contains(&shape.tshape_address())
    }
    fn add(&mut self, shape: &Shape) {
        self.in_flight.push(shape.tshape_address());
    }
    fn remove(&mut self, shape: &Shape) {
        let address = shape.tshape_address();
        if let Some(position) = self.in_flight.iter().position(|a| *a == address) {
            self.in_flight.remove(position);
        }
    }
}

/// The static `EncodeStatus`: the ShapeExtend status bits (1 to 8 are DONE1 to DONE8,
/// 9 DONE, 10 to 17 FAIL1 to FAIL8, 18 FAIL).
fn encode_status(status: i32) -> i32 {
    match status {
        1..=8 => 1 << (status - 1),
        9 => 0x00ff,
        10..=17 => 0x0100 << (status - 10),
        18 => 0xff00,
        _ => 0,
    }
}

/// Whether `result` replaces `shape`: another TShape with the location mode, another
/// shape (IsSame) otherwise.
fn is_replaced<R: ReShape + ?Sized>(reshape: &R, result: &Shape, shape: &Shape) -> bool {
    if reshape.mode_consider_location() {
        return !result.is_partner(shape);
    }
    !result.is_same(shape)
}

/// `Status(ashape, newsh, last)`; see [`ReShape::status`].
pub(super) fn status<R: ReShape + ?Sized>(
    reshape: &R,
    shape: &Shape,
    last: bool,
) -> Result<(i32, Shape), Failure> {
    if shape.is_null() {
        return Ok((0, Shape::default()));
    }
    let location = shape.location().clone();
    let mut key = shape.clone();
    let consider = reshape.mode_consider_location();
    if consider {
        key.set_location(&Default::default(), false)?;
    }
    let stored = reshape
        .reshape_data()
        .with(|fields| fields.replacements.seek(&key).map(|r| r.result()));
    let (mut result, mut status) = match stored {
        Some(result) => (result, 1),
        None => (key.clone(), 0),
    };
    if status > 0 {
        if result.is_null() {
            status = -1;
        } else if result.is_equal(&key) {
            status = 0;
        } else if last && is_replaced(reshape, &result, &key) {
            // sln 29.11.01 Bug24: follow the replacements the way they are stored.
            result = reshape.apply(&key, ShapeEnum::Shape)?;
            if result.is_null() {
                status = -1;
            }
            if result.is_equal(&key) {
                status = 0;
            }
        }
    }
    if consider && !result.is_null() {
        let relocated = if status > 0 && !result.location().is_identity() {
            location.multiplied(result.location())?
        } else {
            location
        };
        result.set_location(&relocated, false)?;
    }
    Ok((status, result))
}

/// `applyImpl(shape, until, theInFlight)`; see [`ReShape::apply`].
pub(super) fn apply_impl<R: ReShape + ?Sized>(
    reshape: &R,
    shape: &Shape,
    until: ShapeEnum,
    in_flight: &mut ApplyGuard,
) -> Result<Shape, Failure> {
    set_status(reshape, encode_status(0));
    if shape.is_null() {
        return Ok(shape.clone());
    }
    let replaced = reshape.value(shape)?;
    if replaced.is_null() {
        set_status(reshape, encode_status(2));
        return Ok(replaced);
    }
    // DFS cycle guard: a shape met again up the call stack is replaced by a shape that
    // contains it; its direct replacement is returned without descending.
    if in_flight.contains(shape) {
        return Ok(replaced);
    }
    if is_replaced(reshape, &replaced, shape) {
        in_flight.add(shape);
        let result = apply_impl(reshape, &replaced, until, in_flight)?;
        in_flight.remove(shape);
        reshape
            .reshape_data()
            .with_mut(|fields| fields.status |= encode_status(1));
        return Ok(result);
    }
    let kind = shape.shape_type()?;
    if kind > until || (kind == until && until > ShapeEnum::Compound) {
        return Ok(replaced);
    }
    if kind == ShapeEnum::Vertex || kind == ShapeEnum::Shape {
        return Ok(shape.clone());
    }
    rebuild(reshape, shape, until, in_flight)
}

fn set_status<R: ReShape + ?Sized>(reshape: &R, status: i32) {
    reshape
        .reshape_data()
        .with_mut(|fields| fields.status = status);
}

fn current_status<R: ReShape + ?Sized>(reshape: &R) -> i32 {
    reshape.reshape_data().with(|fields| fields.status)
}

/// The state of the rebuilt copy of a shape.
struct Rebuilt {
    result: Shape,
    modified: bool,
    is_empty: bool,
    status: i32,
}

/// Rebuilds an empty copy of the shape with the sub-shapes applied; the copy is
/// recorded as the replacement of the shape (a removal when it ends empty).
fn rebuild<R: ReShape + ?Sized>(
    reshape: &R,
    shape: &Shape,
    until: ShapeEnum,
    in_flight: &mut ApplyGuard,
) -> Result<Shape, Failure> {
    let kind = shape.shape_type()?;
    let mut rebuilt = Rebuilt {
        result: shape.empty_copied()?,
        modified: false,
        is_empty: true,
        status: current_status(reshape),
    };
    let orientation = shape.orientation();
    // Protect against INTERNAL or EXTERNAL shapes.
    rebuilt.result.set_orientation(Orientation::Forward);
    in_flight.add(shape);
    for child in ShapeIterator::new(shape, false, true) {
        let applied = apply_impl(reshape, &child, until, in_flight)?;
        add_applied(reshape, kind, (&child, &applied), &mut rebuilt)?;
    }
    in_flight.remove(shape);
    if !rebuilt.modified {
        return Ok(shape.clone());
    }
    let mut result = rebuilt.result;
    if rebuilt.is_empty && !matches!(kind, ShapeEnum::Vertex | ShapeEnum::Edge | ShapeEnum::Face) {
        // An empty container is removed.
        result = Shape::default();
    } else {
        restore_attributes(kind, shape, &result)?;
        result.set_orientation(orientation);
    }
    let replacement_kind = if result.is_null() {
        ReplacementKind::Remove
    } else {
        ReplacementKind::Modify
    };
    reshape.replace_with_kind(shape, &result, replacement_kind)?;
    set_status(reshape, rebuilt.status);
    Ok(result)
}

/// Adds one applied sub-shape to the copy: whole when the types match (always in a
/// compound), else its sub-shapes of the original type (a failure status when there
/// are none or they have another type).
fn add_applied<R: ReShape + ?Sized>(
    reshape: &R,
    kind: ShapeEnum,
    (child, applied): (&Shape, &Shape),
    rebuilt: &mut Rebuilt,
) -> Result<(), Failure> {
    if applied != child {
        if current_status(reshape) & encode_status(4) != 0 {
            rebuilt.status |= encode_status(4);
        }
        rebuilt.modified = true;
    }
    if applied.is_null() {
        rebuilt.status |= encode_status(4);
        return Ok(());
    }
    rebuilt.is_empty = false;
    rebuilt.status |= encode_status(3);
    let builder = BRepBuilder::default();
    if kind == ShapeEnum::Compound || applied.shape_type()? == child.shape_type()? {
        // Fix for SAMTECH bug OCC322 about absent internal vertices after sewing.
        return builder.add(&mut rebuilt.result, applied);
    }
    let mut count = 0;
    for sub in ShapeIterator::new(applied, true, true) {
        count += 1;
        if sub.shape_type()? == child.shape_type()? {
            builder.add(&mut rebuilt.result, &sub)?;
        } else {
            rebuilt.status |= encode_status(10);
        }
    }
    if count == 0 {
        rebuilt.status |= encode_status(10);
    }
    Ok(())
}

/// What `EmptyCopied` lost: the ranges of an edge, the natural restriction of a face,
/// the closed flag of a wire or a shell.
fn restore_attributes(kind: ShapeEnum, shape: &Shape, result: &Shape) -> Result<(), Failure> {
    match kind {
        ShapeEnum::Edge => copy_ranges(result, shape, 0.0, 1.0),
        ShapeEnum::Face => {
            if BRepTool::natural_restriction(&topods::face(shape)?)? {
                BRepBuilder::default().natural_restriction(&topods::face(result)?, true)?;
            }
            Ok(())
        }
        ShapeEnum::Wire | ShapeEnum::Shell => result.set_closed(BRepTool::is_closed(result)?),
        _ => Ok(()),
    }
}

/// The static `CopyRanges(toedge, fromedge, alpha, beta)`: each 3D curve and p-curve of
/// `from` gives its range, scaled by `alpha` and `beta`, to the first matching curve of
/// `to` (a 3D curve, or a p-curve on the same surface and location).
fn copy_ranges(to: &Shape, from: &Shape, alpha: f64, beta: f64) -> Result<(), Failure> {
    let from_curves = edge_tshape(from, "BRepTools_ReShape::CopyRanges")?.curves();
    let to_curves = edge_tshape(to, "BRepTools_ReShape::CopyRanges")?.curves();
    for representation in &from_curves {
        let from_gc = as_gcurve(representation);
        let Some(from_gc) = from_gc.get() else {
            continue;
        };
        let is_3d = from_gc.is_curve3d();
        if is_3d && from_gc.curve3d()?.is_null() {
            continue;
        }
        if !is_3d && (from_gc.pcurve()?.is_null() || !from_gc.is_curve_on_surface()) {
            continue;
        }
        let key = if is_3d {
            None
        } else {
            Some((from_gc.surface()?, from_gc.location()))
        };
        let target = to_curves.iter().map(as_gcurve).find(|candidate| {
            candidate
                .get()
                .is_some_and(|gc| matches_curve(gc, is_3d, key.as_ref()))
        });
        if let Some(target) = target.as_ref().and_then(|t| t.get()) {
            let (first, last) = (from_gc.first(), from_gc.last());
            let length = last - first;
            target.set_range(first + alpha * length, first + beta * length)?;
        }
    }
    Ok(())
}

type SurfaceKey = (
    Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
    rustycad_tkmath::toploc::Location,
);

fn matches_curve(
    candidate: &dyn crate::brep::BRepGCurve,
    is_3d: bool,
    key: Option<&SurfaceKey>,
) -> bool {
    if is_3d {
        return candidate.is_curve3d();
    }
    let Some((surface, location)) = key else {
        return false;
    };
    candidate.is_curve_on_surface()
        && candidate.surface().is_ok_and(|s| &s == surface)
        && &candidate.location() == location
}
