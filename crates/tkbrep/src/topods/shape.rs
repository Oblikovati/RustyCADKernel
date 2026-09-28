// Created on: 1990-12-11
// Created by: Remi Lequette
// Copyright (c) 1990-1999 Matra Datavision
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
// Rust translation of ModelingData/TKBRep/TopoDS/TopoDS_Shape.hxx and
// TopoDS_Shape.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopoDS_Shape`: a located and oriented reference to a TShape (docs/topology-design.md
//! section 1). The flag members are in `shape_flags.rs`, identity and hashing in
//! `shape_identity.rs`.
use super::errors::null_shape;
use super::tshape::TShape;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};
use rustycad_tkmath::gp::Transform3;
use rustycad_tkmath::toploc::Location;
use std::io;

/// A shape: a TShape handle, a location and an orientation. A default shape is null.
///
/// ```
/// use rustycad_tkbrep::topods::{Builder, Shape, Wire};
/// use rustycad_tkg3d::top_abs::Orientation;
/// let mut wire = Wire::default();
/// Builder.make_wire(&mut wire);
/// let reversed: Shape = wire.reversed();
/// assert!(reversed.is_same(&wire) && !reversed.is_equal(&wire));
/// assert_eq!(reversed.orientation(), Orientation::Reversed);
/// ```
#[derive(Clone)]
pub struct Shape {
    tshape: Handle<dyn TShape>,
    location: Location,
    orient: Orientation,
}

impl Default for Shape {
    /// `TopoDS_Shape()`: no TShape, the identity location and EXTERNAL.
    fn default() -> Self {
        Self {
            tshape: Handle::default(),
            location: Location::default(),
            orient: Orientation::External,
        }
    }
}

impl Shape {
    /// A shape from its three fields, as the builders set them;
    /// `Shape::from_parts(tshape, Location::default(), Orientation::Forward)`.
    pub fn from_parts(
        tshape: Handle<dyn TShape>,
        location: Location,
        orientation: Orientation,
    ) -> Self {
        Self {
            tshape,
            location,
            orient: orientation,
        }
    }

    /// `IsNull()`: true when the shape references no TShape; `shape.is_null()`.
    pub fn is_null(&self) -> bool {
        self.tshape.is_null()
    }

    /// `Nullify()`: drops the TShape, clears the location and sets EXTERNAL;
    /// `shape.nullify()`.
    pub fn nullify(&mut self) {
        self.tshape.nullify();
        self.location.clear();
        self.orient = Orientation::External;
    }

    /// `Location()`: the local coordinate system; `shape.location()`.
    pub fn location(&self) -> &Location {
        &self.location
    }

    /// `Location(theLoc, theRaiseExc)`: sets the location; with `raise_exc`, a scaling or
    /// negative transformation raises Standard_DomainError and changes nothing;
    /// `shape.set_location(&location, false)?`.
    pub fn set_location(&mut self, location: &Location, raise_exc: bool) -> Result<(), Failure> {
        if raise_exc {
            validate_transformation(&location.transformation())?;
        }
        self.location = location.clone();
        Ok(())
    }

    /// `Located(theLoc, theRaiseExc)`: a copy with the location set;
    /// `shape.located(&location, false)?`.
    pub fn located(&self, location: &Location, raise_exc: bool) -> Result<Self, Failure> {
        let mut shape = self.clone();
        shape.set_location(location, raise_exc)?;
        Ok(shape)
    }

    /// `Orientation()`; `shape.orientation()`.
    pub fn orientation(&self) -> Orientation {
        self.orient
    }

    /// `Orientation(theOrient)`; `shape.set_orientation(Orientation::Reversed)`.
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orient = orientation;
    }

    /// `Oriented(theOrient)`: a copy with the orientation set; `shape.oriented(o)`.
    pub fn oriented(&self, orientation: Orientation) -> Self {
        let mut shape = self.clone();
        shape.set_orientation(orientation);
        shape
    }

    /// `TShape()`: the handle of the shared topology, null for a null shape; `shape.tshape()`.
    pub fn tshape(&self) -> &Handle<dyn TShape> {
        &self.tshape
    }

    /// `TShape(theTShape)`: replaces the TShape, keeping the location and orientation;
    /// `shape.set_tshape(&tshape)`.
    pub fn set_tshape(&mut self, tshape: &Handle<dyn TShape>) {
        self.tshape = tshape.clone();
    }

    /// The TShape, or Standard_NullObject where the native member dereferences a null
    /// handle; `shape.checked_tshape("TopoDS_Shape::Free")?`.
    pub(crate) fn checked_tshape(&self, member: &str) -> Result<&dyn TShape, Failure> {
        self.tshape.get().ok_or_else(|| null_shape(member))
    }

    /// `ShapeType()`: the TShape type; Standard_NullObject for a null shape, as the
    /// native documentation states; `shape.shape_type()?`.
    pub fn shape_type(&self) -> Result<ShapeEnum, Failure> {
        Ok(self.checked_tshape("TopoDS_Shape::ShapeType")?.shape_type())
    }

    /// `Move(thePosition, theRaiseExc)`: premultiplies the location by `position`;
    /// `shape.move_by(&location, false)?`.
    pub fn move_by(&mut self, position: &Location, raise_exc: bool) -> Result<(), Failure> {
        if raise_exc {
            validate_transformation(&position.transformation())?;
        }
        self.location = position.multiplied(&self.location)?;
        Ok(())
    }

    /// `Moved(thePosition, theRaiseExc)`: a moved copy; `shape.moved(&location, false)?`.
    pub fn moved(&self, position: &Location, raise_exc: bool) -> Result<Self, Failure> {
        let mut shape = self.clone();
        shape.move_by(position, raise_exc)?;
        Ok(shape)
    }

    /// `Reverse()`: `TopAbs::Reverse` of the orientation; `shape.reverse()`.
    pub fn reverse(&mut self) {
        self.orient = top_abs::reverse(self.orient);
    }

    /// `Reversed()`; `shape.reversed()`.
    pub fn reversed(&self) -> Self {
        let mut shape = self.clone();
        shape.reverse();
        shape
    }

    /// `Complement()`: `TopAbs::Complement` of the orientation; `shape.complement()`.
    pub fn complement(&mut self) {
        self.orient = top_abs::complement(self.orient);
    }

    /// `Complemented()`; `shape.complemented()`.
    pub fn complemented(&self) -> Self {
        let mut shape = self.clone();
        shape.complement();
        shape
    }

    /// `Compose(theOrient)`: `TopAbs::Compose(myOrient, theOrient)`; `shape.compose(o)`.
    pub fn compose(&mut self, orientation: Orientation) {
        self.orient = top_abs::compose(self.orient, orientation);
    }

    /// `Composed(theOrient)`; `shape.composed(o)`.
    pub fn composed(&self, orientation: Orientation) -> Self {
        let mut shape = self.clone();
        shape.compose(orientation);
        shape
    }

    /// `NbChildren()`: the number of direct sub-shapes, 0 for a null shape;
    /// `shape.nb_children()`.
    pub fn nb_children(&self) -> i32 {
        self.tshape.get().map_or(0, |tshape| tshape.nb_children())
    }

    /// `EmptyCopy()`: replaces the TShape by its empty copy; Standard_NullObject for a
    /// null shape; `shape.empty_copy()?`.
    pub fn empty_copy(&mut self) -> Result<(), Failure> {
        let copy = self.checked_tshape("TopoDS_Shape::EmptyCopy")?.empty_copy();
        self.tshape = copy;
        Ok(())
    }

    /// `EmptyCopied()`: a copy with an empty copy of the TShape; `shape.empty_copied()?`.
    pub fn empty_copied(&self) -> Result<Self, Failure> {
        let mut shape = self.clone();
        shape.empty_copy()?;
        Ok(shape)
    }

    /// `DumpJson(theOStream, theDepth)`: the class name, the TShape (when not null), the
    /// location and the orientation; `shape.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, output: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump::dump_class_begin(output, "TopoDS_Shape")?;
        if depth != 0
            && let Some(tshape) = self.tshape.get()
        {
            let mut field = DumpStream::default();
            tshape.dump_json(&mut field, depth - 1)?;
            dump::dump_key_to_class(output, "TShape", field.bytes())?;
        }
        if depth != 0 {
            let mut field = DumpStream::default();
            self.location.dump_json(&mut field, depth - 1)?;
            dump::dump_key_to_class(output, "Location", field.bytes())?;
        }
        dump::dump_integer(output, "Orient", self.orient as i32)
    }
}

/// The protected `validateTransformation`: a transformation with a scale factor other
/// than +-1 (beyond `TopLoc_Location::ScalePrec()`) or a negative determinant is refused.
fn validate_transformation(transform: &Transform3) -> Result<(), Failure> {
    let scale = transform.scale_factor();
    if (scale.abs() - 1.0).abs() > Location::scale_prec() || transform.is_negative() {
        return Err(Failure::with_type(
            "Standard_DomainError",
            &format!(
                "Transformation with scaling transformation is forbidden: scale {scale}, \
                 negative {}; expected a rigid motion",
                transform.is_negative()
            ),
            "",
        ));
    }
    Ok(())
}
