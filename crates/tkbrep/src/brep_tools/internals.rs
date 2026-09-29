// Created on: 1993-01-21
// Created by: Remi LEQUETTE
// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of BRepTools::RemoveInternals and BRepTools::CheckLocations with the
// anonymous-namespace helpers (ModelingData/TKBRep/BRepTools/BRepTools.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Removing INTERNAL sub-shapes and finding bad locations.
use crate::brep::BRepBuilder;
use crate::topexp;
use crate::topods::{Shape, ShapeIterator};
use crate::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::toploc::Location;

/// The direct sub-shapes of `shape` with the cumulated orientation and location.
fn children(shape: &Shape) -> Vec<Shape> {
    ShapeIterator::new(shape, true, true).collect()
}

/// The internal and non-internal sub-shapes met so far and those found in both roles.
#[derive(Default)]
struct InternalRoles {
    non_internals: MapOfShape,
    internals: MapOfShape,
    to_keep: MapOfShape,
}

/// `findInternalsToKeep`: depth first, each sub-shape is recorded as internal or not,
/// and kept when it has been met in both roles (it preserves the connectivity).
fn find_internals_to_keep(shape: &Shape, roles: &mut InternalRoles) {
    for child in children(shape) {
        find_internals_to_keep(&child, roles);
        if child.orientation() == Orientation::Internal {
            roles.internals.add(child.clone());
        } else {
            roles.non_internals.add(child.clone());
        }
        if roles.non_internals.contains(&child) && roles.internals.contains(&child) {
            roles.to_keep.add(child);
        }
    }
}

/// `removeShapes`: removes `to_remove` from `shape`, unfreezing it for the time of the
/// removal.
fn remove_shapes(shape: &mut Shape, to_remove: &[Shape]) -> Result<(), Failure> {
    let was_free = shape.free()?;
    shape.set_free(true)?;
    let builder = BRepBuilder::default();
    for child in to_remove {
        builder.remove(shape, child)?;
    }
    shape.set_free(was_free)
}

/// `removeInternals`: removes the INTERNAL sub-shapes not in `to_keep`, recursively,
/// and the sub-shapes emptied by the recursion; true when `shape` lost sub-shapes and
/// has none left.
fn remove_internals_of(shape: &mut Shape, to_keep: Option<&MapOfShape>) -> Result<bool, Failure> {
    let mut to_remove = Vec::new();
    for mut child in children(shape) {
        if child.orientation() == Orientation::Internal {
            if to_keep.is_none_or(|keep| !keep.contains(&child)) {
                to_remove.push(child);
            }
        } else if remove_internals_of(&mut child, to_keep)? {
            to_remove.push(child);
        }
    }
    if to_remove.is_empty() {
        return Ok(false);
    }
    remove_shapes(shape, &to_remove)?;
    Ok(shape.nb_children() == 0)
}

/// `BRepTools::RemoveInternals(theS, theForce)`: removes the INTERNAL sub-shapes of
/// `shape` recursively (and the sub-shapes left empty). Without `force`, the internal
/// sub-shapes also used as non-internal ones are kept, to preserve the connectivity;
/// `remove_internals(&mut shape, false)?`.
pub fn remove_internals(shape: &mut Shape, force: bool) -> Result<(), Failure> {
    let mut roles = InternalRoles::default();
    if !force {
        find_internals_to_keep(shape, &mut roles);
    }
    let to_keep = (roles.to_keep.extent() > 0).then_some(&roles.to_keep);
    remove_internals_of(shape, to_keep)?;
    Ok(())
}

/// `BRepTools::CheckLocations(theS, theProblemShapes)`: appends to `problems` every
/// sub-shape (and the shape) whose own location is negative or scales by more than
/// `TopLoc_Location::ScalePrec()`; `check_locations(&shape, &mut problems)`.
pub fn check_locations(shape: &Shape, problems: &mut Vec<Shape>) {
    if shape.is_null() {
        return;
    }
    let mut shapes = IndexedMapOfShape::default();
    topexp::map_shapes(shape, &mut shapes, false, false);
    for index in 1..=shapes.extent() {
        let Ok(current) = shapes.find_key(index) else {
            continue;
        };
        let trsf = current.location().transformation();
        let is_bad =
            trsf.is_negative() || (trsf.scale_factor().abs() - 1.0).abs() > Location::scale_prec();
        if is_bad {
            problems.push(current.clone());
        }
    }
}
