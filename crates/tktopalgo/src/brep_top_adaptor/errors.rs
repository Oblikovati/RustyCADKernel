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
// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepTopAdaptor at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The exceptions of BRepTopAdaptor, with the offending value in the message.
use rustycad_tkernel::standard::Failure;

/// Standard_NullObject: where the native member dereferences a null handle or a
/// failed down cast (a crash natively).
pub(crate) fn null_object(message: &str) -> Failure {
    Failure::with_message("Standard_NullObject", message)
}

/// Standard_ConstructionError: a handle of the wrong adaptor type.
pub(crate) fn construction_error(message: &str) -> Failure {
    Failure::with_message("Standard_ConstructionError", message)
}

/// Standard_DomainError: an arc or a vertex without a 3d representation.
pub(crate) fn domain_error(message: &str) -> Failure {
    Failure::with_message("Standard_DomainError", message)
}

/// Standard_NotImplemented: `Initialize()` without a surface.
pub(crate) fn not_implemented(message: &str) -> Failure {
    Failure::with_message("Standard_NotImplemented", message)
}

/// Standard_NoSuchObject: an iterator read past its end (a null dereference natively).
pub(crate) fn no_such_object(message: &str) -> Failure {
    Failure::with_message("Standard_NoSuchObject", message)
}

/// The `(int)` cast of a double in the native build: out-of-range and NaN casts are
/// undefined in C++ and x86 `cvttsd2si` yields `INT_MIN` for them (kept on x86), where
/// Rust's `as` saturates.
pub(crate) fn c_int_cast(value: f64) -> i32 {
    let in_range: bool = value > -2_147_483_649.0 && value < 2_147_483_648.0;
    if cfg!(any(target_arch = "x86", target_arch = "x86_64")) && !in_range {
        return i32::MIN;
    }
    value as i32
}
