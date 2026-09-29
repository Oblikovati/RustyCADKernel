// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/Approx (package overview)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Approx: the approximation drivers of TKGeomBase — the multi-line drivers over
//! AppParCurves ([`ComputeLine`], [`BSplComputeLine`], [`ComputeCLine`] and its
//! instantiations [`FitAndDivide`], [`FitAndDivide2d`]), the join of Bezier
//! MultiCurves ([`MCurvesToBSpCurve`]), and the AdvApprox-based approximations of
//! curves, curves on surfaces, curvilinear parametrizations, same-parameter pcurves
//! and sweeps.
//!
//! The native `.gxx` generics take a `MultiLine` and its `LineTool`; the port takes
//! any type implementing [`crate::app_par_curves::LineTool`] (the AppDef, GeomInt and
//! BRepApprox instantiations are the same structs over their own lines).
mod bspl_compute_line;
mod check_multi_curve;
mod compute_c_line;
mod compute_line;
mod mcurves_to_bsp_curve;
mod status;
mod tangency;

pub use bspl_compute_line::BSplComputeLine;
pub use compute_c_line::{ComputeCLine, ComputeCLineOptions, FitAndDivide, FitAndDivide2d};
pub use compute_line::{ComputeLine, ComputeLineOptions};
pub use mcurves_to_bsp_curve::MCurvesToBSpCurve;
pub use status::{ParametrizationType, Status};
