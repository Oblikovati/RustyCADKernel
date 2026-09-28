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
// Rust translation of ModelingData/TKGeomBase/Extrema at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The classic Extrema package: the extrema of the distance between points, curves
//! and surfaces.
//!
//! Each class drops the `Extrema_` prefix (`Extrema_ExtPElC` is [`ExtPElC`]). The
//! native class templates (`Extrema_GGExtPC`, `Extrema_GFuncExtPC`, ...) are generic
//! Rust types over the [`ExtremaCurveTool`] trait, which bundles their curve, point,
//! vector and tool parameters; the native aliases (`Extrema_ExtPC`,
//! `Extrema_ExtPC2d`, `Extrema_EPCOfExtPC`, ...) are type aliases over [`CurveTool`]
//! or [`Curve2dTool`]. An object that keeps a curve (`TheCurve*` natively) borrows it
//! for its lifetime. docs/tkgeombase-progress.md gives the full mapping;
//! `ExtPElC::with_line(point, line, 1e-7, -10.0, 10.0).nb_ext()? == 1`.
mod curve_curve;
mod curve_surface;
mod curve_tool;
mod curve_tools;
mod elementary_point_curve;
mod ext_el_c;
mod ext_el_c2d;
mod ext_el_cs;
mod ext_el_ss;
mod ext_p_el_c;
mod ext_p_el_c2d;
mod ext_p_el_s;
mod failures;
mod gp_space;
mod pair_result;
mod point_curve;
mod point_surface;
mod pon;
mod singular;
mod surface_ref;
mod surface_surface;

pub use curve_curve::{
    CCLocFOfLocECC, ECC, ECC2d, ExtCC, ExtCC2d, GFuncExtCC, GGenExtCC, GenLocateExtCC,
    GlobOptFuncCCC0, GlobOptFuncCCC1, GlobOptFuncCCC2, LocECC, LocateExtCC,
};
pub use curve_surface::{
    ExtCS, FuncExtCS, GenExtCS, GlobOptFuncCQuadric, GlobOptFuncCS, GlobOptFuncConicS,
};
pub use curve_tool::{ExtremaCurveTool, SplineKnots, ToolD1, ToolD2, ToolD3};
pub use curve_tools::{Curve2dTool, CurveTool};
pub use elementary_point_curve::ElementaryPointCurve;
pub use ext_el_c::ExtElC;
pub use ext_el_c2d::ExtElC2d;
pub use ext_el_cs::ExtElCS;
pub use ext_el_ss::ExtElSS;
pub use ext_p_el_c::ExtPElC;
pub use ext_p_el_c2d::ExtPElC2d;
pub use ext_p_el_s::ExtPElS;
pub use gp_space::{GpPoint, GpVector};
pub use point_curve::{
    ELPCOfLocateExtPC, ELPCOfLocateExtPC2d, EPCOfELPCOfLocateExtPC, EPCOfELPCOfLocateExtPC2d,
    EPCOfExtPC, EPCOfExtPC2d, ExtPC, ExtPC2d, GCurveLocator, GFuncExtPC, GGExtPC, GGenExtPC,
    GLocateExtPC, GenLocateExtPC, LocEPCOfLocateExtPC, LocEPCOfLocateExtPC2d, LocateExtPC,
    LocateExtPC2d, PCFOfEPCOfELPCOfLocateExtPC, PCFOfEPCOfELPCOfLocateExtPC2d, PCFOfEPCOfExtPC,
    PCFOfEPCOfExtPC2d, PCLocFOfLocEPCOfLocateExtPC, PCLocFOfLocEPCOfLocateExtPC2d,
};
pub use point_surface::{ExtPExtS, ExtPRevS, ExtPS, FuncPSDist, FuncPSNorm, GenExtPS, GenLocateExtPS, Sampling};
pub use surface_surface::{ExtSS, FuncExtSS, GenExtSS};
pub use pon::{POnCurv, POnCurv2d, POnCurve, POnSurf, POnSurfParams};

/// `Extrema_ElementType`: the grid element that holds a point on surface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ElementType {
    /// `Extrema_Node`: a grid node.
    #[default]
    Node,
    /// `Extrema_UIsoEdge`: an edge along U.
    UIsoEdge,
    /// `Extrema_VIsoEdge`: an edge along V.
    VIsoEdge,
    /// `Extrema_Face`: a grid cell.
    Face,
}

/// `Extrema_ExtAlgo`: the point-surface search algorithm.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ExtAlgo {
    /// `Extrema_ExtAlgo_Grad`: grid sampling with a gradient refinement.
    #[default]
    Grad,
    /// `Extrema_ExtAlgo_Tree`: a bounding-sphere tree over the grid.
    Tree,
}

/// `Extrema_ExtFlag`: which extrema a search returns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ExtFlag {
    /// `Extrema_ExtFlag_MIN`: the minima.
    Min,
    /// `Extrema_ExtFlag_MAX`: the maxima.
    Max,
    /// `Extrema_ExtFlag_MINMAX`: both.
    #[default]
    MinMax,
}
