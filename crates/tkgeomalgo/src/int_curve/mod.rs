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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntCurve at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package IntCurve: the intersection of 2D conics, and the generic
//! curve-curve intersection templates that Geom2dInt instantiates.
//!
//! [`IConicTool`], [`PConic`], [`PConicTool`] and [`ProjectOnPConicTool`] present a
//! gp conic to the IntImpParGen template; [`ConicTypes`] is that instantiation and
//! [`IntImpConicParConic`] and [`MyImpParToolOfIntImpConicParConic`] are its native
//! classes. [`IntConicConic`] intersects every pair of conics. The free functions and
//! classes of IntCurve_IntConicConic_Tool.hxx are [`determine_transition_lc`],
//! [`normalize_on_circle_domain`], [`Interval`] and [`PeriodicInterval`].
//!
//! The curve templates are generics that Geom2dInt instantiates
//! (docs/geomalgo-design.md): [`IntConicCurveGen`] (IntCurve_IntConicCurveGen.gxx),
//! [`UserIntConicCurveGen`] (IntCurve_UserIntConicCurveGen.gxx) and
//! [`DistBetweenPCurvesGen`] (IntCurve_DistBetweenPCurvesGen.gxx), over the curve tool
//! trait [`PCurveTool`].
mod conic_conic_ana;
mod conic_conic_bounds;
mod conic_conic_circle_pair;
mod conic_conic_circles;
mod conic_conic_line_circle;
mod conic_conic_line_conic;
mod conic_conic_line_tools;
mod conic_conic_lines;
mod conic_conic_lines_coincident;
mod conic_conic_lines_segment;
mod conic_conic_tool;
mod dist_between_pcurves_gen;
mod iconic_tool;
mod int_conic_conic;
mod int_conic_curve_gen;
mod pconic;
mod pcurve_tool;
mod project_on_pconic_tool;
mod user_int_conic_curve_gen;

pub use conic_conic_tool::{
    Interval, PI_P_PI, PeriodicInterval, determine_transition_lc, normalize_on_circle_domain,
};
pub use dist_between_pcurves_gen::DistBetweenPCurvesGen;
pub use iconic_tool::IConicTool;
pub use int_conic_conic::IntConicConic;
pub use int_conic_curve_gen::IntConicCurveGen;
pub use pconic::{PConic, PConicTool};
pub use pcurve_tool::{GpConic, PCurveTool};
pub use project_on_pconic_tool::ProjectOnPConicTool;
pub use user_int_conic_curve_gen::UserIntConicCurveGen;

use crate::int_imp_par_gen::{Intersector, IntersectorTypes, MyImpParTool};

/// The IntImpParGen instantiation of IntCurve_IntImpConicParConic_0.cxx: a gp conic
/// ([`IConicTool`]) against a parametric gp conic ([`PConic`]).
pub struct ConicTypes;

impl IntersectorTypes for ConicTypes {
    type Imp = IConicTool;
    type Par = PConicTool;
    type Project = ProjectOnPConicTool;
}

/// `IntCurve_IntImpConicParConic`: `IntImpParGen_Intersector` over [`ConicTypes`].
pub type IntImpConicParConic = Intersector<ConicTypes>;

/// `IntCurve_MyImpParToolOfIntImpConicParConic`: the function of [`IntImpConicParConic`].
pub type MyImpParToolOfIntImpConicParConic<'a> = MyImpParTool<'a, ConicTypes>;
