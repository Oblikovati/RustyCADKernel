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
// Rust translation of ModelingData/TKBRep/BRepTools/BRepTools_History.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_History`: the history of the shapes of an operation (generated,
//! modified and removed shapes).
mod merge;

use crate::topexp;
use crate::topods::Shape;
use crate::toptools::{IndexedMapOfShape, MapOfShape, ShapeMapHasher};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `NCollection_DataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher>`.
pub(crate) type ShapeLists = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// `BRepTools_History::TRelationType`: how a shape relates to its initial shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationType {
    /// `TRelationType_Removed`.
    Removed,
    /// `TRelationType_Generated`.
    Generated,
    /// `TRelationType_Modified`.
    Modified,
}

/// The queries of an algorithm that the history is built from (the `TheAlgo` template
/// parameter of the native constructor and `Merge`).
pub trait HistoryAlgorithm {
    /// `theAlgo.IsDeleted(S)`.
    fn is_deleted(&mut self, shape: &Shape) -> Result<bool, Failure>;
    /// `theAlgo.Modified(S)`.
    fn modified(&mut self, shape: &Shape) -> Result<Vec<Shape>, Failure>;
    /// `theAlgo.Generated(S)`.
    fn generated(&mut self, shape: &Shape) -> Result<Vec<Shape>, Failure>;
}

/// The three relations of the history.
#[derive(Clone, Default)]
pub(crate) struct HistoryFields {
    pub(crate) modified: ShapeLists,
    pub(crate) generated: ShapeLists,
    pub(crate) removed: MapOfShape,
}

/// The history of an operation (`BRepTools_History`): for each initial vertex, edge,
/// face or solid, the shapes generated from it, the shapes it was modified into, or
/// its removal. A shape cannot be both modified and removed, nor both generated and
/// modified into the same shape; such additions override the previous relation, as
/// natively (whose assertions are silent in release builds).
///
/// ```
/// use rustycad_tkbrep::brep_tools::History;
/// let history = History::new();
/// assert!(!history.has_removed());
/// ```
#[derive(Default)]
pub struct History {
    parent: StandardTransient,
    fields: SharedState<HistoryFields>,
}
impl_transient!(History, "BRepTools_History", StandardTransient, parent);

impl History {
    /// `BRepTools_History()`; `History::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepTools_History(theArguments, theAlgo)`: for every supported sub-shape of the
    /// arguments (in `TopExp::MapShapes` order), its deletion, modifications and
    /// generations by `algorithm`; `History::from_algorithm(&arguments, &mut algorithm)?`.
    pub fn from_algorithm(
        arguments: &[Shape],
        algorithm: &mut dyn HistoryAlgorithm,
    ) -> Result<Self, Failure> {
        let history = Self::new();
        let mut shapes = IndexedMapOfShape::default();
        for argument in arguments.iter().filter(|shape| !shape.is_null()) {
            topexp::map_shapes(argument, &mut shapes, true, true);
        }
        for index in 1..=shapes.extent() {
            let shape = shapes.find_key(index)?.clone();
            if !Self::is_supported_type(&shape)? {
                continue;
            }
            if algorithm.is_deleted(&shape)? {
                history.remove(&shape)?;
            }
            for modified in algorithm.modified(&shape)? {
                history.add_modified(&shape, &modified)?;
            }
            for generated in algorithm.generated(&shape)? {
                history.add_generated(&shape, &generated)?;
            }
        }
        Ok(history)
    }

    /// `IsSupportedType(theShape)`: a vertex, an edge, a face or a solid;
    /// Standard_NullObject for a null shape (a crash natively);
    /// `History::is_supported_type(&shape)?`.
    pub fn is_supported_type(shape: &Shape) -> Result<bool, Failure> {
        Ok(matches!(
            shape.shape_type()?,
            ShapeEnum::Vertex | ShapeEnum::Edge | ShapeEnum::Face | ShapeEnum::Solid
        ))
    }

    /// Whether a null or supported shape may be an initial shape of a generation.
    fn accepts_generation(initial: &Shape) -> Result<bool, Failure> {
        Ok(initial.is_null() || Self::is_supported_type(initial)?)
    }

    /// `AddGenerated(theInitial, theGenerated)`: appends a generation (an initial shape of
    /// an unsupported type is ignored); `history.add_generated(&edge, &face)?`.
    pub fn add_generated(&self, initial: &Shape, generated: &Shape) -> Result<(), Failure> {
        if !self.prepare_generated(initial, generated)? {
            return Ok(());
        }
        let mut fields = self.fields.write();
        let list = fields.generated.try_bound(initial.clone(), List::default());
        list.append(generated.clone());
        Ok(())
    }

    /// `AddModified(theInitial, theModified)`: appends a modification (it cancels a
    /// removal); `history.add_modified(&face, &new_face)?`.
    pub fn add_modified(&self, initial: &Shape, modified: &Shape) -> Result<(), Failure> {
        if !self.prepare_modified(initial, modified)? {
            return Ok(());
        }
        let mut fields = self.fields.write();
        let list = fields.modified.try_bound(initial.clone(), List::default());
        list.append(modified.clone());
        Ok(())
    }

    /// `Remove(theRemoved)`: records a removal (it cancels the modifications);
    /// `history.remove(&face)?`.
    pub fn remove(&self, removed: &Shape) -> Result<(), Failure> {
        if !Self::is_supported_type(removed)? {
            return Ok(());
        }
        let mut fields = self.fields.write();
        fields.modified.unbind(removed);
        fields.removed.add(removed.clone());
        Ok(())
    }

    /// `ReplaceGenerated(theInitial, theGenerated)`: the generations of the initial shape
    /// become the one shape; `history.replace_generated(&edge, &face)?`.
    pub fn replace_generated(&self, initial: &Shape, generated: &Shape) -> Result<(), Failure> {
        if !self.prepare_generated(initial, generated)? {
            return Ok(());
        }
        let mut fields = self.fields.write();
        let mut list = List::default();
        list.append(generated.clone());
        fields.generated.bind(initial.clone(), list);
        Ok(())
    }

    /// `ReplaceModified(theInitial, theModified)`: the modifications of the initial shape
    /// become the one shape; `history.replace_modified(&face, &new_face)?`.
    pub fn replace_modified(&self, initial: &Shape, modified: &Shape) -> Result<(), Failure> {
        if !self.prepare_modified(initial, modified)? {
            return Ok(());
        }
        let mut fields = self.fields.write();
        let mut list = List::default();
        list.append(modified.clone());
        fields.modified.bind(initial.clone(), list);
        Ok(())
    }

    /// `Clear()`; `history.clear()`.
    pub fn clear(&self) {
        let mut fields = self.fields.write();
        fields.modified.clear(true);
        fields.generated.clear(true);
        fields.removed.clear(true);
    }

    /// `Generated(theInitial)`: the generations (empty for an unsupported type);
    /// `history.generated(&edge)?`.
    pub fn generated(&self, initial: &Shape) -> Result<Vec<Shape>, Failure> {
        if !Self::accepts_generation(initial)? {
            return Ok(Vec::new());
        }
        let fields = self.fields.read();
        Ok(fields
            .generated
            .seek(initial)
            .map(|list| list.iter().cloned().collect())
            .unwrap_or_default())
    }

    /// `Modified(theInitial)`: the modifications (empty for an unsupported type);
    /// `history.modified(&face)?`.
    pub fn modified(&self, initial: &Shape) -> Result<Vec<Shape>, Failure> {
        if !Self::is_supported_type(initial)? {
            return Ok(Vec::new());
        }
        let fields = self.fields.read();
        Ok(fields
            .modified
            .seek(initial)
            .map(|list| list.iter().cloned().collect())
            .unwrap_or_default())
    }

    /// `IsRemoved(theInitial)` (false for an unsupported type); `history.is_removed(&face)?`.
    pub fn is_removed(&self, initial: &Shape) -> Result<bool, Failure> {
        if !Self::is_supported_type(initial)? {
            return Ok(false);
        }
        Ok(self.fields.read().removed.contains(initial))
    }

    /// `HasGenerated()`; `history.has_generated()`.
    pub fn has_generated(&self) -> bool {
        !self.fields.read().generated.is_empty()
    }

    /// `HasModified()`; `history.has_modified()`.
    pub fn has_modified(&self) -> bool {
        !self.fields.read().modified.is_empty()
    }

    /// `HasRemoved()`; `history.has_removed()`.
    pub fn has_removed(&self) -> bool {
        !self.fields.read().removed.is_empty()
    }

    /// `Dump(theS)`: the numbers of removed, modified and generated shapes;
    /// `history.dump(&mut stream)`.
    pub fn dump(&self, stream: &mut IoStream) {
        let fields = self.fields.read();
        stream.insert_bytes(b"History contains:\n - ");
        stream.insert_integer(fields.removed.extent() as i64);
        stream.insert_bytes(b" Deleted shapes;\n - ");
        stream.insert_integer(fields.modified.extent() as i64);
        stream.insert_bytes(b" Modified shapes;\n - ");
        stream.insert_integer(fields.generated.extent() as i64);
        stream.insert_bytes(b" Generated shapes.\n");
    }

    /// `prepareGenerated`: false for an unsupported initial shape; the generated shape is
    /// no longer a modification of it.
    fn prepare_generated(&self, initial: &Shape, generated: &Shape) -> Result<bool, Failure> {
        if !Self::accepts_generation(initial)? {
            return Ok(false);
        }
        if let Some(list) = self.fields.write().modified.change_seek(initial) {
            list.remove_value(generated);
        }
        Ok(true)
    }

    /// `prepareModified`: false for an unsupported initial shape; the initial shape is no
    /// longer removed and the modified shape no longer a generation of it.
    fn prepare_modified(&self, initial: &Shape, modified: &Shape) -> Result<bool, Failure> {
        if !Self::is_supported_type(initial)? {
            return Ok(false);
        }
        let mut fields = self.fields.write();
        fields.removed.remove(initial);
        if let Some(list) = fields.generated.change_seek(initial) {
            list.remove_value(modified);
        }
        Ok(true)
    }

    /// A copy of the three relations.
    pub(crate) fn snapshot(&self) -> HistoryFields {
        self.fields.read().clone()
    }
}
