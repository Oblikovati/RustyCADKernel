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
// Rust translation of ModelingData/TKG2d (Geom2d and Geom2dEval packages) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT TKG2d: transient 2D geometry.
//!
//! The Geom2d class hierarchy maps to Rust as described in docs/geom-design.md:
//! abstract classes are traits reached through `Handle<dyn Trait>`, and each
//! object keeps its state behind one lock so that mutation through any handle is
//! shared, as in OCCT. The toolkit API and legacy-test acceptance remain in progress.
pub mod geom2d;
pub mod geom2d_eval;
