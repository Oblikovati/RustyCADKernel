// Created on: 1993-01-19
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
// Rust translation of TopExp::MapShapesAndAncestors and
// TopExp::MapShapesAndUniqueAncestors (ModelingData/TKBRep/TopExp/TopExp.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The ancestor maps: each sub-shape of one type bound to the list of the shapes of
//! another type that contain it.
use super::Explorer;
use crate::topods::Shape;
use crate::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// How an ancestor joins the list of a sub-shape.
#[derive(Clone, Copy)]
enum Append {
    /// `MapShapesAndAncestors`: always appended.
    Always,
    /// `MapShapesAndUniqueAncestors`: appended unless an equal (`IsEqual` when
    /// `useOrientation`, else `IsSame`) ancestor is already listed.
    Unique { use_orientation: bool },
}

/// `MapShapesAndAncestors(S, TS, TA, M)`: binds each sub-shape of type `sub_kind` to
/// the list of the sub-shapes of type `ancestor_kind` containing it (in exploration
/// order, repeated when an ancestor holds it twice); the sub-shapes that are under no
/// ancestor get an empty list. The map is not cleared first.
///
/// ```
/// use rustycad_tkbrep::topexp;
/// use rustycad_tkbrep::topods::{Builder, Compound};
/// use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
/// use rustycad_tkg3d::top_abs::ShapeEnum;
/// let mut compound = Compound::default();
/// Builder.make_compound(&mut compound);
/// let mut map = IndexedDataMapOfShapeListOfShape::default();
/// topexp::map_shapes_and_ancestors(&compound, ShapeEnum::Compound, ShapeEnum::Solid, &mut map)?;
/// assert_eq!(map.extent(), 1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub fn map_shapes_and_ancestors(
    shape: &Shape,
    sub_kind: ShapeEnum,
    ancestor_kind: ShapeEnum,
    map: &mut IndexedDataMapOfShapeListOfShape,
) -> Result<(), Failure> {
    bind_ancestors(shape, sub_kind, ancestor_kind, map, Append::Always)
}

/// `MapShapesAndUniqueAncestors(S, TS, TA, M, useOrientation)`: as
/// [`map_shapes_and_ancestors`], but each list holds an ancestor once (compared with
/// `IsEqual` when `use_orientation`, else `IsSame`); the native default of
/// `useOrientation` is false; `topexp::map_shapes_and_unique_ancestors(&s, ts, ta, &mut m, false)?`.
pub fn map_shapes_and_unique_ancestors(
    shape: &Shape,
    sub_kind: ShapeEnum,
    ancestor_kind: ShapeEnum,
    map: &mut IndexedDataMapOfShapeListOfShape,
    use_orientation: bool,
) -> Result<(), Failure> {
    let append = Append::Unique { use_orientation };
    bind_ancestors(shape, sub_kind, ancestor_kind, map, append)
}

/// The shared body: visit the ancestors, then the sub-shapes under no ancestor.
fn bind_ancestors(
    shape: &Shape,
    sub_kind: ShapeEnum,
    ancestor_kind: ShapeEnum,
    map: &mut IndexedDataMapOfShapeListOfShape,
    append: Append,
) -> Result<(), Failure> {
    let mut ancestors = Explorer::new(shape, ancestor_kind, ShapeEnum::Shape)?;
    while ancestors.more() {
        let ancestor = ancestors.current().clone();
        let mut subs = Explorer::new(&ancestor, sub_kind, ShapeEnum::Shape)?;
        while subs.more() {
            let index = index_of(map, subs.current());
            append_ancestor(map.change_from_index(index)?, &ancestor, append);
            subs.next()?;
        }
        ancestors.next()?;
    }
    let mut free = Explorer::new(shape, sub_kind, ancestor_kind)?;
    while free.more() {
        index_of(map, free.current());
        free.next()?;
    }
    Ok(())
}

/// `M.FindIndex(S)`, adding `S` with an empty list when absent.
fn index_of(map: &mut IndexedDataMapOfShapeListOfShape, shape: &Shape) -> usize {
    let index = map.find_index(shape);
    if index != 0 {
        return index;
    }
    map.add(shape.clone(), List::default())
}

fn append_ancestor(list: &mut List<Shape>, ancestor: &Shape, append: Append) {
    if let Append::Unique { use_orientation } = append {
        let listed = |shape: &Shape| {
            if use_orientation {
                ancestor.is_equal(shape)
            } else {
                ancestor.is_same(shape)
            }
        };
        if list.iter().any(listed) {
            return;
        }
    }
    list.append(ancestor.clone());
}
