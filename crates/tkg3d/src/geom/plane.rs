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
// Rust translation of ModelingData/TKG3d/Geom/Geom_Plane.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_Plane`: an infinite plane parameterized along the X and Y directions of
//! its local frame.
use super::curve::GeomCurve;
use super::elementary_surface::{
    GeomElementarySurface, dump_elementary_surface, impl_elementary_position,
};
use super::geometry::GeomGeometry;
use super::line::GeomLine;
use super::parametric::scaled_parameter;
use super::surface::{
    GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3, SurfaceBounds, check_dn_orders,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name};
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{
    Axis3, Direction3, GeneralTransform2, Plane3, Point3, Transform2, Transform3, Vector3, origin2d,
};
use std::io;

#[derive(Clone, Copy)]
struct PlaneState {
    pos: Axis3,
}

/// A plane; `Handle::new(GeomPlane::new(frame))`.
pub struct GeomPlane {
    parent: StandardTransient,
    state: SharedState<PlaneState>,
}
impl_transient!(
    GeomPlane,
    "Geom_Plane",
    dyn GeomElementarySurface,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomSurface, dyn GeomElementarySurface]
);
impl Clone for GeomPlane {
    fn clone(&self) -> Self {
        Self::new(self.state.read().pos)
    }
}

fn zero() -> Vector3 {
    Vector3::new(0.0, 0.0, 0.0)
}

impl GeomPlane {
    /// Plane on a frame; its normal is the main direction (`Geom_Plane(gp_Ax3)`);
    /// `GeomPlane::new(frame)`.
    pub fn new(frame: Axis3) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(PlaneState { pos: frame }),
        }
    }
    /// Plane from a gp plane (`Geom_Plane(gp_Pln)`); `GeomPlane::from_pln(plane)`.
    pub fn from_pln(plane: Plane3) -> Self {
        Self::new(plane.position())
    }
    /// Plane through `point` normal to `normal` (`Geom_Plane(P, V)`);
    /// `GeomPlane::from_point(p, n)?`.
    pub fn from_point(point: Point3, normal: Direction3) -> Result<Self, Failure> {
        Ok(Self::from_pln(Plane3::new(point, normal)?))
    }
    /// Plane `A*X + B*Y + C*Z + D = 0`; Standard_ConstructionError when the normal
    /// `(A, B, C)` is null (`Geom_Plane(A, B, C, D)`);
    /// `GeomPlane::from_coefficients(0.0, 0.0, 1.0, -2.0)?`.
    pub fn from_coefficients(a: f64, b: f64, c: f64, d: f64) -> Result<Self, Failure> {
        Ok(Self::from_pln(Plane3::from_equation(a, b, c, d)?))
    }
    /// Replace by a gp plane (`SetPln`); `plane.set_pln(pln)`.
    pub fn set_pln(&self, plane: Plane3) {
        self.state.write().pos = plane.position();
    }
    /// The gp plane (`Pln`); `plane.pln()`.
    pub fn pln(&self) -> Plane3 {
        Plane3::from_position(self.state.read().pos)
    }
    /// The equation coefficients `(A, B, C, D)` in the absolute frame (`Coefficients`);
    /// `let [a, b, c, d] = plane.coefficients()?;`.
    pub fn coefficients(&self) -> Result<[f64; 4], Failure> {
        self.pln().coefficients()
    }
    fn pos(&self) -> Axis3 {
        self.state.read().pos
    }
}
impl GeomGeometry for GeomPlane {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        self.state.write().pos.transform(transform);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(self.clone()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let pos: Axis3 = self.pos();
        dump_class_name(stream, "Geom_Plane")?;
        dump_base(stream, depth, |s, d| dump_elementary_surface(s, d, pos))
    }
}
impl GeomSurface for GeomPlane {
    /// Reverses the X direction of the frame, unlike the other elementary surfaces.
    fn u_reverse(&self) -> Result<(), Failure> {
        self.state.write().pos.x_reverse();
        Ok(())
    }
    fn u_reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        Ok(-u)
    }
    /// Reverses the Y direction of the frame, unlike the other elementary surfaces.
    fn v_reverse(&self) -> Result<(), Failure> {
        self.state.write().pos.y_reverse();
        Ok(())
    }
    fn v_reversed_parameter(&self, v: f64) -> Result<f64, Failure> {
        Ok(-v)
    }
    fn transform_parameters(&self, u: f64, v: f64, transform: Transform3) -> (f64, f64) {
        (
            scaled_parameter(u, transform),
            scaled_parameter(v, transform),
        )
    }
    fn parametric_transformation(
        &self,
        transform: Transform3,
    ) -> Result<GeneralTransform2, Failure> {
        let mut t2 = Transform2::default();
        t2.set_scale(origin2d(), transform.scale_factor().abs());
        Ok(GeneralTransform2::from(t2))
    }
    fn bounds(&self) -> SurfaceBounds {
        let infinite: f64 = precision::infinite();
        (-infinite, infinite, -infinite, infinite)
    }
    fn is_u_closed(&self) -> Result<bool, Failure> {
        Ok(false)
    }
    fn is_v_closed(&self) -> Result<bool, Failure> {
        Ok(false)
    }
    fn is_u_periodic(&self) -> bool {
        false
    }
    fn is_v_periodic(&self) -> bool {
        false
    }
    fn u_iso(&self, u: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        let line = GeomLine::from_lin(elslib::plane_u_iso(self.pos(), u));
        Ok(Handle::new(line).cast())
    }
    fn v_iso(&self, v: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        let line = GeomLine::from_lin(elslib::plane_v_iso(self.pos(), v));
        Ok(Handle::new(line).cast())
    }
    fn continuity(&self) -> Shape {
        Shape::CN
    }
    fn is_cn_u(&self, _n: i32) -> Result<bool, Failure> {
        Ok(true)
    }
    fn is_cn_v(&self, _n: i32) -> Result<bool, Failure> {
        Ok(true)
    }
    fn eval_d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        Ok(elslib::plane_value(u, v, self.pos()))
    }
    fn eval_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        let (point, d1u, d1v) = elslib::plane_d1(u, v, self.pos());
        Ok(GeomSurfaceD1 { point, d1u, d1v })
    }
    fn eval_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        let (point, d1u, d1v) = elslib::plane_d1(u, v, self.pos());
        let (d2u, d2v, d2uv) = (zero(), zero(), zero());
        Ok(GeomSurfaceD2 {
            point,
            d1u,
            d1v,
            d2u,
            d2v,
            d2uv,
        })
    }
    fn eval_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        let (point, d1u, d1v) = elslib::plane_d1(u, v, self.pos());
        Ok(GeomSurfaceD3 {
            point,
            d1u,
            d1v,
            ..GeomSurfaceD3::default()
        })
    }
    fn eval_dn(&self, _u: f64, _v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        check_dn_orders("Geom_Plane", nu, nv)?;
        let pos: Axis3 = self.pos();
        Ok(match (nu, nv) {
            (0, 1) => Vector3::from(pos.y_direction()),
            (1, 0) => Vector3::from(pos.x_direction()),
            _ => zero(),
        })
    }
}
impl GeomElementarySurface for GeomPlane {
    impl_elementary_position!();
}
