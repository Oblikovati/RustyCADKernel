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

//! The Rust twin of tools/oracle/tkgeombase_probe_approx_adaptors.cxx.
use super::super::common::{self as p, Reader, attempt, end};
use super::super::geom_convert::{bspline, curve2d as print_curve2d};
use super::super::geom_tools::build::{curve, curve2d, surface};
use super::super::geom2d_convert::bspline as bspline2d;
use super::shape;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::approx::{
    Curve2d, Curve3d, CurveOnSurface, CurveOnSurfaceOptions, CurvilinearOptions,
    CurvilinearParameter, CurvlinFunc, SameParameter,
};

pub fn adapted_curve(r: &mut Reader<'_>) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    let kind = r.integer();
    let range = if kind == 1 {
        Some((r.real(), r.real()))
    } else {
        None
    };
    let c = curve(r)?;
    Ok(match range {
        Some((u1, u2)) => Handle::new(GeomAdaptorCurve::with_range(&c, u1, u2)?).cast(),
        None => Handle::new(GeomAdaptorCurve::with_curve(&c)?).cast(),
    })
}

pub fn adapted_curve2d(r: &mut Reader<'_>) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
    Ok(Handle::new(Geom2dAdaptorCurve::with_curve(&curve2d(r)?)?).cast())
}

pub fn adapted_surface(r: &mut Reader<'_>) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
    Ok(Handle::new(GeomAdaptorSurface::with_surface(&surface(r)?)?).cast())
}

fn dump(text: &str) {
    print!("{} ", text.replace('\n', "|"));
}

pub fn curve3d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (tol, order) = (r.real(), shape(r.integer()));
    let (seg, deg) = (r.integer(), r.integer());
    attempt(|| {
        let c = adapted_curve(&mut r)?;
        let a = Curve3d::new(&c, tol, order, seg, deg)?;
        p::flag(a.is_done());
        p::flag(a.has_result());
        bspline(&a.curve());
        p::real(a.max_error());
        if a.has_result() {
            let mut text = String::new();
            a.dump(&mut text)?;
            dump(&text);
        }
        Ok(())
    });
    end();
}

pub fn curve2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (f, l, tu, tv) = (r.real(), r.real(), r.real(), r.real());
    let cont = shape(r.integer());
    let (deg, seg) = (r.integer(), r.integer());
    attempt(|| {
        let c = adapted_curve2d(&mut r)?;
        let a = Curve2d::new(&c, f, l, tu, tv, cont, deg, seg)?;
        p::flag(a.is_done());
        p::flag(a.has_result());
        bspline2d(&a.curve());
        p::real(a.max_error_2d_u());
        p::real(a.max_error_2d_v());
        Ok(())
    });
    end();
}

fn cos_results(a: &CurveOnSurface) {
    p::flag(a.is_done());
    p::flag(a.has_result());
    bspline(&a.curve_3d());
    p::real(a.max_error_3d());
    bspline2d(&a.curve_2d());
    p::real(a.max_error_2d_u());
    p::real(a.max_error_2d_v());
}

pub fn cos_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let variant = r.integer();
    let (f, l, tol) = (r.real(), r.real(), r.real());
    let continuity = shape(r.integer());
    let (max_degree, max_segments) = (r.integer(), r.integer());
    let (only_3d, only_2d) = (r.flag(), r.flag());
    let options = CurveOnSurfaceOptions {
        max_segments,
        max_degree,
        continuity,
        only_3d,
        only_2d,
    };
    attempt(|| {
        let c = adapted_curve2d(&mut r)?;
        let s = adapted_surface(&mut r)?;
        if variant == 0 {
            cos_results(&CurveOnSurface::new(&c, &s, f, l, tol, options)?);
        } else {
            let mut a = CurveOnSurface::without_perform(&c, &s, f, l, tol);
            cos_results(&a);
            a.perform(options)?;
            cos_results(&a);
        }
        Ok(())
    });
    end();
}

fn curvlin(r: &mut Reader<'_>, kind: i32, tol: f64) -> Result<CurvlinFunc, Failure> {
    if kind == 1 {
        return CurvlinFunc::from_curve(&adapted_curve(r)?, tol);
    }
    let c1 = adapted_curve2d(r)?;
    let s1 = adapted_surface(r)?;
    if kind == 2 {
        return CurvlinFunc::from_curve_on_surface(&c1, &s1, tol);
    }
    let c2 = adapted_curve2d(r)?;
    let s2 = adapted_surface(r)?;
    CurvlinFunc::from_two_curves_on_surfaces(&c1, &c2, &s1, &s2, tol)
}

fn curvlin_command(
    r: &mut Reader<'_>,
    f: &CurvlinFunc,
    kind: i32,
    c3d: &dyn Adaptor3dCurve,
) -> Result<(), Failure> {
    match r.integer() {
        1 => [f.first_parameter(), f.last_parameter(), f.get_length()]
            .into_iter()
            .for_each(p::real),
        2 => {
            let s = shape(r.integer());
            let n = f.nb_intervals(s)?;
            p::int(n);
            let mut t = Array1::with_bounds(1, n + 1).unwrap();
            f.intervals(&mut t, s)?;
            t.iter().for_each(|&v| p::real(v));
        }
        3 => p::real(f.get_s_parameter(r.real())?),
        4 => {
            let (s, order) = (r.real(), r.integer());
            let mut res = Array1::with_bounds(0, 2 * kind).unwrap();
            res.init(-7.);
            let done = match kind {
                1 => f.eval_case1(s, order, &mut res)?,
                2 => f.eval_case2(s, order, &mut res)?,
                _ => f.eval_case3(s, order, &mut res)?,
            };
            p::flag(done);
            res.iter().for_each(|&v| p::real(v));
        }
        5 => {
            let (a, b, t) = (r.real(), r.real(), r.real());
            f.trim(a, b, t)?;
            p::real(f.first_parameter());
            p::real(f.last_parameter());
        }
        6 => {
            f.length()?;
            p::real(f.get_length());
        }
        7 => f.set_tol(r.real()),
        8 => p::real(f.get_u_parameter(c3d, r.real(), 1)?),
        _ => {
            let (a, b) = (r.real(), r.real());
            p::real(f.length_of(c3d, a, b)?);
        }
    }
    Ok(())
}

pub fn curvlin_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, tol) = (r.integer(), r.real());
    attempt(|| {
        let f = curvlin(&mut r, kind, tol)?;
        let c3d = adapted_curve(&mut r)?;
        let c3d = c3d.get().expect("probe curve");
        for _ in 0..r.integer() {
            attempt(|| curvlin_command(&mut r, &f, kind, c3d));
        }
        Ok(())
    });
    end();
}

fn curvilinear_results(a: &CurvilinearParameter, kind: i32) -> Result<(), Failure> {
    p::flag(a.is_done());
    p::flag(a.has_result());
    bspline(&a.curve_3d());
    p::real(a.max_error_3d());
    bspline2d(&a.curve_2d1());
    p::real(a.max_error_2d1());
    bspline2d(&a.curve_2d2());
    if kind != 2 {
        p::real(a.max_error_2d2());
    }
    let mut text = String::new();
    a.dump(&mut text).expect("string");
    dump(&text);
    Ok(())
}

pub fn curvilinear_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, tol) = (r.integer(), r.real());
    let order = shape(r.integer());
    let (max_degree, max_segments) = (r.integer(), r.integer());
    let options = CurvilinearOptions {
        order,
        max_degree,
        max_segments,
    };
    attempt(|| {
        let a = match kind {
            1 => CurvilinearParameter::from_curve(&adapted_curve(&mut r)?, tol, options)?,
            2 => {
                let c = adapted_curve2d(&mut r)?;
                let s = adapted_surface(&mut r)?;
                CurvilinearParameter::from_curve_on_surface(&c, &s, tol, options)?
            }
            _ => {
                let c1 = adapted_curve2d(&mut r)?;
                let s1 = adapted_surface(&mut r)?;
                let c2 = adapted_curve2d(&mut r)?;
                let s2 = adapted_surface(&mut r)?;
                CurvilinearParameter::from_two_curves_on_surfaces(
                    [&c1, &c2],
                    [&s1, &s2],
                    tol,
                    options,
                )?
            }
        };
        curvilinear_results(&a, kind)
    });
    end();
}

fn same_results(a: &SameParameter) -> Result<(), Failure> {
    p::flag(a.is_done());
    p::flag(a.is_same_parameter());
    p::real(a.tol_reached());
    print_curve2d(&a.curve_2d());
    let cos = a.curve_on_surface();
    p::flag(cos.is_null());
    let Some(cos) = cos.get() else {
        return Ok(());
    };
    let (f, l) = (cos.first_parameter()?, cos.last_parameter()?);
    p::real(f);
    p::real(l);
    for i in 0..=4 {
        p::pnt(cos.value(f + (l - f) * f64::from(i) / 4.)?);
    }
    Ok(())
}

pub fn same_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (variant, tol) = (r.integer(), r.real());
    attempt(|| {
        let c3 = curve(&mut r)?;
        let c2 = curve2d(&mut r)?;
        let s = surface(&mut r)?;
        let a3: Handle<dyn Adaptor3dCurve> = Handle::new(GeomAdaptorCurve::with_curve(&c3)?).cast();
        let a_s: Handle<dyn Adaptor3dSurface> =
            Handle::new(GeomAdaptorSurface::with_surface(&s)?).cast();
        let a = match variant {
            0 => SameParameter::from_geometry(&c3, &c2, &s, tol)?,
            1 => SameParameter::from_adaptors_and_pcurve(&a3, &c2, &a_s, tol)?,
            _ => {
                let a2: Handle<dyn Adaptor2dCurve2d> =
                    Handle::new(Geom2dAdaptorCurve::with_curve(&c2)?).cast();
                SameParameter::from_adaptors(&a3, &a2, &a_s, tol)?
            }
        };
        same_results(&a)
    });
    end();
}
