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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BSplineCurve.cxx, Geom2d_BSplineCurve_1.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The virtual members of Geom2d_Geometry, Geom2d_Curve and Geom2d_BoundedCurve.
use super::Geom2dBSplineCurve;
use crate::geom2d::{
    Geom2dBoundedCurve, Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3, Geom2dGeometry,
    dump_base, dump_bounded_curve, dump_class_name,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Transform2, Vector2};
use std::io;

impl Geom2dGeometry for Geom2dBSplineCurve {
    fn transform(&self, transform: Transform2) -> Result<(), Failure> {
        self.state
            .write()
            .transform_poles(|pole| pole.transform(transform));
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        Ok(Handle::new(Self::copy_of(self)?).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state = self.state.read();
        dump_class_name(stream, "Geom2d_BSplineCurve")?;
        dump_base(stream, depth, dump_bounded_curve)?;
        state.dump_fields(stream)
    }
}
impl Geom2dCurve for Geom2dBSplineCurve {
    fn reverse(&self) -> Result<(), Failure> {
        self.state.write().reverse()
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        Ok(self.first_parameter() + self.last_parameter() - u)
    }
    fn first_parameter(&self) -> f64 {
        self.state.read().first_parameter()
    }
    fn last_parameter(&self) -> f64 {
        self.state.read().last_parameter()
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        let state = self.state.read();
        let (start, end) = (state.start_point()?, state.end_point()?);
        Ok(start.square_distance(end) <= precision::computational())
    }
    fn is_periodic(&self) -> bool {
        self.state.read().periodic
    }
    fn continuity(&self) -> Shape {
        self.state.read().smooth
    }
    fn is_cn(&self, n: i32) -> Result<bool, Failure> {
        self.state.read().is_cn(n)
    }
    fn eval_d0(&self, u: f64) -> Result<Point2, Failure> {
        self.state.read().eval_d0(u)
    }
    fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        let (point, d1) = self.state.read().eval_d1(u)?;
        Ok(Geom2dCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        let (point, d1, d2) = self.state.read().eval_d2(u)?;
        Ok(Geom2dCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        let (point, d1, d2, d3) = self.state.read().eval_d3(u)?;
        Ok(Geom2dCurveD3 { point, d1, d2, d3 })
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        self.state.read().eval_dn(u, n)
    }
}
impl Geom2dBoundedCurve for Geom2dBSplineCurve {
    fn end_point(&self) -> Result<Point2, Failure> {
        self.state.read().end_point()
    }
    fn start_point(&self) -> Result<Point2, Failure> {
        self.state.read().start_point()
    }
}
