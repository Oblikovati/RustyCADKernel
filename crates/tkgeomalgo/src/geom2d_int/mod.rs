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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/Geom2dInt at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package Geom2dInt: the instantiation of the IntCurve and IntImpParGen
//! templates on `Adaptor2d_Curve2d`.
//!
//! [`Geom2dCurveTool`] is the curve tool and [`TheProjPCurOfGInter`] the projection
//! of a point; [`Geom2dTypes`] is the IntImpParGen instantiation. Every other native
//! class is an alias of the template it instantiates (docs/geomalgo-design.md).
//! The Extrema aliases reuse the instantiations of TKGeomBase over
//! `Extrema_Curve2dTool`: the members the Extrema templates call (Value, D0-D2, DN,
//! GetType and the parameter bounds) forward to the adaptor in both tools.
mod geom2d_curve_tool;
mod the_proj_pcur;

pub use geom2d_curve_tool::{Curve2d, Geom2dCurveTool};
pub use the_proj_pcur::TheProjPCurOfGInter;

use crate::int_curve::{
    DistBetweenPCurvesGen, ExactIntersectionPoint, IConicTool, IntConicCurveGen, IntCurveCurveGen,
    IntPolyPolyGen, Polygon2dGen, UserIntConicCurveGen,
};
use crate::int_imp_par_gen::{Intersector, IntersectorTypes, MyImpParTool};
use rustycad_tkgeombase::extrema::{Curve2dTool, GCurveLocator, GFuncExtPC, GenLocateExtPC};

/// The IntImpParGen instantiation of Geom2dInt: a gp conic ([`IConicTool`]) against
/// an `Adaptor2d_Curve2d` ([`Geom2dCurveTool`], [`TheProjPCurOfGInter`]).
pub struct Geom2dTypes;

impl IntersectorTypes for Geom2dTypes {
    type Imp = IConicTool;
    type Par = Geom2dCurveTool;
    type Project = TheProjPCurOfGInter;
}

/// `Geom2dInt_TheIntersectorOfTheIntConicCurveOfGInter`.
pub type TheIntersectorOfTheIntConicCurveOfGInter = Intersector<Geom2dTypes>;
/// `Geom2dInt_MyImpParToolOfTheIntersectorOfTheIntConicCurveOfGInter`.
pub type MyImpParToolOfTheIntersectorOfTheIntConicCurveOfGInter<'a> = MyImpParTool<'a, Geom2dTypes>;
/// `Geom2dInt_TheIntConicCurveOfGInter`: IntCurve_IntConicCurveGen over [`Geom2dTypes`].
pub type TheIntConicCurveOfGInter = IntConicCurveGen<Geom2dTypes>;
/// `Geom2dInt_IntConicCurveOfGInter`: IntCurve_UserIntConicCurveGen over [`Geom2dTypes`].
pub type IntConicCurveOfGInter = UserIntConicCurveGen<Geom2dTypes>;
/// `Geom2dInt_TheDistBetweenPCurvesOfTheIntPCurvePCurveOfGInter`.
pub type TheDistBetweenPCurvesOfTheIntPCurvePCurveOfGInter<'a> =
    DistBetweenPCurvesGen<'a, Geom2dCurveTool>;
/// `Geom2dInt_TheCurveLocatorOfTheProjPCurOfGInter`: Extrema_GCurveLocator.
pub type TheCurveLocatorOfTheProjPCurOfGInter = GCurveLocator<Curve2dTool>;
/// `Geom2dInt_TheLocateExtPCOfTheProjPCurOfGInter`: Extrema_GenLocateExtPC.
pub type TheLocateExtPCOfTheProjPCurOfGInter<'a> = GenLocateExtPC<'a, Curve2dTool>;
/// `Geom2dInt_PCLocFOfTheLocateExtPCOfTheProjPCurOfGInter`: Extrema_GFuncExtPC.
pub type PCLocFOfTheLocateExtPCOfTheProjPCurOfGInter<'a> = GFuncExtPC<'a, Curve2dTool>;
/// `Geom2dInt_ThePolygon2dOfTheIntPCurvePCurveOfGInter`: IntCurve_Polygon2dGen over
/// [`Geom2dCurveTool`].
pub type ThePolygon2dOfTheIntPCurvePCurveOfGInter = Polygon2dGen<Geom2dCurveTool>;
/// `Geom2dInt_ExactIntersectionPointOfTheIntPCurvePCurveOfGInter`:
/// IntCurve_ExactIntersectionPoint over [`Geom2dCurveTool`].
pub type ExactIntersectionPointOfTheIntPCurvePCurveOfGInter<'a> =
    ExactIntersectionPoint<'a, Geom2dCurveTool>;
/// `Geom2dInt_TheIntPCurvePCurveOfGInter`: IntCurve_IntPolyPolyGen over [`Geom2dTypes`].
pub type TheIntPCurvePCurveOfGInter = IntPolyPolyGen<Geom2dTypes>;
/// `Geom2dInt_GInter`: IntCurve_IntCurveCurveGen over [`Geom2dTypes`].
pub type GInter = IntCurveCurveGen<Geom2dTypes>;
