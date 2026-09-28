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
// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_TBezierSurface.cxx (the Geom_Geometry, Geom_Surface and Geom_BoundedSurface members) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The virtual members of GeomEval_TBezierSurface.
use super::evaluation::{self, TBezierSurfaceView};
use super::{CLASS, GeomEvalTBezierSurface};
use crate::geom::{
    GeomBoundedSurface, GeomCurve, GeomGeometry, GeomSurface, GeomSurfaceD1, GeomSurfaceD2,
    GeomSurfaceD3, SurfaceBounds, errors,
};
use crate::geom_eval::bezier_grid::{TensorSums, dump_bounded_surface_base, refuse};
use crate::geom_eval::bezier_results::{
    SurfaceDerivs, check_dn_range, low_order_index, to_d1, to_d2, to_d3,
};
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::dump_flag;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Transform3, Vector3, Xyz};
use std::f64::consts::PI;
use std::io;

impl GeomEvalTBezierSurface {
    // The point and derivatives of the evaluator of order `max`, in the order of
    // `ORDER_PAIRS`: the plain sums, or their rational quotients.
    fn evaluate(&self, u: f64, v: f64, max: usize) -> SurfaceDerivs {
        let view: TBezierSurfaceView<'_> = self.view();
        let sums: TensorSums = evaluation::sums(&view, u, v, max);
        if self.rational {
            return evaluation::rational(&sums, max);
        }
        let mut values = [Xyz::default(); 10];
        values[..sums.n.len()].copy_from_slice(&sums.n);
        values
    }
}

impl GeomGeometry for GeomEvalTBezierSurface {
    fn transform(&self, _transform: Transform3) -> Result<(), Failure> {
        refuse(CLASS, "Transform", "")
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(self.copied()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_bounded_surface_base(stream, depth, CLASS)?;
        dump::dump_real(stream, "AlphaU", self.alpha_u)?;
        dump::dump_real(stream, "AlphaV", self.alpha_v)?;
        dump_flag(stream, "Rational", self.rational)?;
        dump::dump_integer(stream, "Poles.NbRows", self.nb_u_poles())?;
        dump::dump_integer(stream, "Poles.NbColumns", self.nb_v_poles())
    }
}

impl GeomSurface for GeomEvalTBezierSurface {
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
        (0.0, PI / self.alpha_u, 0.0, PI / self.alpha_v)
    }
    fn is_u_closed(&self) -> Result<bool, Failure> {
        Ok(self.boundary_lines_coincide(true))
    }
    fn is_v_closed(&self) -> Result<bool, Failure> {
        Ok(self.boundary_lines_coincide(false))
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
        Ok(Point3::from(self.evaluate(u, v, 0)[0]))
    }
    fn eval_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        Ok(to_d1(self.evaluate(u, v, 1)))
    }
    fn eval_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        Ok(to_d2(self.evaluate(u, v, 2)))
    }
    fn eval_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        Ok(to_d3(self.evaluate(u, v, 3)))
    }
    // Non-rational surfaces differentiate the bases directly; rational ones delegate
    // the orders up to 3 to EvalD1..EvalD3 and refuse higher orders, as natively.
    fn eval_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        check_dn_range(CLASS, nu, nv)?;
        if !self.rational {
            let value: Xyz = evaluation::non_rational_dn(&self.view(), u, v, nu, nv);
            return Ok(Vector3::from(value));
        }
        let Some(index) = low_order_index(nu, nv) else {
            return Err(errors::undefined_derivative(&format!(
                "{CLASS}::EvalDN: Nu = {nu}, Nv = {nv}; rational derivatives of order > 3 are not implemented, expected Nu + Nv <= 3"
            )));
        };
        let max: usize = (nu + nv) as usize;
        Ok(Vector3::from(self.evaluate(u, v, max)[index]))
    }
}

impl GeomBoundedSurface for GeomEvalTBezierSurface {}
