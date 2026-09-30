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

//! OCCT package GeomFill: surfaces filled between boundary curves, skinned through
//! sections, and swept along paths.
//!
//! The package is ported in two parts, each class in its own file. The filling part
//! (docs/geomalgo-design.md section 13) holds the skinning classes (`GeomFill_Line`,
//! `GeomFill_Profiler`, `GeomFill_SectionGenerator`, `GeomFill_Generator`, the
//! `GeomFill_AppSurf` instance of the AppBlend template), the filling of two to four
//! curves (`GeomFill_BezierCurves`, `GeomFill_BSplineCurves` and the pole schemes
//! `GeomFill_Filling`, `GeomFill_Coons`, `GeomFill_Curved`, `GeomFill_Stretch`) and
//! the constrained filling of boundaries (`GeomFill_ConstrainedFilling` with its
//! boundaries, tangent fields, Coons patch and corner states). The sweep part holds
//! the trihedron, location and section laws and the sweeps.

// Filling part (docs/geomalgo-design.md section 13).
mod app_surf;
mod approx_style;
mod bezier_curves;
mod bound_parameter;
mod bound_with_surf;
mod boundary;
mod bspline_curves;
mod bspline_distribution;
mod constrained_filling;
mod coons;
mod coons_alg_patch;
mod corner_state;
mod curved;
mod curves_filling;
mod degenerated_bound;
mod filling;
mod filling_style;
mod generator;
mod line;
mod profiler;
mod profiler_unify;
mod section_generator;
mod simple_bound;
mod stretch;
mod tgt_field;
mod tgt_on_coons;

pub use app_surf::GeomFillAppSurf;
pub use approx_style::GeomFillApproxStyle;
pub use bezier_curves::GeomFillBezierCurves;
pub use bound_with_surf::GeomFillBoundWithSurf;
pub use boundary::{GeomFillBoundary, GeomFillBoundaryTolerances};
pub use bspline_curves::GeomFillBSplineCurves;
pub use constrained_filling::GeomFillConstrainedFilling;
pub use coons::GeomFillCoons;
pub use coons_alg_patch::GeomFillCoonsAlgPatch;
pub use corner_state::GeomFillCornerState;
pub use curved::GeomFillCurved;
pub use degenerated_bound::GeomFillDegeneratedBound;
pub use filling::GeomFillFilling;
pub use filling_style::GeomFillFillingStyle;
pub use generator::GeomFillGenerator;
pub use line::GeomFillLine;
pub use profiler::GeomFillProfiler;
pub use section_generator::GeomFillSectionGenerator;
pub use simple_bound::GeomFillSimpleBound;
pub use stretch::GeomFillStretch;
pub use tgt_field::GeomFillTgtField;
pub use tgt_on_coons::GeomFillTgtOnCoons;

// Sweep classes (docs/geomalgo-design.md, section 14).
mod arc_section;
mod circle_spans;
mod constant_bi_normal;
mod convertor_basis;
mod corrected_frenet;
mod corrected_frenet_init;
mod corrected_frenet_mode;
mod corrected_frenet_walk;
mod curve_and_trihedron;
mod darboux;
mod darboux_normal;
mod discrete_trihedron;
mod draft_trihedron;
mod evolved_section;
mod fixed;
mod frame_math;
mod frenet;
mod frenet_init;
mod frenet_singular;
mod function_draft;
mod function_guide;
mod geomfill;
mod geomfill_circle;
mod guide_plan_init;
mod guide_trihedron_ac;
mod guide_trihedron_ac_d2;
mod guide_trihedron_plan;
mod loc_function;
mod location_draft;
mod location_draft_eval;
mod location_guide;
mod location_guide_eval;
mod location_guide_rotation;
mod location_helpers;
mod location_law;
mod pipe_error;
mod plan_func;
mod polynomial_convertor;
mod quasi_angular_convertor;
mod quasi_angular_terms;
mod section_curve;
mod section_law;
mod section_placement;
mod section_placement_axis;
mod section_placement_frame;
mod section_placement_search;
mod snglr_func;
mod sweep_failures;
mod tensor;
mod trihedron_law;
mod trihedron_with_guide;
mod uniform_section;

pub use arc_section::{ArcPolesD1, ArcPolesD2, ArcSectionD0, ArcSectionD1, ArcSectionD2};
pub use constant_bi_normal::GeomFillConstantBiNormal;
pub use corrected_frenet::GeomFillCorrectedFrenet;
pub use curve_and_trihedron::GeomFillCurveAndTrihedron;
pub use darboux::GeomFillDarboux;
pub use discrete_trihedron::GeomFillDiscreteTrihedron;
pub use draft_trihedron::GeomFillDraftTrihedron;
pub use evolved_section::GeomFillEvolvedSection;
pub use fixed::GeomFillFixed;
pub use frenet::GeomFillFrenet;
pub use function_draft::GeomFillFunctionDraft;
pub use function_guide::GeomFillFunctionGuide;
pub use geomfill::{get_minimal_weights, get_shape, get_tolerance, knots, mults};
pub use geomfill_circle::{CircleD0, CircleD1, CircleD2, get_circle, get_circle_d1, get_circle_d2};
pub use guide_trihedron_ac::GeomFillGuideTrihedronAC;
pub use guide_trihedron_plan::GeomFillGuideTrihedronPlan;
pub use loc_function::GeomFillLocFunction;
pub use location_draft::GeomFillLocationDraft;
pub use location_guide::GeomFillLocationGuide;
pub use location_law::{GeomFillLocationLaw, LocationD0, LocationD1, LocationD2, LocationPoles2d};
pub use pipe_error::{GeomFillPipeError, GeomFillTrihedron};
pub use plan_func::GeomFillPlanFunc;
pub use polynomial_convertor::GeomFillPolynomialConvertor;
pub use quasi_angular_convertor::GeomFillQuasiAngularConvertor;
pub use section_law::{GeomFillSectionLaw, SectionD1, SectionD2};
pub use section_placement::GeomFillSectionPlacement;
pub use snglr_func::GeomFillSnglrFunc;
pub use tensor::GeomFillTensor;
pub use trihedron_law::{
    GeomFillTrihedronLaw, GeomFillTrihedronLawBase, TrihedronD0, TrihedronD1, TrihedronD2,
};
pub use trihedron_with_guide::{
    GeomFillTrihedronWithGuide, GeomFillTrihedronWithGuideBase, GuideFields,
};
pub use uniform_section::GeomFillUniformSection;
