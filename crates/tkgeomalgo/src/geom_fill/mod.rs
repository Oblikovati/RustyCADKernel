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

// Gordon surfaces (docs/geomalgo-design.md section 15): GeomFill_GordonUtilities.pxx,
// GeomFill_Gordon and GeomFill_NetworkSurface.
mod gordon;
mod gordon_contacts;
mod gordon_fallback;
mod gordon_network;
mod gordon_normalize;
mod gordon_order;
mod gordon_params;
mod gordon_perform;
mod gordon_rebuild;
mod gordon_reorder;
mod gordon_reparam;
mod network_surface;
mod network_surface_align;
mod network_surface_rational;
mod network_surface_seam;
mod network_surface_skin;

pub use gordon::{
    GeomFillGordon, GordonApproximationMode, GordonBuildReport, GordonBuildStage, GordonStatus,
};
pub use network_surface::{ClosedSeams, GeomFillNetworkSurface, NetworkSurfaceStatus};
