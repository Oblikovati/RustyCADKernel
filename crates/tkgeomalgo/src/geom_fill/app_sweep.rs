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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_AppSweep.hxx and
// GeomFill_AppSweep_0.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_AppSweep`: the instance of the template `AppBlend_AppSurf.gxx` on
//! `GeomFill_Line` and `GeomFill_SweepSectionGenerator`.
use crate::app_blend::AppBlendAppSurf;

/// The approximation of the sections of `GeomFill_SweepSectionGenerator`
/// (`GeomFill_AppSweep`). As for `GeomFill_AppSurf`, the template stores neither
/// its line nor its generator, so the instance is the AppBlend type itself
/// (docs/geomalgo-design.md sections 7 and 14); `GeomFillSweepSectionGenerator`
/// implements `TheSectionGenerator`.
///
/// `let mut app = GeomFillAppSweep::new(3, 8, 1e-7, 1e-9, 0, false);
/// app.perform(line.get(), &mut generator, false)?;`
pub type GeomFillAppSweep = AppBlendAppSurf;
