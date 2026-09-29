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
// Rust translation of ModelingData/TKGeomBase/BndLib at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! BndLib: add the bounding box of a gp curve or surface, or of a curve or surface
//! adaptor, to an existing box.
//!
//! The static `BndLib::Add` overloads are module functions named by argument type
//! (`add_lin`, `add_circ`, `add_circ_arc`, `add_cylinder_patch` ...). Unlike GeomBndLib
//! they accumulate into the caller's box: the 3D circle and ellipse arcs read that box
//! back, so earlier content changes their result, as natively. `BndLib_Add3dCurve`,
//! `BndLib_Add2dCurve` and `BndLib_AddSurface` delegate to the GeomBndLib dispatchers.
mod add_curve;
mod add_surface;
pub mod compute;
mod conics;
mod open_curves;
mod quadrics;

pub use add_curve::{Add2dCurve, Add3dCurve};
pub use add_surface::AddSurface;
pub use conics::{
    add_circ, add_circ_arc, add_circ2d, add_circ2d_arc, add_elips, add_elips_arc, add_elips2d,
    add_elips2d_arc,
};
pub use open_curves::{add_hypr, add_hypr2d, add_lin, add_lin2d, add_parab, add_parab2d};
pub use quadrics::{
    add_cone, add_cone_patch, add_cylinder, add_cylinder_patch, add_sphere, add_sphere_patch,
    add_torus, add_torus_patch,
};
