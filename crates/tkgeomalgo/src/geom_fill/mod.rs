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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package GeomFill. Not ported yet: its classes land in this module when
//! the package is ported (docs/tkgeomalgo-progress.md lists the ported packages).
//!
//! The sweep part (the trihedron, location and section laws, `GeomFill_Sweep`,
//! `GeomFill_Pipe` and their helpers) follows docs/geomalgo-design.md, section 14.

// Sweep classes (docs/geomalgo-design.md, section 14).
mod arc_section;
mod circle_spans;
mod constant_bi_normal;
mod convertor_basis;
mod corrected_frenet;
mod corrected_frenet_init;
mod corrected_frenet_mode;
mod corrected_frenet_walk;
mod darboux;
mod darboux_normal;
mod discrete_trihedron;
mod draft_trihedron;
mod fixed;
mod frame_math;
mod frenet;
mod frenet_init;
mod frenet_singular;
mod geomfill;
mod geomfill_circle;
mod guide_plan_init;
mod guide_trihedron_ac;
mod guide_trihedron_ac_d2;
mod guide_trihedron_plan;
mod location_law;
mod pipe_error;
mod plan_func;
mod polynomial_convertor;
mod quasi_angular_convertor;
mod quasi_angular_terms;
mod section_law;
mod snglr_func;
mod sweep_failures;
mod tensor;
mod trihedron_law;
mod trihedron_with_guide;

pub use arc_section::{ArcPolesD1, ArcPolesD2, ArcSectionD0, ArcSectionD1, ArcSectionD2};
pub use constant_bi_normal::GeomFillConstantBiNormal;
pub use corrected_frenet::GeomFillCorrectedFrenet;
pub use darboux::GeomFillDarboux;
pub use discrete_trihedron::GeomFillDiscreteTrihedron;
pub use draft_trihedron::GeomFillDraftTrihedron;
pub use fixed::GeomFillFixed;
pub use frenet::GeomFillFrenet;
pub use geomfill::{get_minimal_weights, get_shape, get_tolerance, knots, mults};
pub use geomfill_circle::{CircleD0, CircleD1, CircleD2, get_circle, get_circle_d1, get_circle_d2};
pub use guide_trihedron_ac::GeomFillGuideTrihedronAC;
pub use guide_trihedron_plan::GeomFillGuideTrihedronPlan;
pub use location_law::{GeomFillLocationLaw, LocationD0, LocationD1, LocationD2, LocationPoles2d};
pub use pipe_error::{GeomFillPipeError, GeomFillTrihedron};
pub use plan_func::GeomFillPlanFunc;
pub use polynomial_convertor::GeomFillPolynomialConvertor;
pub use quasi_angular_convertor::GeomFillQuasiAngularConvertor;
pub use section_law::{GeomFillSectionLaw, SectionD1, SectionD2};
pub use snglr_func::GeomFillSnglrFunc;
pub use tensor::GeomFillTensor;
pub use trihedron_law::{
    GeomFillTrihedronLaw, GeomFillTrihedronLawBase, TrihedronD0, TrihedronD1, TrihedronD2,
};
pub use trihedron_with_guide::{
    GeomFillTrihedronWithGuide, GeomFillTrihedronWithGuideBase, GuideFields,
};
