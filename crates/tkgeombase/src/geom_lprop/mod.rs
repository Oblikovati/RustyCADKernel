// Created on: 1994-02-23
// Created by: Laurent BOURESCHE
// Copyright (c) 1994-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/GeomLProp at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GeomLProp: local properties of Geom curves and surfaces.
//!
//! [`CLPropsBase`] (`GeomLProp_CLPropsBase`, with the aliases [`CLProps`] and
//! [`CLProps2d`]) and [`SLPropsBase`] ([`SLProps`]) are the native class templates,
//! generic over the curve or surface type and the access policy;
//! [`CurAndInf2d`] finds the curvature extrema and inflections of a 2D curve, and
//! [`continuity`] measures the continuity of two curves at a junction.
mod clprops;
mod continuity;
mod cur_and_inf2d;
mod numeric_cur_inf2d;
mod slprops;
pub mod surface_access;
mod surface_curvatures;
pub mod surface_utils;

pub use clprops::{CLProps, CLProps2d, CLPropsBase};
pub use continuity::{continuity, continuity_with_tolerances};
pub use cur_and_inf2d::CurAndInf2d;
pub use slprops::{SLProps, SLPropsBase};
