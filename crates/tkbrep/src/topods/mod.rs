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
// Rust translation of ModelingData/TKBRep/TopoDS at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! TopoDS: the topological data structure.
//!
//! A [`Shape`] is a value: a handle on a shared [`TShape`], a location and an
//! orientation. The typed shapes ([`Vertex`] ... [`Compound`]) are newtypes over it and
//! the `TopoDS` namespace casts are the functions [`vertex`] ... [`compound`].
//! [`Builder`] makes the shapes without geometry and links sub-shapes, and
//! [`ShapeIterator`] (`TopoDS_Iterator`) walks the direct sub-shapes. The mapping rules
//! are in docs/topology-design.md.
mod alerts;
mod builder;
pub mod errors;
mod hshape;
mod iterator;
mod shape;
mod shape_flags;
mod shape_identity;
mod tshape;
mod tshapes;
mod typed;

pub use alerts::{AlertAttribute, AlertWithShape};
pub use builder::Builder;
pub use hshape::HShape;
pub use iterator::ShapeIterator;
pub use shape::Shape;
pub(crate) use tshape::dump_tshape;
pub use tshape::{TShape, TShapeBase, bit_layout};
pub use tshapes::{TCompSolid, TCompound, TEdge, TFace, TShell, TSolid, TVertex, TWire};
pub use typed::{
    CompSolid, Compound, Edge, Face, Shell, Solid, Vertex, Wire, comp_solid, compound, edge, face,
    shell, solid, vertex, wire,
};
