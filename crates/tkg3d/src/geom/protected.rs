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
// Rust translation of ModelingData/TKG3d/Geom (protected members of the abstract classes, see docs/geom-design.md) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Proof that a caller is inside this crate, for the native `protected` members
//! that abstract classes share with their subclasses (`Geom_Vector::gpVec`,
//! `Geom_Conic::pos`, `Geom_ElementarySurface::pos`...). Other crates cannot
//! build a value of it.

/// The access token of the protected trait members; only this crate builds one.
pub struct ProtectedAccess {
    _private: (),
}
pub(crate) const PROTECTED: ProtectedAccess = ProtectedAccess { _private: () };
