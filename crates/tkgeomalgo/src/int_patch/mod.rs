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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package IntPatch: the surface-surface intersection (docs/geomalgo-design.md
//! section 16).
//!
//! This module holds the core of the package: the intersection points and lines
//! ([`IntPatchPoint`], the line hierarchy [`IntPatchLine`], [`IntPatchPointLine`],
//! [`IntPatchWLine`], [`IntPatchRLine`], [`IntPatchGLine`], [`IntPatchALine`]), the tools
//! and functions, the IntPatch instances of the IntImp, IntStart and IntWalk templates,
//! and the polyhedra, their BVH interference and the 2d polygons. The intersection
//! algorithms land on top of it.
mod aline;
mod aline_compute;
mod aline_to_wline;
mod arc_function;
mod bvh_traversal;
mod cs_function;
mod dump;
pub(crate) mod failures;
mod gline;
mod gline_compute;
mod h_curve2d_tool;
mod h_inter_tool;
mod instances;
mod imp_imp;
mod imp_prm;
mod interference_polyhedron;
mod iwline;
mod line;
mod path_point;
mod point;
mod point_dump;
mod point_line;
mod point_line_dump;
mod polyarc;
mod polygo;
mod polyhedron;
mod polyhedron_bvh;
mod polyline;
mod rline;
mod rline_compute;
mod rst_int;
mod special_points;
mod t3bits;
mod vertices;
mod wline;

pub use aline::IntPatchALine;
pub use aline_to_wline::IntPatchALineToWLine;
pub use arc_function::IntPatchArcFunction;
pub use bvh_traversal::{IntPatchBVHTraversal, IntPatchTrianglePair};
pub use cs_function::IntPatchCSFunction;
pub use gline::IntPatchGLine;
pub use h_curve2d_tool::IntPatchHCurve2dTool;
pub use h_inter_tool::IntPatchHInterTool;
pub use instances::{
    IntPatchCurvIntSurf, IntPatchTheIWalking, IntPatchTheIWalkingTypes, IntPatchTheSOnBounds,
    IntPatchTheSOnBoundsTypes, IntPatchTheSearchInside, IntPatchTheSearchInsideTypes,
    IntPatchTheSurfFunction,
};
pub use imp_imp::{IntPatchImpImpIntersection, IntPatchImpImpStatus};
pub use imp_prm::IntPatchImpPrmIntersection;
pub use interference_polyhedron::IntPatchInterferencePolyhedron;
pub use iwline::IntPatchTheIWLineOfTheIWalking;
pub use line::{IntPatchLine, IntPatchLineBase, IntPatchLineTransitions};
pub use path_point::{IntPatchThePathPointOfTheSOnBounds, IntPatchTheSegmentOfTheSOnBounds};
pub use point::IntPatchPoint;
pub use point_line::{IntPatchPointLine, curvature_radius_of_inters_line};
pub use polyarc::IntPatchPolyArc;
pub use polygo::{IntPatchPolygo, IntPatchPolygoBase};
pub use polyhedron::IntPatchPolyhedron;
pub use polyhedron_bvh::{IntPatchPolyhedronBVH, IntPatchPolyhedronTool};
pub use polyline::IntPatchPolyLine;
pub use rline::IntPatchRLine;
pub use rst_int::IntPatchRstInt;
pub use special_points::IntPatchSpecialPoints;
pub use t3bits::IntPatchPrmPrmIntersectionT3Bits;
pub use wline::{IntPatchWLType, IntPatchWLine};

/// `IntPatch_SearchPnt`: `Intf_InterferencePolygon2d`.
pub type IntPatchSearchPnt = crate::intf::InterferencePolygon2d;

/// `IntPatch_IType`: the geometry of an intersection line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum IntPatchIType {
    /// `IntPatch_Lin`: a straight line.
    #[default]
    Lin,
    /// `IntPatch_Circle`.
    Circle,
    /// `IntPatch_Ellipse`.
    Ellipse,
    /// `IntPatch_Parabola`.
    Parabola,
    /// `IntPatch_Hyperbola`.
    Hyperbola,
    /// `IntPatch_Analytic`: an analytic curve (`IntAna_Curve`).
    Analytic,
    /// `IntPatch_Walking`: the points of a walking.
    Walking,
    /// `IntPatch_Restriction`: a restriction of a surface.
    Restriction,
}

/// `IntPatch_SpecPntType`: the kinds of special (singular) points of the surface-surface
/// intersection (a pole of a sphere, the apex of a cone, a point on a seam).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum IntPatchSpecPntType {
    /// `IntPatch_SPntNone`.
    #[default]
    SPntNone,
    /// `IntPatch_SPntSeamU`.
    SPntSeamU,
    /// `IntPatch_SPntSeamV`.
    SPntSeamV,
    /// `IntPatch_SPntSeamUV`.
    SPntSeamUV,
    /// `IntPatch_SPntPoleSeamU`.
    SPntPoleSeamU,
    /// `IntPatch_SPntPole`.
    SPntPole,
}
