// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/FEmTool and AppParCurves (stream output helper)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The text a native `Standard_OStream << double` writes, shared by the Dump and
//! Out members of the TKGeomBase ports.
use rustycad_tkernel::tcollection::AsciiString;

/// `Standard_OStream << double` with default flags, which formats like C's `%g`
/// (six significant digits); `stream_real(0.5) == "0.5"`.
pub(crate) fn stream_real(value: f64) -> String {
    // AsciiString(double) uses the same `%g` conversion.
    String::from_utf8_lossy(AsciiString::from_real(value).as_bytes()).into_owned()
}
