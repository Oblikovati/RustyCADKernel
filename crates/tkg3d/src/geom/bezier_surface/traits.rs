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
// Rust translation of ModelingData/TKG3d/Geom/Geom_BezierSurface.cxx (the Geom_Geometry, Geom_Surface and Geom_BoundedSurface members) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The virtual members of Geom_Geometry, Geom_Surface and Geom_BoundedSurface.
use super::GeomBezierSurface;
use super::grid::at2;
use crate::geom::{
    GeomBoundedSurface, GeomCurve, GeomGeometry, GeomSurface, GeomSurfaceD1, GeomSurfaceD2,
    GeomSurfaceD3, SurfaceBounds, dump_surface,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name, dump_flag};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};
use std::io;

impl GeomGeometry for GeomBezierSurface {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        self.state.write().transform(transform);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(Self::copy_of(self)?).cast())
    }
    /// The base level is Geom_BoundedSurface::DumpJson, which is the inherited
    /// Geom_Surface one (it writes the class name "Geom_Surface").
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state = self.state.read();
        dump_class_name(stream, "Geom_BezierSurface")?;
        dump_base(stream, depth, dump_surface)?;
        dump_flag(stream, "URational", state.u_rational)?;
        dump_flag(stream, "VRational", state.v_rational)?;
        dump::dump_integer(stream, "Poles.Size", state.poles.len() as i32)?;
        if state.is_rational() {
            dump::dump_integer(stream, "Weights.Size", state.weights.len() as i32)?;
        }
        dump::dump_real(stream, "UMaxDerivInv", state.u_max_deriv_inv)?;
        dump::dump_real(stream, "VMaxDerivInv", state.v_max_deriv_inv)?;
        dump_flag(stream, "MaxDerivInvOk", state.max_deriv_inv_ok)
    }
}

// Whether the first and last pole lines coincide within Precision::Confusion.
fn lines_coincide(
    poles: &rustycad_tkernel::ncollection::Array2<'_, Point3>,
    u: bool,
) -> Result<bool, Failure> {
    let (first, last, count) = if u {
        (
            poles.lower_row(),
            poles.upper_row(),
            poles.lower_col()..=poles.upper_col(),
        )
    } else {
        (
            poles.lower_col(),
            poles.upper_col(),
            poles.lower_row()..=poles.upper_row(),
        )
    };
    for k in count {
        let (a, b) = if u {
            (at2(poles, first, k)?, at2(poles, last, k)?)
        } else {
            (at2(poles, k, first)?, at2(poles, k, last)?)
        };
        if a.distance(&b) > precision::confusion() {
            return Ok(false);
        }
    }
    Ok(true)
}

impl GeomSurface for GeomBezierSurface {
    fn u_reverse(&self) -> Result<(), Failure> {
        self.state.write().u_reverse()
    }
    fn u_reversed_parameter(&self, u: f64) -> f64 {
        1. - u
    }
    fn v_reverse(&self) -> Result<(), Failure> {
        self.state.write().v_reverse()
    }
    fn v_reversed_parameter(&self, v: f64) -> f64 {
        1. - v
    }
    fn bounds(&self) -> SurfaceBounds {
        (0.0, 1.0, 0.0, 1.0)
    }
    fn is_u_closed(&self) -> Result<bool, Failure> {
        lines_coincide(&self.state.read().poles, true)
    }
    fn is_v_closed(&self) -> Result<bool, Failure> {
        lines_coincide(&self.state.read().poles, false)
    }
    fn is_u_periodic(&self) -> bool {
        false
    }
    fn is_v_periodic(&self) -> bool {
        false
    }
    fn u_iso(&self, u: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        self.state.read().iso(u, true)
    }
    fn v_iso(&self, v: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        self.state.read().iso(v, false)
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
        self.state.read().eval_d0(u, v)
    }
    fn eval_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        self.state.read().eval_d1(u, v)
    }
    fn eval_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        self.state.read().eval_d2(u, v)
    }
    fn eval_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        self.state.read().eval_d3(u, v)
    }
    fn eval_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        self.state.read().eval_dn(u, v, nu, nv)
    }
}
impl GeomBoundedSurface for GeomBezierSurface {}
