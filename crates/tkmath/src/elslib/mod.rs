// Copyright (c) 1991-1999 Matra Datavision
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

//! ElSLib: values, derivatives, parameters and iso-curves of planes and quadrics.
//!
//! Every static function of the OCCT class is a free function here; the overloads taking gp
//! surfaces are the generic [`value`], [`d0`], [`d1`], [`d2`], [`d3`], [`dn`] and
//! [`parameters`] functions over [`ElementarySurface`] and [`CurvedSurface`]. ElSLib.cxx is
//! compiled with `No_Exception`, so none of these functions can fail.
mod derivatives;
mod first;
mod frame;
mod iso;
mod parameters;
mod second;
mod surfaces;
mod third;
mod values;
pub use derivatives::{cone_dn, cylinder_dn, plane_dn, sphere_dn, torus_dn};
pub use first::{SurfaceD1, cone_d1, cylinder_d1, plane_d1, sphere_d1, torus_d1};
pub use iso::{
    cone_u_iso, cone_v_iso, cylinder_u_iso, cylinder_v_iso, plane_u_iso, plane_v_iso, sphere_u_iso,
    sphere_v_iso, torus_u_iso, torus_v_iso,
};
pub use parameters::{
    cone_parameters, cylinder_parameters, plane_parameters, sphere_parameters, torus_parameters,
};
pub use second::{SurfaceD2, cone_d2, cylinder_d2, sphere_d2, torus_d2};
pub use surfaces::{CurvedSurface, ElementarySurface, d0, d1, d2, d3, dn, parameters, value};
pub use third::{SurfaceD3, cone_d3, cylinder_d3, sphere_d3, torus_d3};
pub use values::{
    cone_d0, cone_value, cylinder_d0, cylinder_value, plane_d0, plane_value, sphere_d0,
    sphere_value, torus_d0, torus_value,
};
