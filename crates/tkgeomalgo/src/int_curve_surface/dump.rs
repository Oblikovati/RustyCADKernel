// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of the std::cout output of the IntCurveSurface Dump members at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The text of a double written by `std::cout << value` in the pinned C locale.
use rustycad_tkernel::tcollection::AsciiString;

/// `operator<<(double)`: `%g` with six significant digits; `general(0.5) == "0.5"`.
pub(crate) fn general(value: f64) -> String {
    String::from_utf8_lossy(AsciiString::from_real(value).as_bytes()).into_owned()
}
