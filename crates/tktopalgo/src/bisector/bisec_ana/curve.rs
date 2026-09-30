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
// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_BisecAna.cxx (the Geom2d_Curve members) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `Geom2d_Geometry` and `Geom2d_Curve` members of `Bisector_BisecAna`: they
//! forward to the trimmed conic, or to its basis for the evaluators.
use super::BisecAna;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3, Geom2dGeometry, Geom2dTrimmedCurve,
    dump_curve,
};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Transform2, Vector2};
use std::io;

impl Geom2dGeometry for BisecAna {
    fn transform(&self, transform: Transform2) -> Result<(), Failure> {
        self.trimmed().transform(transform)
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        let copy: BisecAna = BisecAna::new();
        let trimmed: Handle<Geom2dTrimmedCurve> = self.trimmed().copy()?.cast();
        copy.init(&trimmed);
        Ok(Handle::new(copy).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        // The class inherits Geom2d_Curve::DumpJson, whose class name is static.
        dump_curve(stream, depth)
    }
}

impl Geom2dCurve for BisecAna {
    fn reverse(&self) -> Result<(), Failure> {
        self.trimmed().reverse()
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        self.trimmed().reversed_parameter(u)
    }
    fn first_parameter(&self) -> f64 {
        // NIZHNY-EAP, Feb 2000: the trimmed range, not the basis range.
        self.trimmed().first_parameter()
    }
    fn last_parameter(&self) -> f64 {
        self.trimmed().last_parameter()
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        self.geom2d_curve().is_closed()
    }
    fn is_periodic(&self) -> bool {
        self.geom2d_curve().is_periodic()
    }
    fn continuity(&self) -> Shape {
        self.trimmed().continuity()
    }
    fn is_cn(&self, n: i32) -> Result<bool, Failure> {
        self.trimmed().is_cn(n)
    }
    fn eval_d0(&self, u: f64) -> Result<Point2, Failure> {
        self.geom2d_curve().eval_d0(u)
    }
    fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        self.geom2d_curve().eval_d1(u)
    }
    fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        self.geom2d_curve().eval_d2(u)
    }
    fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        self.geom2d_curve().eval_d3(u)
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        self.geom2d_curve().eval_dn(u, n)
    }
}
