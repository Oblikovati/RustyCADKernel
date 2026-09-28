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

//! The package's `DEFINE_STANDARD_EXCEPTION` classes as typed `Failure` constructors.
use rustycad_tkernel::standard::Failure;

/// `math_NotSquare` (a `Standard_DimensionError`), raised for non-square input;
/// `not_square("math_Matrix::Invert() - matrix is not square")`.
pub fn not_square(message: &str) -> Failure {
    Failure::with_type("math_NotSquare", message, "")
}

/// `math_SingularMatrix` (a `Standard_Failure`), raised when a factorization has no pivot;
/// `singular_matrix("")`.
pub fn singular_matrix(message: &str) -> Failure {
    Failure::with_type("math_SingularMatrix", message, "")
}
