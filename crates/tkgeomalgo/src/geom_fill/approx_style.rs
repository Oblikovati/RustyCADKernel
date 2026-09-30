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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_ApproxStyle.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_ApproxStyle`: which part of a sweep drives its approximation.

/// The approximation style (`GeomFill_ApproxStyle`), with the native values;
/// `GeomFillApproxStyle::Location as i32 == 1`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum GeomFillApproxStyle {
    /// `GeomFill_Section`.
    #[default]
    Section = 0,
    /// `GeomFill_Location`.
    Location = 1,
}
