// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_AHTBezierSurface.cxx (the Geom_Geometry, Geom_Surface and Geom_BoundedSurface members) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The virtual members of GeomEval_AHTBezierSurface.
use super::evaluation::{evaluate, high_order_dn};
use super::{CLASS, GeomEvalAHTBezierSurface};
use crate::geom::{
    GeomBoundedSurface, GeomCurve, GeomGeometry, GeomSurface, GeomSurfaceD1, GeomSurfaceD2,
    GeomSurfaceD3, SurfaceBounds,
};
use crate::geom_eval::bezier_grid::{dump_bounded_surface_base, refuse};
use crate::geom_eval::bezier_results::{
    check_dn_range, low_order_index, to_d1, to_d2, to_d3,
};
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{dump_field, dump_flag};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};
use std::io;

impl GeomGeometry for GeomEvalAHTBezierSurface {
    fn transform(&self, _transform: Transform3) -> Result<(), Failure> {
        refuse(CLASS, "Transform", "")
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(self.copied()).cast())
    }
    // Each pole is dumped under the key "Pole" (DumpFieldToName("&aPole")), row by row.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_bounded_surface_base(stream, depth, CLASS)?;
        dump::dump_integer(stream, "AlgDegreeU", self.alg_degrees[0])?;
        dump::dump_integer(stream, "AlgDegreeV", self.alg_degrees[1])?;
        dump::dump_real(stream, "AlphaU", self.alphas[0])?;
        dump::dump_real(stream, "AlphaV", self.alphas[1])?;
        dump::dump_real(stream, "BetaU", self.betas[0])?;
        dump::dump_real(stream, "BetaV", self.betas[1])?;
        dump_flag(stream, "URational", self.rational)?;
        dump_flag(stream, "VRational", self.rational)?;
        for pole in self.poles.as_slice() {
            dump_field(stream, depth, "Pole", |nested, _| pole.dump_json(nested))?;
        }
        Ok(())
    }
}

impl GeomSurface for GeomEvalAHTBezierSurface {
    fn u_reverse(&self) -> Result<(), Failure> {
        refuse(CLASS, "UReverse", "")
    }
    fn u_reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        refuse(CLASS, "UReversedParameter", &format!("(U = {u})"))
    }
    fn v_reverse(&self) -> Result<(), Failure> {
        refuse(CLASS, "VReverse", "")
    }
    fn v_reversed_parameter(&self, v: f64) -> Result<f64, Failure> {
        refuse(CLASS, "VReversedParameter", &format!("(V = {v})"))
    }
    fn bounds(&self) -> SurfaceBounds {
        (0.0, 1.0, 0.0, 1.0)
    }
    // Always false, even when the boundary poles coincide (the T-Bezier surface tests them).
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
        refuse(CLASS, "UIso", &format!("(U = {u})"))
    }
    fn v_iso(&self, v: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        refuse(CLASS, "VIso", &format!("(V = {v})"))
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
        Ok(Point3::from(evaluate(&self.view(), u, v, 0)[0]))
    }
    fn eval_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        Ok(to_d1(evaluate(&self.view(), u, v, 1)))
    }
    fn eval_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        Ok(to_d2(evaluate(&self.view(), u, v, 2)))
    }
    fn eval_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        Ok(to_d3(evaluate(&self.view(), u, v, 3)))
    }
    // Orders up to 3 delegate to EvalD1..EvalD3 (the Horner path included), as natively.
    fn eval_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        check_dn_range(CLASS, nu, nv)?;
        if let Some(index) = low_order_index(nu, nv) {
            let max: usize = (nu + nv) as usize;
            return Ok(Vector3::from(evaluate(&self.view(), u, v, max)[index]));
        }
        let value = high_order_dn(&self.view(), u, v, nu as usize, nv as usize);
        Ok(Vector3::from(value))
    }
}

impl GeomBoundedSurface for GeomEvalAHTBezierSurface {}
