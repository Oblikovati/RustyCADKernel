// Copyright (c) 1997-1999 Matra Datavision
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

//! The Rust twin of the CompProjectedCurve operation of
//! tools/oracle/tkgeombase_probe_projlib_curves.cxx.
use super::super::approx::{adapted_curve, adapted_surface, shape};
use super::super::common::{self as p, Reader, attempt, end};
use super::super::geom_convert::{curve as print_curve, curve2d as print_curve2d};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkgeombase::proj_lib::CompProjectedCurve;
use rustycad_tkmath::gp::Point3;

fn parts(x: &CompProjectedCurve) -> Result<(), Failure> {
    p::int(x.nb_curves());
    let sequence = x.get_sequence();
    for i in 1..=x.nb_curves() {
        let (f, l) = x.bounds(i)?;
        (p::real(f), p::real(l));
        let (single, pt) = x.is_single_pnt(i)?;
        (p::flag(single), p::pnt2d(pt));
        let (u_iso, u) = x.is_u_iso(i)?;
        (p::flag(u_iso), p::real(u));
        let (v_iso, v) = x.is_v_iso(i)?;
        (p::flag(v_iso), p::real(v));
        p::real(x.max_distance(i)?);
        let part = sequence
            .get()
            .expect("probe sequence")
            .sequence(|s| s.value(i as usize).cloned())?;
        let points: Vec<Point3> = part
            .get()
            .expect("probe part")
            .sequence(|s| s.iter().copied().collect());
        p::int(points.len() as i32);
        points.into_iter().for_each(p::pnt);
    }
    Ok(())
}

fn eval(x: &CompProjectedCurve) -> Result<(), Failure> {
    if x.nb_curves() < 1 {
        return Ok(());
    }
    let (f, l) = x.bounds(1)?;
    for i in 0..=4 {
        let u = f + (l - f) * f64::from(i) / 4.;
        p::show(x.value(u), p::pnt2d);
        attempt(|| {
            let (q, v) = x.d1(u)?;
            (p::pnt2d(q), p::vec2d(v));
            Ok(())
        });
        attempt(|| {
            let (q, v1, v2) = x.d2(u)?;
            (p::pnt2d(q), p::vec2d(v1), p::vec2d(v2));
            Ok(())
        });
        p::show(x.dn(u, 1), p::vec2d);
        p::show(x.dn(u, 3), p::vec2d);
        p::show(x.dn(u, 0), p::vec2d);
    }
    Ok(())
}

fn intervals(x: &CompProjectedCurve, s: i32) -> Result<(), Failure> {
    let n = x.nb_intervals(shape(s))?;
    p::int(n);
    let mut t: Array1<'static, f64> = Array1::with_bounds(1, n + 1).expect("probe bounds");
    x.intervals(&mut t, shape(s))?;
    t.as_slice().iter().for_each(|v| p::real(*v));
    Ok(())
}

fn adaptor(x: &CompProjectedCurve) {
    p::show(x.first_parameter(), p::real);
    p::show(x.last_parameter(), p::real);
    p::show(x.continuity(), |s| p::int(s as i32));
    p::int(x.get_type().expect("probe type") as i32);
    let (tu, tv) = x.get_tolerance();
    (
        p::real(tu),
        p::real(tv),
        p::flag(x.get_proj2d()),
        p::flag(x.get_proj3d()),
    );
    for s in 0..=6 {
        attempt(|| intervals(x, s));
    }
    attempt(|| {
        let c: Handle<dyn Adaptor2dCurve2d> = x.shallow_copy()?;
        p::int(c.get_type()? as i32);
        p::pnt2d(c.value(0.5 * (c.first_parameter()? + c.last_parameter()?))?);
        Ok(())
    });
    attempt(|| {
        let (f, l) = x.bounds(1)?;
        let t = x.trim(f, 0.5 * (f + l), 1.0e-7)?;
        (p::real(t.first_parameter()?), p::real(t.last_parameter()?));
        Ok(())
    });
    attempt(|| x.line().map(|_| ()));
    attempt(|| x.resolution(1.0e-3).map(|_| ()));
    p::show(x.is_periodic(), p::flag);
}

fn results(x: &CompProjectedCurve) -> Result<(), Failure> {
    for i in 1..=x.nb_curves() {
        p::flag(x.result_is_point(i)?);
        p::real(x.get_result_2d_u_approx_error(i)?);
        p::real(x.get_result_2d_v_approx_error(i)?);
        p::real(x.get_result_3d_approx_error(i)?);
        if x.get_proj2d() {
            print_curve2d(&x.get_result_2d_c(i)?);
            p::show(x.get_result_2d_p(i), p::pnt2d);
        }
        if x.get_proj3d() {
            print_curve(&x.get_result_3d_c(i)?);
            p::show(x.get_result_3d_p(i), p::pnt);
        }
    }
    Ok(())
}

pub fn comp_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let variant = r.integer();
    let (tol3d, tu, tv, md) = (r.real(), r.real(), r.real(), r.real());
    let (p2, p3, cont) = (r.flag(), r.flag(), shape(r.integer()));
    let (deg, seg) = (r.integer(), r.integer());
    attempt(|| {
        let c = adapted_curve(&mut r)?;
        let s = adapted_surface(&mut r)?;
        let x = match variant {
            0 => CompProjectedCurve::with_tolerances(&s, &c, tu, tv)?,
            1 => CompProjectedCurve::with_max_dist(&s, &c, tu, tv, md)?,
            _ => CompProjectedCurve::with_tol3d(tol3d, &s, &c, md)?,
        };
        parts(&x)?;
        eval(&x)?;
        adaptor(&x);
        x.set_tol3d(tol3d);
        x.set_continuity(cont);
        x.set_max_degree(deg);
        x.set_max_seg(seg);
        x.set_proj2d(p2);
        x.set_proj3d(p3);
        attempt(|| {
            x.perform()?;
            results(&x)
        });
        Ok(())
    });
    end();
}
