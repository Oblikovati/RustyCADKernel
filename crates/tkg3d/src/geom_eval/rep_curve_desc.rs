// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_RepCurveDesc.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomEval_RepCurveDesc`: descriptors that redirect the evaluation of a curve
//! to another curve (its "evaluation representation").
//!
//! The native descriptors expose public mutable fields through shared handles;
//! here each descriptor keeps them behind a lock with getters and setters.
rustycad_tkg2d::define_rep_curve_descriptors!(dyn crate::geom::GeomCurve);
