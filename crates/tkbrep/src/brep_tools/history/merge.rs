// Created by: Eugeny MALTCHIKOV
// Copyright (c) 2017 OPEN CASCADE SAS
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
// Rust translation of BRepTools_History::Merge
// (ModelingData/TKBRep/BRepTools/BRepTools_History.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_History::Merge`: the composition of this history (1 to 2) with a next
//! one (2 to 3).
//!
//! The native maps are iterated in their bucket order, which follows the shape hashes
//! (the TShape addresses); the port iterates the ported maps in the same order.
use super::{History, HistoryAlgorithm, HistoryFields, ShapeLists};
use crate::topods::Shape;
use crate::toptools::MapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};

/// `add(theMap, theList)` and `add(theList, theMap)` of the anonymous namespace.
fn add_list_to_map(map: &mut MapOfShape, list: &List<Shape>) {
    for shape in list.iter() {
        map.add(shape.clone());
    }
}

fn add_map_to_list(list: &mut List<Shape>, map: &MapOfShape) {
    for shape in map.iter() {
        list.append(shape.clone());
    }
}

/// The shapes propagated by the first step.
#[derive(Default)]
struct Propagated {
    removed: MapOfShape,
    modified_or_generated: MapOfShape,
}

impl History {
    /// `Merge(theHistory23)` with a handle; a null handle merges nothing;
    /// `history.merge_handle(&next)?`.
    pub fn merge_handle(&self, next: &Handle<History>) -> Result<(), Failure> {
        match next.get() {
            Some(next) => self.merge(next),
            None => Ok(()),
        }
    }

    /// `Merge(theArguments, theAlgo)`: merges the history of an algorithm;
    /// `history.merge_algorithm(&arguments, &mut algorithm)?`.
    pub fn merge_algorithm(
        &self,
        arguments: &[Shape],
        algorithm: &mut dyn HistoryAlgorithm,
    ) -> Result<(), Failure> {
        self.merge(&History::from_algorithm(arguments, algorithm)?)
    }

    /// `Merge(theHistory23)`: the removals, modifications and generations of the next
    /// history applied to the results of this one, then its own relations of the shapes
    /// this history did not produce; the initial shapes left without result become
    /// removed; `history.merge(&next)?`.
    pub fn merge(&self, next: &History) -> Result<(), Failure> {
        let h23 = next.snapshot();
        if h23.modified.is_empty() && h23.generated.is_empty() && h23.removed.is_empty() {
            return Ok(());
        }
        let mut fields = self.fields.read().clone();
        let propagated = propagate_directly(&mut fields, &h23);
        propagate_sequentially(&mut fields, &h23, &propagated.modified_or_generated);
        unbind_empty(&mut fields);
        for shape in h23.removed.iter() {
            if !propagated.removed.contains(shape)
                && !fields.modified.is_bound(shape)
                && !fields.generated.is_bound(shape)
            {
                fields.removed.add(shape.clone());
            }
        }
        *self.fields.write() = fields;
        Ok(())
    }
}

/// Propagates R23, M23 and G23 to M12 and G12 directly (generations first).
fn propagate_directly(fields: &mut HistoryFields, h23: &HistoryFields) -> Propagated {
    let mut propagated = Propagated::default();
    for kind in 0..2 {
        let keys = map_keys(relation(fields, kind));
        for key in keys {
            propagate_list(fields, h23, (kind, &key), &mut propagated);
        }
    }
    propagated
}

/// `aS1ToGAndM[kind]`: the generations (0) or the modifications (1).
fn relation(fields: &HistoryFields, kind: usize) -> &ShapeLists {
    if kind == 0 {
        &fields.generated
    } else {
        &fields.modified
    }
}

fn relation_mut(fields: &mut HistoryFields, kind: usize) -> &mut ShapeLists {
    if kind == 0 {
        &mut fields.generated
    } else {
        &mut fields.modified
    }
}

/// One list of M12 or G12: its removed shapes dropped, its modified ones replaced by
/// their modifications, the generations of its shapes collected (and added to the
/// generations of the initial shape for a modification list).
fn propagate_list(
    fields: &mut HistoryFields,
    h23: &HistoryFields,
    (kind, key): (usize, &Shape),
    propagated: &mut Propagated,
) {
    let mut additions = [MapOfShape::default(), MapOfShape::default()];
    let Some(list) = relation_mut(fields, kind).change_seek(key) else {
        return;
    };
    let mut kept = List::default();
    for shape in list.iter() {
        if h23.removed.contains(shape) {
            propagated.removed.add(shape.clone());
            continue;
        }
        if let Some(generations) = h23.generated.seek(shape) {
            add_list_to_map(&mut additions[0], generations);
            propagated.modified_or_generated.add(shape.clone());
        }
        match h23.modified.seek(shape) {
            Some(modifications) => {
                add_list_to_map(&mut additions[kind], modifications);
                propagated.modified_or_generated.add(shape.clone());
            }
            None => {
                kept.append(shape.clone());
            }
        }
    }
    add_map_to_list(&mut kept, &additions[kind]);
    *list = kept;
    if kind != 0 && !additions[0].is_empty() {
        let generations = fields.generated.try_bound(key.clone(), List::default());
        add_map_to_list(generations, &additions[0]);
    }
}

/// Propagates M23 and G23 of the shapes not propagated directly: their lists are
/// appended to M12 and G12 and they are no longer removed.
fn propagate_sequentially(
    fields: &mut HistoryFields,
    h23: &HistoryFields,
    propagated: &MapOfShape,
) {
    for kind in 0..2 {
        let source = relation(h23, kind);
        let mut cursor = source.cursor();
        while cursor.more() {
            if let (Ok(shape), Ok(list)) = (cursor.key(), cursor.value())
                && !propagated.contains(shape)
            {
                let target = relation_mut(fields, kind).try_bound(shape.clone(), List::default());
                for item in list.iter() {
                    target.append(item.clone());
                }
                fields.removed.remove(shape);
            }
            cursor.next();
        }
    }
}

/// Unbinds the empty lists of M12 and G12, whose initial shapes become removed.
fn unbind_empty(fields: &mut HistoryFields) {
    for kind in 0..2 {
        let empty: Vec<Shape> = {
            let mut keys = Vec::new();
            let mut cursor = relation(fields, kind).cursor();
            while cursor.more() {
                if let (Ok(shape), Ok(list)) = (cursor.key(), cursor.value())
                    && list.is_empty()
                {
                    keys.push(shape.clone());
                }
                cursor.next();
            }
            keys
        };
        for shape in empty {
            fields.removed.add(shape.clone());
            relation_mut(fields, kind).unbind(&shape);
        }
    }
}

/// The keys of a map in bucket order.
fn map_keys(map: &ShapeLists) -> Vec<Shape> {
    let mut keys = Vec::new();
    let mut cursor = map.cursor();
    while cursor.more() {
        if let Ok(key) = cursor.key() {
            keys.push(key.clone());
        }
        cursor.next();
    }
    keys
}
