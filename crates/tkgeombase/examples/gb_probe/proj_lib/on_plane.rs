// Copyright (c) 1994-1999 Matra Datavision
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

//! The Rust twin of the ProjectOnPlane operation of
//! tools/oracle/tkgeombase_probe_projlib.cxx.
use super::super::approx::adapted_curve;
use super::super::common::{self as p, Reader, attempt, end};
use super::super::geom_convert::{bezier, bspline};
use super::super::geom_tools::build::ax3;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::proj_lib::ProjectOnPlane;
use rustycad_tkmath::geom_abs::Shape;

/// gb_pl_on_plane_eval.
pub fn eval(c: &dyn Adaptor3dCurve, u: f64) {
    p::show(c.value(u), p::pnt);
    attempt(|| {
        let (q, v1) = c.d1(u)?;
        (p::pnt(q), p::vec(v1));
        Ok(())
    });
    attempt(|| {
        let (q, v1, v2) = c.d2(u)?;
        (p::pnt(q), p::vec(v1), p::vec(v2));
        Ok(())
    });
    attempt(|| {
        let (q, v1, v2, v3) = c.d3(u)?;
        (p::pnt(q), p::vec(v1), p::vec(v2), p::vec(v3));
        Ok(())
    });
    p::show(c.dn(u, 2), p::vec);
}

fn typed(x: &ProjectOnPlane) {
    p::show(x.line(), p::lin);
    p::show(x.circle(), p::circ);
    p::show(x.ellipse(), p::elips);
    p::show(x.hyperbola(), p::hypr);
    p::show(x.parabola(), p::parab);
    p::show(x.degree(), p::int);
    p::show(x.is_rational(), p::flag);
    p::show(x.nb_poles(), p::int);
    p::show(x.nb_knots(), p::int);
    p::show(x.bezier(), |b| bezier(&b));
    p::show(x.bspline(), |b| bspline(&b));
}

fn intervals(x: &ProjectOnPlane) -> Result<(), Failure> {
    let n = x.nb_intervals(Shape::C2)?;
    p::int(n);
    let mut t: Array1<'static, f64> = Array1::with_bounds(1, n + 1).expect("probe bounds");
    x.intervals(&mut t, Shape::C2)?;
    t.as_slice().iter().for_each(|v| p::real(*v));
    Ok(())
}

fn samples(x: &ProjectOnPlane) -> Result<(), Failure> {
    let (f, l) = (x.first_parameter()?, x.last_parameter()?);
    (p::real(f), p::real(l));
    for i in 0..=4 {
        eval(x, f + (l - f) * f64::from(i) / 4.);
    }
    attempt(|| {
        let t = x.trim(f + (l - f) / 8., l - (l - f) / 8., 1.0e-7)?;
        (p::real(t.first_parameter()?), p::real(t.last_parameter()?));
        p::pnt(t.value(0.5 * (f + l))?);
        Ok(())
    });
    attempt(|| {
        let c = x.shallow_copy()?;
        p::int(c.get_type()? as i32);
        eval(c.get().expect("probe copy"), 0.5 * (f + l));
        Ok(())
    });
    Ok(())
}

fn results(x: &ProjectOnPlane) -> Result<(), Failure> {
    p::int(x.get_type()? as i32);
    p::ax3(x.get_plane());
    p::dir(x.get_direction());
    p::flag(x.get_result().is_null());
    p::flag(x.get_curve().is_null());
    p::show(x.is_periodic(), p::flag);
    p::show(x.period(), p::real);
    p::show(x.is_closed(), p::flag);
    p::show(x.continuity(), |s| p::int(s as i32));
    p::show(x.resolution(1.0e-3), p::real);
    attempt(|| intervals(x));
    typed(x);
    attempt(|| samples(x));
    Ok(())
}

pub fn on_plane_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (variant, keep, tol) = (r.integer(), r.flag(), r.real());
    let (pl, d) = (ax3(&mut r), r.dir());
    attempt(|| {
        let c: Handle<dyn Adaptor3dCurve> = adapted_curve(&mut r)?;
        let x = match variant {
            0 => ProjectOnPlane::new(),
            1 => ProjectOnPlane::with_plane(&pl),
            _ => ProjectOnPlane::with_direction(&pl, d),
        };
        p::int(x.get_type()? as i32);
        x.load(&c, tol, keep)?;
        if variant == 3 {
            x.load(&c, tol, !keep)?;
        }
        results(&x)
    });
    end();
}
