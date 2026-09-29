// Created on: 1991-07-02
// Created by: Remi LEQUETTE
// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of ModelingData/TKBRep/BRep/BRep_Builder.hxx/.cxx, BRep_Tool.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The TShape casts of BRep_Builder and BRep_Tool.
//!
//! Natively both reinterpret the TShape handle of a shape as the BRep class without a
//! check. These functions check the cast: a null shape raises Standard_NullObject and a
//! TShape of another class raises Standard_TypeMismatch (docs/topology-design.md
//! section 9).
use super::errors::{null_object, tshape_mismatch};
use super::{BRepTEdge, BRepTFace, BRepTVertex};
use crate::topods::Shape;
use rustycad_tkernel::standard::{Failure, RuntimeTyped, Transient, project_concrete};

/// The TShape of `shape` as the BRep class `T`; `member` names the caller in errors.
fn brep_tshape<'a, T: RuntimeTyped + 'static>(
    shape: &'a Shape,
    member: &str,
) -> Result<&'a T, Failure> {
    let tshape = shape
        .tshape()
        .get()
        .ok_or_else(|| null_object(&format!("{member}: the shape is null")))?;
    let object: &dyn Transient = tshape;
    project_concrete::<T>(object)
        .ok_or_else(|| tshape_mismatch(member, T::TYPE_NAME, object.dynamic_type().name()))
}

/// The `BRep_TVertex` of a vertex; `vertex_tshape(&vertex, "BRep_Tool::Pnt")?`.
pub fn vertex_tshape<'a>(shape: &'a Shape, member: &str) -> Result<&'a BRepTVertex, Failure> {
    brep_tshape::<BRepTVertex>(shape, member)
}

/// The `BRep_TEdge` of an edge; `edge_tshape(&edge, "BRep_Tool::Tolerance")?`.
pub fn edge_tshape<'a>(shape: &'a Shape, member: &str) -> Result<&'a BRepTEdge, Failure> {
    brep_tshape::<BRepTEdge>(shape, member)
}

/// The `BRep_TFace` of a face; `face_tshape(&face, "BRep_Tool::Surface")?`.
pub fn face_tshape<'a>(shape: &'a Shape, member: &str) -> Result<&'a BRepTFace, Failure> {
    brep_tshape::<BRepTFace>(shape, member)
}
