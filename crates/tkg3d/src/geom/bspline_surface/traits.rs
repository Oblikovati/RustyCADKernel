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
// Rust translation of ModelingData/TKG3d/Geom/Geom_BSplineSurface.cxx, Geom_BSplineSurface_1.cxx (virtual members) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The virtual members of Geom_Geometry, Geom_Surface and Geom_BoundedSurface.
use super::GeomBSplineSurface;
use crate::geom::{
    GeomBoundedSurface, GeomCurve, GeomGeometry, GeomSurface, GeomSurfaceD1, GeomSurfaceD2,
    GeomSurfaceD3, SurfaceBounds, dump_surface,
};
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name, dump_flag};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};
use std::io;

impl GeomGeometry for GeomBSplineSurface {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        self.state.write().transform(transform);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(Self::copy_of(self)?).cast())
    }
    /// The native dump; Geom_BoundedSurface has no DumpJson, so its base level is the
    /// Geom_Surface one.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state = self.state.read();
        dump_class_name(stream, "Geom_BSplineSurface")?;
        dump_base(stream, depth, dump_surface)?;
        let flags = [
            ("URational", state.u_rational),
            ("VRational", state.v_rational),
            ("UPeriodic", state.u.periodic),
            ("VPeriodic", state.v.periodic),
        ];
        for (name, value) in flags {
            dump_flag(stream, name, value)?;
        }
        let integers = [
            ("UKnotSet", state.u.knot_set as i32),
            ("VKnotSet", state.v.knot_set as i32),
            ("USmooth", state.u.smooth as i32),
            ("VSmooth", state.v.smooth as i32),
            ("UDeg", state.u.deg),
            ("VDeg", state.v.deg),
            ("Poles.Size", state.poles.len() as i32),
        ];
        for (name, value) in integers {
            dump::dump_integer(stream, name, value)?;
        }
        if state.rational() {
            dump::dump_integer(stream, "Weights.Size", state.weights.len() as i32)?;
        }
        let sizes = [
            ("UFlatKnots.Size", state.u.flat_knots.len()),
            ("VFlatKnots.Size", state.v.flat_knots.len()),
            ("UKnots.Size", state.u.knots.len()),
            ("VKnots.Size", state.v.knots.len()),
            ("UMults.Size", state.u.mults.len()),
            ("VMults.Size", state.v.mults.len()),
        ];
        for (name, size) in sizes {
            dump::dump_integer(stream, name, size as i32)?;
        }
        dump::dump_real(stream, "UMaxDerivInv", state.u_max_deriv_inv)?;
        dump::dump_real(stream, "VMaxDerivInv", state.v_max_deriv_inv)?;
        dump_flag(stream, "MaxDerivInvOk", state.max_deriv_inv_ok)
    }
}

impl GeomSurface for GeomBSplineSurface {
    fn u_reverse(&self) -> Result<(), Failure> {
        self.state.write().reverse(true)
    }
    fn u_reversed_parameter(&self, u: f64) -> f64 {
        let state = self.state.read();
        let knots = state.u.knots.as_slice();
        knots[0] + knots[knots.len() - 1] - u
    }
    fn v_reverse(&self) -> Result<(), Failure> {
        self.state.write().reverse(false)
    }
    fn v_reversed_parameter(&self, v: f64) -> f64 {
        let state = self.state.read();
        let knots = state.v.knots.as_slice();
        knots[0] + knots[knots.len() - 1] - v
    }
    /// The constructor checks guarantee at least `Degree + 3` flat knots per direction,
    /// so the bounds always exist.
    fn bounds(&self) -> SurfaceBounds {
        self.state.read().bounds().unwrap_or_default()
    }
    fn is_u_closed(&self) -> Result<bool, Failure> {
        self.state.read().is_closed(true)
    }
    fn is_v_closed(&self) -> Result<bool, Failure> {
        self.state.read().is_closed(false)
    }
    fn is_u_periodic(&self) -> bool {
        self.state.read().u.periodic
    }
    fn is_v_periodic(&self) -> bool {
        self.state.read().v.periodic
    }
    fn u_iso(&self, u: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        Ok(Handle::new(self.state.read().default_iso(true, u)?).cast())
    }
    fn v_iso(&self, v: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        Ok(Handle::new(self.state.read().default_iso(false, v)?).cast())
    }
    /// The smaller of the U and V continuities (`myUSmooth < myVSmooth ? U : V`).
    fn continuity(&self) -> Shape {
        let state = self.state.read();
        if (state.u.smooth as i32) < (state.v.smooth as i32) {
            state.u.smooth
        } else {
            state.v.smooth
        }
    }
    fn is_cn_u(&self, n: i32) -> Result<bool, Failure> {
        self.state.read().is_cn(true, n)
    }
    fn is_cn_v(&self, n: i32) -> Result<bool, Failure> {
        self.state.read().is_cn(false, n)
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
impl GeomBoundedSurface for GeomBSplineSurface {}
