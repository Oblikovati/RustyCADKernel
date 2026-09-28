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
// Rust translation of ModelingData/TKBRep/TopoDS/TopoDS_Vertex.hxx, TopoDS_Edge.hxx,
// TopoDS_Wire.hxx, TopoDS_Face.hxx, TopoDS_Shell.hxx, TopoDS_Solid.hxx,
// TopoDS_CompSolid.hxx, TopoDS_Compound.hxx and the casts of TopoDS.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The typed shapes and the `TopoDS` casts (docs/topology-design.md section 1).
use super::Shape;
use super::errors::type_mismatch;
use rustycad_tkernel::ncollection::HashKey;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, ShapeEnum};
use std::ops::{Deref, DerefMut};

/// The check of every `TopoDS` cast: a null shape or a shape of the requested type.
fn check_cast(shape: &Shape, kind: ShapeEnum, cast: &str) -> Result<(), Failure> {
    let Some(tshape) = shape.tshape().get() else {
        return Ok(());
    };
    let found = tshape.shape_type();
    if found != kind {
        return Err(type_mismatch(cast, top_abs::shape_type_to_string(found)));
    }
    Ok(())
}

/// Declares a typed shape (a `TopoDS_Shape` subclass with no field) and its cast.
macro_rules! typed_shape {
    ($(#[$doc:meta])* $class:ident, $kind:expr, $cast:ident, $native_cast:literal) => {
        $(#[$doc])*
        #[derive(Clone, Default, PartialEq, Eq, Debug, Hash)]
        pub struct $class(Shape);

        impl $class {
            /// `*(TopoDS_X*)&S`: reinterprets `shape` without a type check, as the casts
            /// do where the native file defines `No_Standard_TypeMismatch`;
            /// `Vertex::unchecked(shape)`.
            pub fn unchecked(shape: Shape) -> Self {
                Self(shape)
            }
            /// The underlying shape value; `typed.into_shape()`.
            pub fn into_shape(self) -> Shape {
                self.0
            }
        }
        impl Deref for $class {
            type Target = Shape;
            fn deref(&self) -> &Shape {
                &self.0
            }
        }
        impl DerefMut for $class {
            fn deref_mut(&mut self) -> &mut Shape {
                &mut self.0
            }
        }
        impl AsRef<Shape> for $class {
            fn as_ref(&self) -> &Shape {
                &self.0
            }
        }
        impl From<$class> for Shape {
            fn from(typed: $class) -> Shape {
                typed.0
            }
        }
        /// `std::hash<TopoDS_X>`: the hash of the underlying shape.
        impl HashKey for $class {
            fn occt_hash(&self) -> usize {
                self.0.hash_code()
            }
        }

        #[doc = concat!("`", $native_cast, "(S)`: `S` as a typed shape; Standard_TypeMismatch when \
                         `S` is not null and has another type. Both native overloads (const and \
                         non-const reference) map here and return a copy.")]
        pub fn $cast(shape: &Shape) -> Result<$class, Failure> {
            check_cast(shape, $kind, $native_cast)?;
            Ok($class(shape.clone()))
        }
    };
}

typed_shape!(
    /// `TopoDS_Vertex`; `topods::vertex(&shape)?`.
    Vertex, ShapeEnum::Vertex, vertex, "TopoDS::Vertex"
);
typed_shape!(
    /// `TopoDS_Edge`; `topods::edge(&shape)?`.
    Edge, ShapeEnum::Edge, edge, "TopoDS::Edge"
);
typed_shape!(
    /// `TopoDS_Wire`; `topods::wire(&shape)?`.
    Wire, ShapeEnum::Wire, wire, "TopoDS::Wire"
);
typed_shape!(
    /// `TopoDS_Face`; `topods::face(&shape)?`.
    Face, ShapeEnum::Face, face, "TopoDS::Face"
);
typed_shape!(
    /// `TopoDS_Shell`; `topods::shell(&shape)?`.
    Shell, ShapeEnum::Shell, shell, "TopoDS::Shell"
);
typed_shape!(
    /// `TopoDS_Solid`; `topods::solid(&shape)?`.
    Solid, ShapeEnum::Solid, solid, "TopoDS::Solid"
);
typed_shape!(
    /// `TopoDS_CompSolid`; `topods::comp_solid(&shape)?`.
    CompSolid, ShapeEnum::CompSolid, comp_solid, "TopoDS::CompSolid"
);
typed_shape!(
    /// `TopoDS_Compound`; `topods::compound(&shape)?`.
    Compound, ShapeEnum::Compound, compound, "TopoDS::Compound"
);
