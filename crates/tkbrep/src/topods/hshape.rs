// Created on: 1995-06-28
// Created by: Modelistation
// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of ModelingData/TKBRep/TopoDS/TopoDS_HShape.hxx, TopoDS_HShape.lxx
// and TopoDS_HShape.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopoDS_HShape`: a shape shared by handle.
use super::Shape;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{SharedState, StandardTransient};

/// A transient holding one shape, so that several owners share and modify it. The
/// shape is behind the object lock (geom-design.md section 3): `Shape()` returns a
/// copy and `ChangeShape()` lends the stored shape to a closure.
///
/// ```
/// use rustycad_tkbrep::topods::{HShape, Shape};
/// use rustycad_tkg3d::top_abs::Orientation;
/// let holder = HShape::new(&Shape::default());
/// holder.change_shape(|shape| shape.set_orientation(Orientation::Forward));
/// assert_eq!(holder.shape().orientation(), Orientation::Forward);
/// ```
#[derive(Default)]
pub struct HShape {
    transient: StandardTransient,
    shape: SharedState<Shape>,
}
impl_transient!(HShape, "TopoDS_HShape", StandardTransient, transient);

impl HShape {
    /// `TopoDS_HShape(aShape)`; `HShape::new(&shape)`.
    pub fn new(shape: &Shape) -> Self {
        Self {
            transient: StandardTransient::default(),
            shape: SharedState::new(shape.clone()),
        }
    }

    /// `Shape(aShape)`: replaces the held shape; `holder.set_shape(&shape)`.
    pub fn set_shape(&self, shape: &Shape) {
        *self.shape.write() = shape.clone();
    }

    /// `Shape()`: a copy of the held shape; `holder.shape()`.
    pub fn shape(&self) -> Shape {
        self.shape.read().clone()
    }

    /// `ChangeShape()`: lends the held shape for modification;
    /// `holder.change_shape(|shape| shape.reverse())`.
    pub fn change_shape<R>(&self, change: impl FnOnce(&mut Shape) -> R) -> R {
        change(&mut self.shape.write())
    }
}
