// Copyright (c) 2013-2014 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKernel/Standard/Standard_Time.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Standard_Time: the `IsEqual` overload for `std::time_t`.
//!
//! The C++ template is enabled only when `time_t` is a distinct type from `size_t` and
//! `int`, which holds on every supported profile, where `time_t` is a signed 64-bit count.

/// `std::time_t` on the supported profiles.
pub type TimeT = i64;

/// `IsEqual(time_t, time_t)`: value equality.
///
/// ```
/// assert!(rustycad_tkernel::standard::time::is_equal(1_700_000_000, 1_700_000_000));
/// ```
pub const fn is_equal(first: TimeT, second: TimeT) -> bool {
    first == second
}
