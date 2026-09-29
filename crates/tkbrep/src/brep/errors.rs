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
// Rust translation of ModelingData/TKBRep/BRep/BRep_Builder.hxx/.cxx, BRep_Tool.hxx/.cxx, BRep_CurveRepresentation.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The exceptions of the BRep package, as [`Failure`] values.
//!
//! Natively the representations raise Standard_DomainError from the members their
//! class does not define, BRep_Builder raises TopoDS_LockedShape and
//! Standard_DomainError, and BRep_Tool raises Standard_NoSuchObject and
//! Standard_NullObject. The TShape casts that are undefined behaviour natively raise
//! Standard_NullObject or Standard_TypeMismatch here (docs/topology-design.md section 9).
use rustycad_tkernel::standard::Failure;

/// Standard_DomainError of a representation member that its class does not define
/// (the native message is the base class name);
/// `undefined_member("BRep_CurveRepresentation", "Curve3D")`.
pub fn undefined_member(class: &str, member: &str) -> Failure {
    Failure::with_type(
        "Standard_DomainError",
        &format!(
            "{class}: {member} is not defined by this representation; expected a class that overrides it"
        ),
        "",
    )
}

/// Standard_DomainError with the native message and the offending value;
/// `domain_error("BRep_Builder::Infinite parameter", "parameter 1e100")`.
pub fn domain_error(message: &str, value: &str) -> Failure {
    Failure::with_type("Standard_DomainError", &format!("{message}: {value}"), "")
}

/// Standard_NoSuchObject with the native message and the offending value;
/// `no_such_object("BRep_Tool:: no parameter on edge", "vertex not on the edge")`.
pub fn no_such_object(message: &str, value: &str) -> Failure {
    Failure::with_type("Standard_NoSuchObject", &format!("{message}: {value}"), "")
}

/// Standard_NullObject for a null handle or shape that the native code dereferences;
/// `null_object("BRep_Tool:: TopoDS_Vertex hasn't gp_Pnt")`.
pub fn null_object(message: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("{message}; expected a non-null object"),
        "",
    )
}

/// Standard_TypeMismatch for a TShape that is not the BRep class a member casts it to
/// (undefined behaviour natively); `tshape_mismatch("BRep_Tool::Tolerance", "BRep_TEdge", "TopoDS_TEdge")`.
pub fn tshape_mismatch(member: &str, expected: &str, found: &str) -> Failure {
    Failure::with_type(
        "Standard_TypeMismatch",
        &format!("{member}: TShape of class {found}; expected a {expected}"),
        "",
    )
}

/// Standard_ProgramError of a failed `Standard_ASSERT_RAISE` (raised in release builds too);
/// `program_error("Try to set list with NULL triangulation to the face", "index 2")`.
pub fn program_error(description: &str, value: &str) -> Failure {
    Failure::with_type(
        "Standard_ProgramError",
        &format!("*** ERROR: ASSERT: {description}: {value}"),
        "",
    )
}
