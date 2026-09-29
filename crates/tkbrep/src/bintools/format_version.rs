// Copyright (c) 2020 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKBRep/BinTools/BinTools_FormatVersion.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BinTools_FormatVersion`: the versions of the binary shape format. The enumeration
//! and its LOWER/UPPER bounds are `i32` constants, because `FormatNb` is an `int` and
//! `THE_ASCII_VERSIONS` is indexed by it.

/// `BinTools_FormatVersion_VERSION_1`: no CurveOnSurface UV points; reading resets the
/// Checked flag and updates the UV points of each face.
pub const VERSION_1: i32 = 1;
/// `BinTools_FormatVersion_VERSION_2`: stores the CurveOnSurface UV points.
pub const VERSION_2: i32 = 2;
/// `BinTools_FormatVersion_VERSION_3` (OCCT 6.2.1): the vertex point representations
/// start with their type byte (#0009745).
pub const VERSION_3: i32 = 3;
/// `BinTools_FormatVersion_VERSION_4`: stores the per-vertex normals of the
/// triangulations, and no UV points any more.
pub const VERSION_4: i32 = 4;
/// `BinTools_FormatVersion_CURRENT`.
pub const CURRENT: i32 = VERSION_4;
/// `BinTools_FormatVersion_LOWER`.
pub const LOWER: i32 = VERSION_1;
/// `BinTools_FormatVersion_UPPER`.
pub const UPPER: i32 = VERSION_4;
