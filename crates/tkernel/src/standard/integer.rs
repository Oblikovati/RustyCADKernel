// Copyright (c) 1998-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKernel/Standard/Standard_Integer.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Standard_Integer parity helpers with C++ truncated-remainder semantics.

/// `value % 2 == 0`; `assert!(is_even(-4))`.
pub const fn is_even(value: i32) -> bool {
    value % 2 == 0
}

/// `value % 2 == 1`, so negative odd values are neither odd nor even (the truncated remainder
/// is -1), exactly as the native test; `assert!(is_odd(3) && !is_odd(-3))`.
pub const fn is_odd(value: i32) -> bool {
    value % 2 == 1
}
