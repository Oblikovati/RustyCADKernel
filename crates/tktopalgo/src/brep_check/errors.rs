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
// Rust translation of the exceptions raised in ModelingAlgorithms/TKTopAlgo/BRepCheck at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The exceptions of BRepCheck, with the offending value in the message.
use rustycad_tkernel::standard::Failure;

/// Standard_NoSuchObject: an iterator read past its end or a missing binding.
pub(crate) fn no_such_object(message: &str) -> Failure {
    Failure::with_message("Standard_NoSuchObject", message)
}

/// Standard_NullObject: where the native member dereferences a null handle or a
/// failed down cast (a crash natively).
pub(crate) fn null_object(message: &str) -> Failure {
    Failure::with_message("Standard_NullObject", message)
}

/// Standard_OutOfRange: an index outside the bounds of a native collection.
pub(crate) fn out_of_range(message: &str) -> Failure {
    Failure::with_message("Standard_OutOfRange", message)
}

/// Standard_TypeMismatch: a `TopoDS::X` cast of a shape of another type.
pub(crate) fn type_mismatch(message: &str) -> Failure {
    Failure::with_message("Standard_TypeMismatch", message)
}
