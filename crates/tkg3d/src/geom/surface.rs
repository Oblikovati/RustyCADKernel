// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG3d/Geom/Geom_Surface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_Surface`: parameterized surfaces of space.
use super::curve::GeomCurve;
use super::errors;
use super::geometry::{GeomGeometry, dump_geometry};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{GeneralTransform2, Point3, Transform3, Vector3};
use std::io;

/// Point and first partial derivatives (`Geom_Surface::ResD1`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GeomSurfaceD1 {
    /// Point of the surface.
    pub point: Point3,
    /// Derivative in U.
    pub d1u: Vector3,
    /// Derivative in V.
    pub d1v: Vector3,
}
/// Point and partial derivatives up to order 2 (`Geom_Surface::ResD2`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GeomSurfaceD2 {
    /// Point of the surface.
    pub point: Point3,
    /// Derivative in U.
    pub d1u: Vector3,
    /// Derivative in V.
    pub d1v: Vector3,
    /// Second derivative in U.
    pub d2u: Vector3,
    /// Second derivative in V.
    pub d2v: Vector3,
    /// Mixed second derivative.
    pub d2uv: Vector3,
}
/// Point and partial derivatives up to order 3 (`Geom_Surface::ResD3`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GeomSurfaceD3 {
    /// Point of the surface.
    pub point: Point3,
    /// Derivative in U.
    pub d1u: Vector3,
    /// Derivative in V.
    pub d1v: Vector3,
    /// Second derivative in U.
    pub d2u: Vector3,
    /// Second derivative in V.
    pub d2v: Vector3,
    /// Mixed second derivative.
    pub d2uv: Vector3,
    /// Third derivative in U.
    pub d3u: Vector3,
    /// Third derivative in V.
    pub d3v: Vector3,
    /// Third derivative, twice in U and once in V.
    pub d3uuv: Vector3,
    /// Third derivative, once in U and twice in V.
    pub d3uvv: Vector3,
}

/// Parametric bounds `(U1, U2, V1, V2)` (`Bounds`).
pub type SurfaceBounds = (f64, f64, f64, f64);

/// A parameterized surface: evaluation, bounds, closure, periodicity, continuity,
/// iso curves and orientation; `let point: Point3 = surface.value(0.5, 0.5)?;`.
pub trait GeomSurface: GeomGeometry {
    /// Reverse the U direction (`UReverse`); `surface.u_reverse()?`.
    fn u_reverse(&self) -> Result<(), Failure>;
    /// U parameter on the U-reversed surface (`UReversedParameter`); `surface.u_reversed_parameter(0.2)`.
    fn u_reversed_parameter(&self, u: f64) -> f64;
    /// Reverse the V direction (`VReverse`); `surface.v_reverse()?`.
    fn v_reverse(&self) -> Result<(), Failure>;
    /// V parameter on the V-reversed surface (`VReversedParameter`); `surface.v_reversed_parameter(0.2)`.
    fn v_reversed_parameter(&self, v: f64) -> f64;
    /// U-reversed copy (`UReversed`); `surface.u_reversed()?`.
    fn u_reversed(&self) -> Result<Handle<dyn GeomSurface>, Failure> {
        let copy: Handle<dyn GeomSurface> = self.copy()?.cast::<dyn GeomSurface>();
        copy.u_reverse()?;
        Ok(copy)
    }
    /// V-reversed copy (`VReversed`); `surface.v_reversed()?`.
    fn v_reversed(&self) -> Result<Handle<dyn GeomSurface>, Failure> {
        let copy: Handle<dyn GeomSurface> = self.copy()?.cast::<dyn GeomSurface>();
        copy.v_reverse()?;
        Ok(copy)
    }
    /// Parameters on the surface transformed by `transform`; unchanged by default
    /// (`TransformParameters`); `let (u, v) = surface.transform_parameters(u, v, t);`.
    fn transform_parameters(&self, u: f64, v: f64, _transform: Transform3) -> (f64, f64) {
        (u, v)
    }
    /// The parametric transformation matching `transform`; identity by default
    /// (`ParametricTransformation`). It is fallible because a product of gp_GTrsf2d
    /// (surface of linear extrusion) can raise natively;
    /// `surface.parametric_transformation(t)?`.
    fn parametric_transformation(
        &self,
        _transform: Transform3,
    ) -> Result<GeneralTransform2, Failure> {
        Ok(GeneralTransform2::default())
    }
    /// Parametric bounds `(U1, U2, V1, V2)`, infinite ones included (`Bounds`);
    /// `let (u1, u2, v1, v2) = surface.bounds();`.
    fn bounds(&self) -> SurfaceBounds;
    /// Whether closed in U (`IsUClosed`); `surface.is_u_closed()?`.
    fn is_u_closed(&self) -> Result<bool, Failure>;
    /// Whether closed in V (`IsVClosed`); `surface.is_v_closed()?`.
    fn is_v_closed(&self) -> Result<bool, Failure>;
    /// Whether periodic in U (`IsUPeriodic`); `surface.is_u_periodic()`.
    fn is_u_periodic(&self) -> bool;
    /// Whether periodic in V (`IsVPeriodic`); `surface.is_v_periodic()`.
    fn is_v_periodic(&self) -> bool;
    /// U period; Standard_NoSuchObject when not U-periodic (`UPeriod`); `surface.u_period()?`.
    fn u_period(&self) -> Result<f64, Failure> {
        if !self.is_u_periodic() {
            return Err(errors::no_such_object(
                "Geom_Surface::UPeriod: the surface is not U-periodic; expected IsUPeriodic()",
            ));
        }
        let (u1, u2, _, _) = self.bounds();
        Ok(u2 - u1)
    }
    /// V period; Standard_NoSuchObject when not V-periodic (`VPeriod`); `surface.v_period()?`.
    fn v_period(&self) -> Result<f64, Failure> {
        if !self.is_v_periodic() {
            return Err(errors::no_such_object(
                "Geom_Surface::VPeriod: the surface is not V-periodic; expected IsVPeriodic()",
            ));
        }
        let (_, _, v1, v2) = self.bounds();
        Ok(v2 - v1)
    }
    /// The U isoparametric curve (`UIso`); `surface.u_iso(0.5)?`.
    fn u_iso(&self, u: f64) -> Result<Handle<dyn GeomCurve>, Failure>;
    /// The V isoparametric curve (`VIso`); `surface.v_iso(0.5)?`.
    fn v_iso(&self, v: f64) -> Result<Handle<dyn GeomCurve>, Failure>;
    /// Global continuity (`Continuity`); `surface.continuity()`.
    fn continuity(&self) -> Shape;
    /// Whether the continuity in U is at least `n` (`IsCNu`); `surface.is_cn_u(2)?`.
    fn is_cn_u(&self, n: i32) -> Result<bool, Failure>;
    /// Whether the continuity in V is at least `n` (`IsCNv`); `surface.is_cn_v(2)?`.
    fn is_cn_v(&self, n: i32) -> Result<bool, Failure>;
    /// Point of parameters `(u, v)` (`EvalD0`); `surface.eval_d0(0.5, 0.5)?`.
    fn eval_d0(&self, u: f64, v: f64) -> Result<Point3, Failure>;
    /// Point and first derivatives (`EvalD1`); `surface.eval_d1(0.5, 0.5)?.d1u`.
    fn eval_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure>;
    /// Point and derivatives up to order 2 (`EvalD2`); `surface.eval_d2(0.5, 0.5)?.d2uv`.
    fn eval_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure>;
    /// Point and derivatives up to order 3 (`EvalD3`); `surface.eval_d3(0.5, 0.5)?.d3uuv`.
    fn eval_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure>;
    /// Derivative of orders `(nu, nv)`, `nu + nv >= 1` (`EvalDN`); `surface.eval_dn(0.5, 0.5, 1, 2)?`.
    fn eval_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure>;
    /// Point of parameters `(u, v)` (`D0`); `surface.d0(0.5, 0.5)?`.
    fn d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        self.eval_d0(u, v)
    }
    /// Point and first derivatives (`D1`); `surface.d1(0.5, 0.5)?`.
    fn d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        self.eval_d1(u, v)
    }
    /// Point and derivatives up to order 2 (`D2`); `surface.d2(0.5, 0.5)?`.
    fn d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        self.eval_d2(u, v)
    }
    /// Point and derivatives up to order 3 (`D3`); `surface.d3(0.5, 0.5)?`.
    fn d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        self.eval_d3(u, v)
    }
    /// Derivative of orders `(nu, nv)` (`DN`); `surface.dn(0.5, 0.5, 1, 1)?`.
    fn dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        self.eval_dn(u, v, nu, nv)
    }
    /// Point of parameters `(u, v)`, through EvalD0 (`Value`); `surface.value(0.5, 0.5)?`.
    fn value(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        self.eval_d0(u, v)
    }
}
impl_transient_interface!(dyn GeomSurface, "Geom_Surface", dyn GeomGeometry);

/// `Geom_Surface::DumpJson`; `dump_surface(&mut stream, -1)?`.
pub fn dump_surface(stream: &mut DumpStream, depth: i32) -> io::Result<()> {
    dump_class_name(stream, "Geom_Surface")?;
    dump_base(stream, depth, dump_geometry)
}

/// The native `Nu + Nv < 1` check of the EvalDN members (Geom_UndefinedDerivative);
/// `check_dn_orders("Geom_Plane", nu, nv)?`.
pub fn check_dn_orders(class: &str, nu: i32, nv: i32) -> Result<(), Failure> {
    if nu + nv < 1 || nu < 0 || nv < 0 {
        return Err(errors::undefined_derivative(&format!(
            "{class}::EvalDN: Nu = {nu}, Nv = {nv}; expected Nu >= 0, Nv >= 0 and Nu + Nv >= 1"
        )));
    }
    Ok(())
}
