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
// Rust translation of ModelingData/TKBRep/TopoDS/TopoDS_FrozenShape.hxx,
// TopoDS_LockedShape.hxx and TopoDS_UnCompatibleShapes.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The exceptions TopoDS raises, as [`Failure`] values.
//!
//! `TopoDS_FrozenShape`, `TopoDS_LockedShape` and `TopoDS_UnCompatibleShapes` are
//! `DEFINE_STANDARD_EXCEPTION` classes derived from Standard_DomainError; the failure
//! keeps the most derived name.
use rustycad_tkernel::standard::Failure;

/// `TopoDS_FrozenShape`: the shape is not free and cannot be modified;
/// `frozen_shape("TopoDS_Builder::Add: parent is not free")`.
pub fn frozen_shape(message: &str) -> Failure {
    Failure::with_type("TopoDS_FrozenShape", message, "")
}

/// `TopoDS_LockedShape`: the shape is locked against modifications. TopoDS itself
/// never raises it; BRep_Builder does for a locked TShape; `locked_shape("locked")`.
pub fn locked_shape(message: &str) -> Failure {
    Failure::with_type("TopoDS_LockedShape", message, "")
}

/// `TopoDS_UnCompatibleShapes`: a component type that the parent type cannot hold;
/// `uncompatible_shapes("TopoDS_Builder::Add: VERTEX into WIRE")`.
pub fn uncompatible_shapes(message: &str) -> Failure {
    Failure::with_type("TopoDS_UnCompatibleShapes", message, "")
}

/// `Standard_NullObject` for a member that dereferences the TShape of a null shape (a
/// null handle dereference natively); `null_shape("TopoDS_Shape::ShapeType")`.
pub fn null_shape(member: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("{member}: the shape is null; expected a shape with a TShape"),
        "",
    )
}

/// `Standard_TypeMismatch` of the `TopoDS` casts;
/// `type_mismatch("TopoDS::Vertex", "EDGE")`.
pub fn type_mismatch(cast: &str, found: &str) -> Failure {
    Failure::with_type(
        "Standard_TypeMismatch",
        &format!("{cast}: shape type {found}; expected the cast type or a null shape"),
        "",
    )
}
