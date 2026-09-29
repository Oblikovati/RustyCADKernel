// Created on: 1994-08-25
// Created by: Jacques GOUSSARD
// Copyright (c) 1994-1999 Matra Datavision
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
// Rust translation of ModelingData/TKBRep/BRepTools/BRepTools_Modifier.hxx/.lxx and the
// constructors, Init, Perform, Put, IsMutableInput and SetMutableInput of
// BRepTools_Modifier.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_Modifier`: rebuilds a shape with the geometry a modification gives.
mod prepare;
mod rebuild;
mod rebuild_face;

use super::modification::{BRepToolsModification, NewCurve, NewSurface};
use crate::brep::BRepBuilder;
use crate::brep::errors::null_object;
use crate::topexp;
use crate::topods::{self, Face, Shape, ShapeIterator};
use crate::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape, ShapeMapHasher};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::Shape as Continuity;

/// `NCollection_DataMap<TopoDS_Shape, TopoDS_Shape, TopTools_ShapeMapHasher>`.
type ShapeResults = DataMap<Shape, Shape, ShapeMapHasher>;

/// Rebuilds a shape with the new geometry given by a [`BRepToolsModification`]
/// (`BRepTools_Modifier`): each sub-shape whose geometry or sub-shapes change is
/// rebuilt, the others are kept.
///
/// ```
/// use rustycad_tkbrep::brep_tools::Modifier;
/// let modifier = Modifier::new(false);
/// assert!(!modifier.is_done());
/// ```
#[derive(Default)]
pub struct Modifier {
    map: ShapeResults,
    shape: Shape,
    done: bool,
    nc_info: DataMap<Shape, NewCurve, ShapeMapHasher>,
    ns_info: DataMap<Shape, NewSurface, ShapeMapHasher>,
    non_upd_face: MapOfShape,
    has_new_geom: MapOfShape,
    mutable_input: bool,
}

impl Modifier {
    /// `BRepTools_Modifier(theMutableInput)`: no shape yet; `Modifier::new(false)`.
    pub fn new(mutable_input: bool) -> Self {
        Self {
            mutable_input,
            ..Self::default()
        }
    }

    /// `BRepTools_Modifier(S)`: records the sub-shapes of `shape`;
    /// `Modifier::with_shape(&shape)?`.
    pub fn with_shape(shape: &Shape) -> Result<Self, Failure> {
        let mut modifier = Self::new(false);
        modifier.init(shape)?;
        Ok(modifier)
    }

    /// `BRepTools_Modifier(S, M)`: records the sub-shapes of `shape` and performs the
    /// modification; `Modifier::with_modification(&shape, &modification)?`.
    pub fn with_modification(
        shape: &Shape,
        modification: &Handle<dyn BRepToolsModification>,
    ) -> Result<Self, Failure> {
        let mut modifier = Self::with_shape(shape)?;
        modifier.perform(modification, &ProgressRange::default())?;
        Ok(modifier)
    }

    /// `Init(S)`: sets the shape and records its sub-shapes (the results of a previous
    /// `Perform` are kept for the shapes already recorded, as natively);
    /// `modifier.init(&shape)?`.
    pub fn init(&mut self, shape: &Shape) -> Result<(), Failure> {
        self.shape = shape.clone();
        self.done = false;
        self.put(shape);
        Ok(())
    }

    /// `Put(S)`: binds `shape` and, recursively, its sub-shapes to a null result.
    fn put(&mut self, shape: &Shape) {
        if self.map.is_bound(shape) {
            return;
        }
        self.map.bind(shape.clone(), Shape::default());
        for child in ShapeIterator::new(shape, false, true) {
            self.put(&child);
        }
    }

    /// `IsDone()`; `modifier.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `IsMutableInput()`; `modifier.is_mutable_input()`.
    pub fn is_mutable_input(&self) -> bool {
        self.mutable_input
    }

    /// `SetMutableInput(theMutableInput)`: with a mutable input, the vertices and edges
    /// that keep their geometry are reused instead of copied;
    /// `modifier.set_mutable_input(true)`.
    pub fn set_mutable_input(&mut self, mutable_input: bool) {
        self.mutable_input = mutable_input;
    }

    /// `ModifiedShape(S)`: the result for `shape`; Standard_NoSuchObject for a shape
    /// that is not a sub-shape of the modified shape; `modifier.modified_shape(&face)?`.
    pub fn modified_shape(&self, shape: &Shape) -> Result<Shape, Failure> {
        self.map.find(shape).cloned().map_err(|_| {
            Failure::with_type(
                "Standard_NoSuchObject",
                "BRepTools_Modifier::ModifiedShape: the shape is not a sub-shape of the modified shape",
                "",
            )
        })
    }

    /// `myMap(S)`: the stored result; Standard_NoSuchObject for an unbound shape.
    fn result_of(&self, shape: &Shape) -> Result<Shape, Failure> {
        self.modified_shape(shape)
    }

    /// `myMap(S) = result`.
    fn store(&mut self, shape: &Shape, result: &Shape) -> Result<(), Failure> {
        let slot = self.map.change_find(shape).map_err(|_| {
            Failure::with_type(
                "Standard_NoSuchObject",
                "BRepTools_Modifier: the shape is not recorded",
                "",
            )
        })?;
        *slot = result.clone();
        Ok(())
    }

    /// `Perform(M, theProgress)`: computes the new vertices, curves and surfaces, then
    /// rebuilds the shape; the result gets the orientation of the shape, and the
    /// continuity of each rebuilt edge between two faces is updated. Standard_NullObject
    /// without a shape; nothing is done when the progress indicator breaks;
    /// `modifier.perform(&modification, &ProgressRange::default())?`.
    pub fn perform(
        &mut self,
        modification: &Handle<dyn BRepToolsModification>,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        if self.shape.is_null() {
            return Err(null_object("BRepTools_Modifier::Perform: no shape"));
        }
        let m = modification
            .get()
            .ok_or_else(|| null_object("BRepTools_Modifier::Perform: null modification"))?;
        let scope = ProgressScope::with_literal(progress, "Converting Shape", 2.0, false);
        let mut mve = IndexedDataMapOfShapeListOfShape::default();
        let mut mef = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            &self.shape,
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut mve,
        )?;
        topexp::map_shapes_and_ancestors(&self.shape, ShapeEnum::Edge, ShapeEnum::Face, &mut mef)?;
        self.create_new_vertices(&mve, m)?;
        self.fill_new_curve_info(&mef, m)?;
        self.fill_new_surface_info(m)?;
        if !self.mutable_input {
            self.create_other_vertices(&mve, &mef, m)?;
        }
        let mut new_geom = false;
        let root = self.shape.clone();
        self.rebuild(&root, m, &mut new_geom, &scope.next(1.0))?;
        if !scope.more() {
            return Ok(());
        }
        self.orient_result()?;
        self.update_continuities(&mef, m)?;
        self.done = true;
        Ok(())
    }

    /// The result of the shape gets its orientation (a REVERSED face result is reversed
    /// instead).
    fn orient_result(&mut self) -> Result<(), Failure> {
        let shape = self.shape.clone();
        let mut result = self.result_of(&shape)?;
        if shape.shape_type()? == ShapeEnum::Face && shape.orientation() == Orientation::Reversed {
            result.reverse();
        } else {
            result.set_orientation(shape.orientation());
        }
        self.store(&shape, &result)
    }

    /// The continuity of each rebuilt edge between its first two faces, when the
    /// modification gives one above C0.
    fn update_continuities(
        &self,
        mef: &IndexedDataMapOfShapeListOfShape,
        m: &dyn BRepToolsModification,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        for index in 1..=mef.extent() {
            let edge = topods::edge(mef.find_key(index)?)?;
            let new_edge = topods::edge(&self.result_of(&edge)?)?;
            if edge.is_same(&new_edge) {
                continue;
            }
            let faces: Vec<&Shape> = mef.find_from_index(index)?.iter().take(2).collect();
            let [first, second] = faces.as_slice() else {
                continue;
            };
            let (f1, f2): (Face, Face) = (topods::face(first)?, topods::face(second)?);
            let new_f1 = topods::face(&self.result_of(&f1)?)?;
            let new_f2 = topods::face(&self.result_of(&f2)?)?;
            let continuity = m.continuity(&edge, &f1, &f2, &new_edge, &new_f1, &new_f2)?;
            if continuity > Continuity::C0 {
                builder.continuity_on_faces(&new_edge, &new_f1, &new_f2, continuity)?;
            }
        }
        Ok(())
    }
}

/// `SetShapeFlags(theInSh, theOutSh)`: the Modified, Checked, Orientable, Closed,
/// Infinite and Convex flags of the result copied from the source.
fn set_shape_flags(source: &Shape, result: &Shape) -> Result<(), Failure> {
    result.set_modified(source.modified()?)?;
    result.set_checked(source.checked()?)?;
    result.set_orientable(source.orientable()?)?;
    result.set_closed(source.closed()?)?;
    result.set_infinite(source.infinite()?)?;
    result.set_convex(source.convex()?)
}
