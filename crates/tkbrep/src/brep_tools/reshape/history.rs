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
// Rust translation of BRepTools_ReShape::History and the anonymous-namespace Add
// (ModelingData/TKBRep/BRepTools/BRepTools_ReShape.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_ReShape::History`: the recorded replacements as a history.
use super::ReShapeData;
use crate::brep_tools::History;
use crate::topods::{Shape, ShapeIterator};
use rustycad_tkernel::ncollection::{HashPolicy, IndexedMap, Map};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `NCollection_DefaultHasher<TopoDS_Shape>`: the shape hash and `operator==`
/// (`IsEqual`, the orientation included).
#[derive(Clone, Copy, Debug, Default)]
struct ShapeEqualHasher;

impl HashPolicy<Shape> for ShapeEqualHasher {
    fn hash(&self, key: &Shape) -> usize {
        key.hash_code()
    }
    fn equal(&self, first: &Shape, second: &Shape) -> bool {
        first == second
    }
}

/// The anonymous-namespace `Add(theMap, theShape)`: a wire, shell or compsolid adds its
/// sub-shapes instead of itself.
fn add_expanded(shape: &Shape, mut add: impl FnMut(Shape)) -> Result<(), Failure> {
    if !matches!(
        shape.shape_type()?,
        ShapeEnum::Wire | ShapeEnum::Shell | ShapeEnum::CompSolid
    ) {
        add(shape.clone());
        return Ok(());
    }
    for child in ShapeIterator::new(shape, true, true) {
        add(child);
    }
    Ok(())
}

/// `History()`: for each replaced vertex, edge, face or solid that is not itself a
/// replacement, the shapes its chain of replacements ends in (wires, shells and
/// compsolids expanded) as modifications, or its removal when there are none.
pub(super) fn history_of(data: &ReShapeData) -> Result<Handle<History>, Failure> {
    let history = History::new();
    let replaced: Vec<Shape> = data.with(|fields| {
        let mut keys = Vec::new();
        let mut cursor = fields.replacements.cursor();
        while cursor.more() {
            if let Ok(key) = cursor.key() {
                keys.push(key.clone());
            }
            cursor.next();
        }
        keys
    });
    for shape in replaced {
        let is_new = data.with(|fields| fields.new_shapes.contains(&shape));
        if !History::is_supported_type(&shape)? || is_new {
            continue;
        }
        let modified = final_shapes(data, &shape)?;
        if modified.is_empty() {
            history.remove(&shape)?;
            continue;
        }
        for result in modified.iter() {
            history.add_modified(&shape, result)?;
        }
    }
    Ok(Handle::new(history))
}

/// The ends of the replacement chains of `shape`, walked breadth first.
fn final_shapes(
    data: &ReShapeData,
    shape: &Shape,
) -> Result<Map<Shape, ShapeEqualHasher>, Failure> {
    let mut intermediates: IndexedMap<Shape, ShapeEqualHasher> = IndexedMap::default();
    let mut modified: Map<Shape, ShapeEqualHasher> = Map::default();
    intermediates.add(shape.clone());
    let mut index = 1;
    while index <= intermediates.extent() {
        let intermediate = intermediates.find_key(index)?.clone();
        let replacement = data.with(|fields| fields.replacements.seek(&intermediate).cloned());
        match replacement {
            None => add_expanded(&intermediate, |s| {
                modified.add(s);
            })?,
            Some(replacement) if !replacement.is_removal() => {
                let result = replacement.relation_result().clone();
                if !result.is_null() {
                    add_expanded(&result, |s| {
                        intermediates.add(s);
                    })?;
                }
            }
            Some(_) => {}
        }
        index += 1;
    }
    Ok(modified)
}
