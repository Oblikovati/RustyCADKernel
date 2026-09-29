// Created on: 1994-02-24
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
// Rust translation of ModelingData/TKBRep/BRepLProp at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! BRepLProp: local properties of edges and faces.
//!
//! `BRepLProp_CLProps` and `BRepLProp_SLProps` are the GeomLProp templates over the
//! BRepAdaptor curve and surface ([`BRepLPropCLProps`], [`BRepLPropSLProps`]); the
//! adaptors evaluate themselves (`DirectAccess`), as natively. [`continuity`] measures
//! the continuity of two edges at a junction and [`BRepLPropSurfaceTool`] is the
//! static surface tool.
mod access;
mod continuity;
mod surface_tool;

pub use continuity::{continuity, continuity_with_tolerances};
pub use surface_tool::BRepLPropSurfaceTool;

use crate::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkgeombase::geom_lprop::{CLPropsBase, SLPropsBase};
use rustycad_tkmath::gp::Vector3;

/// `BRepLProp_CLProps`: `GeomLProp_CLPropsBase<gp_Pnt, gp_Vec, gp_Dir,
/// BRepAdaptor_Curve>`. The native properties hold a copy of the adaptor; here they
/// borrow it, which evaluates the same curve.
pub type BRepLPropCLProps<'a> = CLPropsBase<Vector3, &'a BRepAdaptorCurve>;

/// `BRepLProp_SLProps`: `GeomLProp_SLPropsBase<BRepAdaptor_Surface>`, borrowing the
/// adaptor as [`BRepLPropCLProps`] does.
pub type BRepLPropSLProps<'a> = SLPropsBase<&'a BRepAdaptorSurface>;
