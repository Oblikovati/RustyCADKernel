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
// Rust translation of the tool macros of ModelingAlgorithms/TKGeomAlgo/IntImp/*.gxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The type parameters of the IntImp templates: the operations each `.gxx` calls on
//! `ThePSurfaceTool`, `TheISurfaceTool` and `TheCurveTool`, under their native names.
//!
//! Every TKGeomAlgo instance uses `Adaptor3d_HSurfaceTool` for the parametric
//! surface and `IntSurf_QuadricTool` for the implicit one, so both implementations
//! live here once ([`HSurfaceTool`], and `IntSurfQuadricTool` from IntSurf).
use crate::int_surf::{IntSurfQuadric, IntSurfQuadricTool};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::{self, Adaptor3dSurfaceHandle};
use rustycad_tkg3d::geom::GeomSurfaceD1;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkmath::math::FunctionSetWithDerivatives;

/// `ThePSurfaceTool`: the parametric patch of the IntImp templates.
///
/// `T::value(&surface, 0.5, 0.5)?` evaluates the patch of a tool `T`.
pub trait ThePSurfaceTool {
    /// `ThePSurface`, held by value (a handle for the native instances).
    type Surface: Clone;
    /// `FirstUParameter(S)`.
    fn first_u_parameter(surface: &Self::Surface) -> Result<f64, Failure>;
    /// `FirstVParameter(S)`.
    fn first_v_parameter(surface: &Self::Surface) -> Result<f64, Failure>;
    /// `LastUParameter(S)`.
    fn last_u_parameter(surface: &Self::Surface) -> Result<f64, Failure>;
    /// `LastVParameter(S)`.
    fn last_v_parameter(surface: &Self::Surface) -> Result<f64, Failure>;
    /// `UResolution(S, R3d)`.
    fn u_resolution(surface: &Self::Surface, r3d: f64) -> Result<f64, Failure>;
    /// `VResolution(S, R3d)`.
    fn v_resolution(surface: &Self::Surface, r3d: f64) -> Result<f64, Failure>;
    /// `Value(S, U, V)`.
    fn value(surface: &Self::Surface, u: f64, v: f64) -> Result<Point3, Failure>;
    /// `D1(S, U, V, P, D1U, D1V)`.
    fn d1(surface: &Self::Surface, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure>;
}

/// `TheISurfaceTool`: the implicit surface of `IntImp_ZerImpFunc`.
pub trait TheISurfaceTool {
    /// `TheISurface`, held by value.
    type Surface: Clone;
    /// `Value(S, X, Y, Z)`.
    fn value(surface: &Self::Surface, x: f64, y: f64, z: f64) -> f64;
    /// `Gradient(S, X, Y, Z, V)`.
    fn gradient(surface: &Self::Surface, x: f64, y: f64, z: f64) -> Vector3;
    /// `ValueAndGradient(S, X, Y, Z, Val, Grad)`; the outputs may be left unchanged,
    /// as natively.
    fn value_and_gradient(
        surface: &Self::Surface,
        xyz: [f64; 3],
        value: &mut f64,
        gradient: &mut Vector3,
    );
}

/// `TheCurveTool`: the curve bounds and resolution of `IntImp_IntCS`.
pub trait TheCurveTool {
    /// `TheCurve`, held by value.
    type Curve: Clone;
    /// `FirstParameter(C)`.
    fn first_parameter(curve: &Self::Curve) -> Result<f64, Failure>;
    /// `LastParameter(C)`.
    fn last_parameter(curve: &Self::Curve) -> Result<f64, Failure>;
    /// `Resolution(C, R3d)`.
    fn resolution(curve: &Self::Curve, r3d: f64) -> Result<f64, Failure>;
}

/// The 3d evaluations of `TheCurveTool` that `IntImp_ZerCSParFunc` calls. `IntImp_IntCS`
/// needs only [`TheCurveTool`], so the 2d curve tool of `IntPatch_CurvIntSurf`
/// (`IntPatch_HCurve2dTool`, whose `Value` is a 2d point) implements only that trait
/// (docs/geomalgo-design.md section 16).
pub trait TheCurve3dTool: TheCurveTool {
    /// `Value(C, W)`.
    fn value(curve: &Self::Curve, w: f64) -> Result<Point3, Failure>;
    /// `D1(C, W, P, V)`.
    fn d1(curve: &Self::Curve, w: f64) -> Result<(Point3, Vector3), Failure>;
}

/// `TheFunction`: the curve-surface distance function `IntImp_IntCS` solves
/// (`IntImp_ZerCSParFunc` instances, or `IntPatch_CSFunction`).
pub trait TheFunction: FunctionSetWithDerivatives + Clone {
    /// `ThePSurfaceTool` of the surface.
    type SurfaceTool: ThePSurfaceTool;
    /// `TheCurveTool` of the curve.
    type CurveTool: TheCurveTool;
    /// `AuxillarSurface()`.
    fn auxillar_surface(&self) -> &<Self::SurfaceTool as ThePSurfaceTool>::Surface;
    /// `AuxillarCurve()`.
    fn auxillar_curve(&self) -> &<Self::CurveTool as TheCurveTool>::Curve;
    /// `Point()`: the point of the last evaluation.
    fn point(&self) -> Point3;
    /// `Root()`: the squared distance of the last evaluation.
    fn root(&self) -> f64;
    /// The first failure a solver callback raised since the last call (the native
    /// exception); functions that cannot fail keep the default.
    fn take_failure(&mut self) -> Result<(), Failure> {
        Ok(())
    }
}

/// `Adaptor3d_HSurfaceTool` as the `ThePSurfaceTool` of the TKGeomAlgo instances;
/// `HSurfaceTool::value(&adaptor, 0.5, 0.5)?`.
pub struct HSurfaceTool;

impl ThePSurfaceTool for HSurfaceTool {
    type Surface = Adaptor3dSurfaceHandle;
    fn first_u_parameter(surface: &Self::Surface) -> Result<f64, Failure> {
        h_surface_tool::first_u_parameter(surface)
    }
    fn first_v_parameter(surface: &Self::Surface) -> Result<f64, Failure> {
        h_surface_tool::first_v_parameter(surface)
    }
    fn last_u_parameter(surface: &Self::Surface) -> Result<f64, Failure> {
        h_surface_tool::last_u_parameter(surface)
    }
    fn last_v_parameter(surface: &Self::Surface) -> Result<f64, Failure> {
        h_surface_tool::last_v_parameter(surface)
    }
    fn u_resolution(surface: &Self::Surface, r3d: f64) -> Result<f64, Failure> {
        h_surface_tool::u_resolution(surface, r3d)
    }
    fn v_resolution(surface: &Self::Surface, r3d: f64) -> Result<f64, Failure> {
        h_surface_tool::v_resolution(surface, r3d)
    }
    fn value(surface: &Self::Surface, u: f64, v: f64) -> Result<Point3, Failure> {
        h_surface_tool::value(surface, u, v)
    }
    fn d1(surface: &Self::Surface, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        h_surface_tool::d1(surface, u, v)
    }
}

impl TheISurfaceTool for IntSurfQuadricTool {
    type Surface = IntSurfQuadric;
    fn value(surface: &Self::Surface, x: f64, y: f64, z: f64) -> f64 {
        IntSurfQuadricTool::value(surface, x, y, z)
    }
    fn gradient(surface: &Self::Surface, x: f64, y: f64, z: f64) -> Vector3 {
        IntSurfQuadricTool::gradient(surface, x, y, z)
    }
    fn value_and_gradient(
        surface: &Self::Surface,
        [x, y, z]: [f64; 3],
        value: &mut f64,
        gradient: &mut Vector3,
    ) {
        if let Some((v, g)) = IntSurfQuadricTool::value_and_gradient(surface, x, y, z) {
            (*value, *gradient) = (v, g);
        }
    }
}
