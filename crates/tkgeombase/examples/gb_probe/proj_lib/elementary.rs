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

//! The Rust twin of the projector, static, PrjFunc and PrjResolve operations of
//! tools/oracle/tkgeombase_probe_projlib.cxx.
use super::super::common::{self as p, Reader, attempt, end};
use super::super::geom_tools::build::{curve, surface};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::proj_lib::{
    self as projlib, Cone, Cylinder, Plane, PrjFunc, PrjResolve, PrjResolveBox, Project, Projector,
    Sphere, Torus,
};
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::math::{Matrix, Vector};

/// The projector kinds of `gb_pl_make`.
enum AnyProjector {
    Base(Projector),
    Plane(Plane),
    Cylinder(Cylinder),
    Cone(Cone),
    Sphere(Sphere),
    Torus(Torus),
}

impl AnyProjector {
    fn project(&mut self) -> &mut dyn Project {
        match self {
            Self::Base(x) => x,
            Self::Plane(x) => x,
            Self::Cylinder(x) => x,
            Self::Cone(x) => x,
            Self::Sphere(x) => x,
            Self::Torus(x) => x,
        }
    }
    fn base(&mut self) -> &mut Projector {
        match self {
            Self::Base(x) => x,
            Self::Plane(x) => x,
            Self::Cylinder(x) => x,
            Self::Cone(x) => x,
            Self::Sphere(x) => x,
            Self::Torus(x) => x,
        }
    }
}

fn geom_surface(r: &mut Reader<'_>) -> Result<GeomAdaptorSurface, Failure> {
    GeomAdaptorSurface::with_surface(&surface(r)?)
}

fn geom_curve(r: &mut Reader<'_>) -> Result<GeomAdaptorCurve, Failure> {
    GeomAdaptorCurve::with_curve(&curve(r)?)
}

fn make(which: i32, init: bool, s: &GeomAdaptorSurface) -> Result<AnyProjector, Failure> {
    if which == 0 {
        return Ok(AnyProjector::Base(Projector::new()));
    }
    Ok(match s.get_type()? {
        SurfaceType::Plane => {
            let mut x = Plane::new();
            if init {
                x.init(&s.plane()?);
            }
            AnyProjector::Plane(x)
        }
        SurfaceType::Cylinder => {
            let mut x = Cylinder::new();
            if init {
                x.init(&s.cylinder()?);
            }
            AnyProjector::Cylinder(x)
        }
        SurfaceType::Cone => {
            let mut x = Cone::new();
            if init {
                x.init(&s.cone()?);
            }
            AnyProjector::Cone(x)
        }
        SurfaceType::Sphere => {
            let mut x = Sphere::new();
            if init {
                x.init(&s.sphere()?);
            }
            AnyProjector::Sphere(x)
        }
        SurfaceType::Torus => {
            let mut x = Torus::new();
            if init {
                x.init(&s.torus()?);
            }
            AnyProjector::Torus(x)
        }
        _ => AnyProjector::Base(Projector::new()),
    })
}

fn project(p: &mut dyn Project, c: &GeomAdaptorCurve) -> Result<(), Failure> {
    match c.get_type()? {
        CurveType::Line => p.project_line(&c.line()?),
        CurveType::Circle => p.project_circle(&c.circle()?),
        CurveType::Ellipse => p.project_ellipse(&c.ellipse()?),
        CurveType::Parabola => p.project_parabola(&c.parabola()?),
        CurveType::Hyperbola => p.project_hyperbola(&c.hyperbola()?),
        _ => Ok(()),
    }
}

/// gb_pl_projector.
pub fn projector(x: &Projector) {
    p::flag(x.is_done());
    p::int(x.get_type() as i32);
    p::flag(x.is_periodic());
    p::show(x.line(), p::lin2d);
    p::show(x.circle(), p::circ2d);
    p::show(x.ellipse(), p::elips2d);
    p::show(x.hyperbola(), p::hypr2d);
    p::show(x.parabola(), p::parab2d);
    p::flag(x.bspline().is_null());
    p::flag(x.bezier().is_null());
}

pub fn elem_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (which, init, mode) = (r.integer(), r.flag(), r.integer());
    let (a, b, c, d) = (r.real(), r.real(), r.real(), r.real());
    attempt(|| {
        let s = geom_surface(&mut r)?;
        let cu = geom_curve(&mut r)?;
        let mut x = make(which, init, &s)?;
        attempt(|| project(x.project(), &cu));
        match (mode, &mut x) {
            (1, AnyProjector::Sphere(sphere)) => attempt(|| sphere.set_in_bounds(a)),
            (2, _) => x.base().u_frame(a, b, c, d),
            (3, _) => x.base().v_frame(a, b, c, d),
            _ => {}
        }
        projector(x.base());
        Ok(())
    });
    end();
}

fn static_point(
    s: &dyn Adaptor3dSurface,
    point: rustycad_tkmath::gp::Point3,
) -> Result<(), Failure> {
    match s.get_type()? {
        SurfaceType::Plane => p::pnt2d(projlib::project_point_on_plane(&s.plane()?, point)),
        SurfaceType::Cylinder => {
            p::pnt2d(projlib::project_point_on_cylinder(&s.cylinder()?, point))
        }
        SurfaceType::Cone => p::pnt2d(projlib::project_point_on_cone(&s.cone()?, point)),
        SurfaceType::Sphere => p::pnt2d(projlib::project_point_on_sphere(&s.sphere()?, point)),
        SurfaceType::Torus => p::pnt2d(projlib::project_point_on_torus(&s.torus()?, point)),
        _ => {}
    }
    Ok(())
}

fn static_plane(
    s: &dyn Adaptor3dSurface,
    c: &GeomAdaptorCurve,
    ct: CurveType,
) -> Result<(), Failure> {
    let pl = s.plane()?;
    match ct {
        CurveType::Line => p::lin2d(projlib::project_line_on_plane(&pl, &c.line()?)?),
        CurveType::Circle => p::circ2d(projlib::project_circle_on_plane(&pl, &c.circle()?)?),
        CurveType::Ellipse => p::elips2d(projlib::project_ellipse_on_plane(&pl, &c.ellipse()?)?),
        CurveType::Parabola => p::parab2d(projlib::project_parabola_on_plane(&pl, &c.parabola()?)?),
        CurveType::Hyperbola => {
            p::hypr2d(projlib::project_hyperbola_on_plane(&pl, &c.hyperbola()?)?)
        }
        _ => {}
    }
    Ok(())
}

fn static_curve(s: &dyn Adaptor3dSurface, c: &GeomAdaptorCurve) -> Result<(), Failure> {
    let ct = c.get_type()?;
    match s.get_type()? {
        SurfaceType::Plane => static_plane(s, c, ct)?,
        SurfaceType::Cylinder => {
            if ct == CurveType::Line {
                p::lin2d(projlib::project_line_on_cylinder(
                    &s.cylinder()?,
                    &c.line()?,
                )?);
            }
            if ct == CurveType::Circle {
                p::lin2d(projlib::project_circle_on_cylinder(
                    &s.cylinder()?,
                    &c.circle()?,
                )?);
            }
        }
        SurfaceType::Cone => {
            if ct == CurveType::Line {
                p::lin2d(projlib::project_line_on_cone(&s.cone()?, &c.line()?)?);
            }
            if ct == CurveType::Circle {
                p::lin2d(projlib::project_circle_on_cone(&s.cone()?, &c.circle()?)?);
            }
        }
        SurfaceType::Sphere if ct == CurveType::Circle => {
            p::lin2d(projlib::project_circle_on_sphere(
                &s.sphere()?,
                &c.circle()?,
            )?);
        }
        SurfaceType::Torus if ct == CurveType::Circle => {
            p::lin2d(projlib::project_circle_on_torus(&s.torus()?, &c.circle()?)?);
        }
        _ => {}
    }
    Ok(())
}

pub fn static_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mode = r.integer();
    attempt(|| {
        let hs: Handle<dyn Adaptor3dSurface> = Handle::new(geom_surface(&mut r)?).cast();
        p::flag(projlib::is_ana_surf(&hs)?);
        let s: &dyn Adaptor3dSurface = hs.get().expect("probe surface");
        if mode == 0 {
            static_point(s, r.pnt())
        } else {
            static_curve(s, &geom_curve(&mut r)?)
        }
    });
    end();
}

fn matrix(m: &Matrix<'_>) {
    for i in 1..=2 {
        for j in 1..=2 {
            p::real(m[(i, j)]);
        }
    }
}

pub fn prjfunc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (fix, fv, x1, x2) = (r.integer(), r.real(), r.real(), r.real());
    attempt(|| {
        let c = geom_curve(&mut r)?;
        let s = geom_surface(&mut r)?;
        let mut f = PrjFunc::new(&c, fv, &s, fix)?;
        p::int(f.nb_variables());
        p::int(f.nb_equations());
        p::pnt2d(f.solution());
        let mut x: Vector<'static> = Vector::with_bounds(1, 2)?;
        let mut v: Vector<'static> = Vector::with_bounds(1, 2)?;
        let mut m: Matrix<'static> = Matrix::with_bounds(1, 2, 1, 2).unwrap();
        (x[1], x[2]) = (x1, x2);
        p::flag(f.value(&x, &mut v)?);
        p::real(v[1]);
        p::real(v[2]);
        p::pnt2d(f.solution());
        x[1] = x1 + 0.125;
        p::flag(f.derivatives(&x, &mut m)?);
        matrix(&m);
        x[2] = x2 - 0.25;
        p::flag(f.values(&x, &mut v, &mut m)?);
        p::real(v[1]);
        p::real(v[2]);
        matrix(&m);
        p::pnt2d(f.solution());
        Ok(())
    });
    end();
}

pub fn prjresolve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (fix, t, u, v) = (r.integer(), r.real(), r.real(), r.real());
    let bounds = PrjResolveBox {
        tol: r.pnt2d(),
        inf: r.pnt2d(),
        sup: r.pnt2d(),
    };
    let (ftol, strict) = (r.real(), r.flag());
    attempt(|| {
        let c = geom_curve(&mut r)?;
        let s = geom_surface(&mut r)?;
        let (cr, sr): (&dyn Adaptor3dCurve, &dyn Adaptor3dSurface) = (&c, &s);
        let mut x = PrjResolve::new(cr, sr, fix)?;
        p::flag(x.is_done());
        p::show(x.solution(), p::pnt2d);
        x.perform(t, u, v, bounds, ftol, strict)?;
        p::flag(x.is_done());
        p::show(x.solution(), p::pnt2d);
        Ok(())
    });
    end();
}
