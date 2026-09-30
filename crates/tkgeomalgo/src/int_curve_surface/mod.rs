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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntCurveSurface at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package IntCurveSurface: the intersection of a curve with a surface.
//!
//! [`Intersection`] holds the [`IntersectionPoint`]s and [`IntersectionSegment`]s of
//! an intersection; [`HInter`] (`IntCurveSurface_HInter`) computes them for
//! Adaptor3d curves and surfaces, analytically for conics against quadrics, else
//! through the interference of a polygon of the curve with a polyhedron of the
//! surface refined by the exact intersector. The header templates of the package
//! are generics (docs/geomalgo-design.md section 11): [`InterGen`] over
//! [`InterTypes`], the polygon [`InterPolygon`] over an [`InterCurveTool`], the
//! utilities in [`inter_utils`], [`polygon_utils`] and [`polyhedron_utils`]. The
//! native instance classes are aliases (`ThePolygonOfHInter`,
//! `TheInterferenceOfHInter`, `TheCSFunctionOfHInter`, `TheExactHInter`...).
mod dump;
mod failures;
mod h_curve_tool;
mod inter;
pub mod inter_utils;
mod intersection;
mod intersection_point;
mod intersection_segment;
mod polygon;
pub mod polygon_utils;
mod polyhedron;
mod polyhedron_tool;
pub mod polyhedron_utils;
mod quad_curv;
mod sample_pars;
mod tools;
mod tri_connex;

pub use h_curve_tool::{HCurve, TheHCurveTool};
pub use inter::{InterGen, InterTypes, SamplePolygon, SamplePolyhedron};
pub use intersection::Intersection;
pub use intersection_point::{IntersectionPoint, TransitionOnCurve};
pub use intersection_segment::IntersectionSegment;
pub use polygon::{InterPolygon, InterPolygonTool};
pub use polyhedron::ThePolyhedronOfHInter;
pub use polyhedron_tool::ThePolyhedronToolOfHInter;
pub use quad_curv::{QuadCurvExact, QuadCurvFunc};
pub use tools::{InterCurveTool, InterSurfaceTool};
pub use tri_connex::tri_connex;

use crate::int_imp::{HSurfaceTool, IntImpIntCS, IntImpZerCSParFunc};
use crate::intf::InterferencePolygonPolyhedron;
use inter_utils::{SectionPolygon, SectionPolyhedron};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point3;

/// `IntCurveSurface_ThePolygonOfHInter`: the polygon of an Adaptor3d curve.
pub type ThePolygonOfHInter = InterPolygon<TheHCurveTool>;
/// `IntCurveSurface_ThePolygonToolOfHInter`.
pub type ThePolygonToolOfHInter = InterPolygonTool<TheHCurveTool>;
/// `IntCurveSurface_TheInterferenceOfHInter`: Intf_InterferencePolygonPolyhedron.gxx
/// on the polygon and polyhedron of IntCurveSurface_HInter.
pub type TheInterferenceOfHInter =
    InterferencePolygonPolyhedron<ThePolygonToolOfHInter, ThePolyhedronToolOfHInter>;
/// `IntCurveSurface_TheCSFunctionOfHInter`: IntImp_ZerCSParFunc.gxx on
/// Adaptor3d_HSurfaceTool and IntCurveSurface_TheHCurveTool.
pub type TheCSFunctionOfHInter = IntImpZerCSParFunc<HSurfaceTool, TheHCurveTool>;
/// `IntCurveSurface_TheExactHInter`: IntImp_IntCS.gxx on the CS function.
pub type TheExactHInter = IntImpIntCS<TheCSFunctionOfHInter>;
/// `IntCurveSurface_TheQuadCurvFuncOfTheQuadCurvExactHInter`.
pub type TheQuadCurvFuncOfTheQuadCurvExactHInter = QuadCurvFunc<TheHCurveTool>;
/// `IntCurveSurface_TheQuadCurvExactHInter`.
pub type TheQuadCurvExactHInter = QuadCurvExact<HSurfaceTool, TheHCurveTool>;
/// `IntCurveSurface_HInter`: the curve-surface intersection of Adaptor3d handles.
pub type HInter = InterGen<HInterTypes>;

/// The types of `IntCurveSurface_HInter`: the Adaptor3d tools, its polygon and
/// polyhedron with their Intf tools.
pub struct HInterTypes;

impl InterTypes for HInterTypes {
    type CurveTool = TheHCurveTool;
    type SurfaceTool = HSurfaceTool;
    type Polygon = ThePolygonOfHInter;
    type Polyhedron = ThePolyhedronOfHInter;
    type PolygonTool = ThePolygonToolOfHInter;
    type PolyhedronTool = ThePolyhedronToolOfHInter;
}

impl<C: InterCurveTool> SectionPolygon for InterPolygon<C> {
    fn approx_param_on_curve(&self, index: i32, param_on_line: f64) -> Result<f64, Failure> {
        InterPolygon::approx_param_on_curve(self, index, param_on_line)
    }
}

impl<C: InterCurveTool> SamplePolygon<C> for InterPolygon<C> {
    fn with_samples(curve: &C::Curve, nb_pnt: i32) -> Result<Self, Failure> {
        InterPolygon::new(curve, nb_pnt)
    }
    fn with_range(curve: &C::Curve, u1: f64, u2: f64, nb_pnt: i32) -> Result<Self, Failure> {
        InterPolygon::with_range(curve, u1, u2, nb_pnt)
    }
    fn with_parameters(curve: &C::Curve, upars: &Array1<'_, f64>) -> Result<Self, Failure> {
        InterPolygon::with_parameters(curve, upars)
    }
    fn inf_parameter(&self) -> f64 {
        InterPolygon::inf_parameter(self)
    }
    fn sup_parameter(&self) -> f64 {
        InterPolygon::sup_parameter(self)
    }
}

impl SectionPolyhedron for ThePolyhedronOfHInter {
    fn triangle(&self, index: i32) -> Result<[i32; 3], Failure> {
        Ok(ThePolyhedronOfHInter::triangle(self, index))
    }
    fn point(&self, index: i32) -> Result<Point3, Failure> {
        ThePolyhedronOfHInter::point(self, index)
    }
    fn parameters(&self, index: i32) -> Result<(f64, f64), Failure> {
        ThePolyhedronOfHInter::parameters(self, index)
    }
}

impl SamplePolyhedron<HSurfaceTool> for ThePolyhedronOfHInter {
    fn uniform(
        surface: &Adaptor3dSurfaceHandle,
        nbd_u: i32,
        nbd_v: i32,
        [u1, v1, u2, v2]: [f64; 4],
    ) -> Result<Self, Failure> {
        ThePolyhedronOfHInter::new(surface, nbd_u, nbd_v, u1, v1, u2, v2)
    }
    fn with_parameters(
        surface: &Adaptor3dSurfaceHandle,
        upars: &Array1<'_, f64>,
        vpars: &Array1<'_, f64>,
    ) -> Result<Self, Failure> {
        ThePolyhedronOfHInter::with_parameters(surface, upars, vpars)
    }
    fn bounding(&self) -> &BoundingBox3 {
        ThePolyhedronOfHInter::bounding(self)
    }
}
