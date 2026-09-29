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
// Rust translation of the exceptions raised by the 2D curve intersection packages at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The native exceptions of the 2D curve intersection chain (IntRes2d, IntImpParGen,
//! IntCurve, Geom2dInt, Geom2dAPI) as [`Failure`] values that name the member.
use rustycad_tkernel::standard::Failure;

/// The `Standard_DomainError` a member raises outside its domain;
/// `domain_error("IntRes2d_Domain::FirstParameter", "the domain has no first point")`.
pub(crate) fn domain_error(member: &str, reason: &str) -> Failure {
    Failure::with_type(
        "Standard_DomainError",
        &format!("{member}() - {reason}"),
        "",
    )
}

/// The `StdFail_NotDone` of a result read before the computation succeeded;
/// `not_done("IntRes2d_Intersection::NbPoints")`.
pub(crate) fn not_done(member: &str) -> Failure {
    Failure::with_type(
        "StdFail_NotDone",
        &format!("{member}() - the computation is not done; expected IsDone()"),
        "",
    )
}

/// The `Standard_ConstructionError` of an invalid argument;
/// `construction_error("Domaine sur courbe incorrect")`.
pub(crate) fn construction_error(message: &str) -> Failure {
    Failure::with_type("Standard_ConstructionError", message, "")
}
