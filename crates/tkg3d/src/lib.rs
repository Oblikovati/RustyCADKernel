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
// Rust translation of ModelingData/TKG3d/(Geom package and the GeomEval evaluation descriptors it includes) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT TKG3d: transient 3D geometry.
//!
//! The Geom class hierarchy maps to Rust as described in docs/geom-design.md,
//! with the rules of the Geom2d port in `rustycad-tkg2d`: abstract classes are
//! traits reached through `Handle<dyn Trait>`, and each object keeps its state
//! behind one lock so that mutation through any handle is shared, as in OCCT.
//! TKG3d depends on TKG2d natively; this crate reuses the Geom2d lock, array and
//! dump helpers. The AdvApprox package, which `Geom_OffsetSurface` uses for its
//! iso curves, is ported in full. The other TKG3d packages (Adaptor3d,
//! GeomAdaptor, GeomEval curves and surfaces, GeomGridEval, GeomHash, TopAbs)
//! are later work.
pub mod adv_approx;
pub mod geom;
pub mod geom_eval;
pub mod geom_hash;
pub mod top_abs;
