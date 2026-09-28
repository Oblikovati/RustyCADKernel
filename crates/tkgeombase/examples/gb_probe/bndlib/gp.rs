//! The bnd_gp and bnd_helpers cases of tools/oracle/tkgeombase_probe_bndlib.cxx.
use super::super::common::{self as p, Reader, attempt, end};
use super::super::lprop::build;
use super::{bnd_box, bnd_box2d, surface};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::bnd_lib;
use rustycad_tkgeombase::geom_bnd_lib::{
    Circle, Circle2d, Ellipse, Ellipse2d, Hyperbola, Hyperbola2d, Line, Line2d, Parabola,
    Parabola2d, optimization, sampling, spline,
};
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};
use rustycad_tkmath::gp::{
    Circle2, Circle3, Cone3, Cylinder3, Ellipse2, Ellipse3, Hyperbola2, Hyperbola3, Line2, Line3,
    Parabola2, Parabola3, Sphere3, Torus3,
};

// The kinds of bnd_gp whose box is a Bnd_Box2d.
fn flat(kind: i32) -> bool {
    matches!(kind, 1 | 4 | 5 | 8 | 9 | 11 | 13)
}

// Arc parameters, read only for the arc kinds.
fn arc(r: &mut Reader<'_>, is_arc: bool) -> (f64, f64) {
    if is_arc {
        (r.real(), r.real())
    } else {
        (0.0, 0.0)
    }
}

// The 3D curve kinds 0, 2, 3, 6, 7, 10, 12.
fn curve3d(r: &mut Reader<'_>, lib: i32, kind: i32, b: &mut BoundingBox3) -> Result<(), Failure> {
    let geom = lib != 0;
    match kind {
        0 => {
            let pnt = r.pnt();
            let l = Line3::new(pnt, r.dir());
            let (p1, p2, t) = (r.real(), r.real(), r.real());
            if geom {
                *b = Line::lin_box(&l, p1, p2, t)?
            } else {
                bnd_lib::add_lin(&l, p1, p2, t, b)?
            }
        }
        2 | 3 => {
            let a = r.ax2();
            let c = Circle3::new(a, r.real())?;
            let (p1, p2) = arc(r, kind == 3);
            let t = r.real();
            match (geom, kind == 3) {
                (false, false) => bnd_lib::add_circ(&c, t, b)?,
                (false, true) => bnd_lib::add_circ_arc(&c, p1, p2, t, b)?,
                (true, false) => *b = Circle::circ_box(&c, t),
                (true, true) => *b = Circle::circ_arc_box(&c, p1, p2, t),
            }
        }
        6 | 7 => {
            let (a, ra) = (r.ax2(), r.real());
            let c = Ellipse3::new(a, ra, r.real())?;
            let (p1, p2) = arc(r, kind == 7);
            let t = r.real();
            match (geom, kind == 7) {
                (false, false) => bnd_lib::add_elips(&c, t, b)?,
                (false, true) => bnd_lib::add_elips_arc(&c, p1, p2, t, b)?,
                (true, false) => *b = Ellipse::elips_box(&c, t),
                (true, true) => *b = Ellipse::elips_arc_box(&c, p1, p2, t),
            }
        }
        10 => {
            let a = r.ax2();
            let c = Parabola3::new(a, r.real())?;
            let (p1, p2, t) = (r.real(), r.real(), r.real());
            if geom {
                *b = Parabola::parab_box(&c, p1, p2, t)?
            } else {
                bnd_lib::add_parab(&c, p1, p2, t, b)?
            }
        }
        _ => {
            let (a, ra) = (r.ax2(), r.real());
            let c = Hyperbola3::new(a, ra, r.real())?;
            let (p1, p2, t) = (r.real(), r.real(), r.real());
            if geom {
                *b = Hyperbola::hypr_box(&c, p1, p2, t)?
            } else {
                bnd_lib::add_hypr(&c, p1, p2, t, b)?
            }
        }
    }
    Ok(())
}

// The 2D curve kinds 1, 4, 5, 8, 9, 11, 13.
fn curve2d(r: &mut Reader<'_>, lib: i32, kind: i32, b: &mut BoundingBox2) -> Result<(), Failure> {
    let geom = lib != 0;
    if kind == 1 {
        let pnt = r.pnt2d();
        let l = Line2::new(pnt, r.dir2d());
        let (p1, p2, t) = (r.real(), r.real(), r.real());
        if geom {
            *b = Line2d::lin_box(&l, p1, p2, t)?
        } else {
            bnd_lib::add_lin2d(&l, p1, p2, t, b)?
        }
        return Ok(());
    }
    let (a, s) = (r.ax2d(), r.flag());
    match kind {
        4 | 5 => {
            let c = Circle2::from_x_axis(a, r.real(), s)?;
            let (p1, p2) = arc(r, kind == 5);
            let t = r.real();
            match (geom, kind == 5) {
                (false, false) => bnd_lib::add_circ2d(&c, t, b),
                (false, true) => bnd_lib::add_circ2d_arc(&c, p1, p2, t, b),
                (true, false) => *b = Circle2d::circ_box(&c, t),
                (true, true) => *b = Circle2d::circ_arc_box(&c, p1, p2, t),
            }
        }
        8 | 9 => {
            let ra = r.real();
            let c = Ellipse2::from_x_axis(a, ra, r.real(), s)?;
            let (p1, p2) = arc(r, kind == 9);
            let t = r.real();
            match (geom, kind == 9) {
                (false, false) => bnd_lib::add_elips2d(&c, t, b),
                (false, true) => bnd_lib::add_elips2d_arc(&c, p1, p2, t, b),
                (true, false) => *b = Ellipse2d::elips_box(&c, t),
                (true, true) => *b = Ellipse2d::elips_arc_box(&c, p1, p2, t),
            }
        }
        11 => {
            let c = Parabola2::from_mirror_axis(a, r.real(), s)?;
            let (p1, p2, t) = (r.real(), r.real(), r.real());
            if geom {
                *b = Parabola2d::parab_box(&c, p1, p2, t)?
            } else {
                bnd_lib::add_parab2d(&c, p1, p2, t, b)?
            }
        }
        _ => {
            let ra = r.real();
            let c = Hyperbola2::from_x_axis(a, ra, r.real(), s)?;
            let (p1, p2, t) = (r.real(), r.real(), r.real());
            if geom {
                *b = Hyperbola2d::hypr_box(&c, p1, p2, t)?
            } else {
                bnd_lib::add_hypr2d(&c, p1, p2, t, b)?
            }
        }
    }
    Ok(())
}

// The elementary surface kinds 14-21 (BndLib only).
fn surface_kind(r: &mut Reader<'_>, kind: i32, b: &mut BoundingBox3) -> Result<(), Failure> {
    let a = r.ax3();
    match kind {
        14 | 15 => {
            let c = Cylinder3::new(a, r.real())?;
            let (u1, u2) = arc(r, kind == 14);
            let (v1, v2, t) = (r.real(), r.real(), r.real());
            if kind == 14 {
                bnd_lib::add_cylinder_patch(&c, [u1, u2, v1, v2], t, b)?
            } else {
                bnd_lib::add_cylinder(&c, v1, v2, t, b)?
            }
        }
        16 | 17 => {
            let ang = r.real();
            let c = Cone3::new(a, ang, r.real())?;
            let (u1, u2) = arc(r, kind == 16);
            let (v1, v2, t) = (r.real(), r.real(), r.real());
            if kind == 16 {
                bnd_lib::add_cone_patch(&c, [u1, u2, v1, v2], t, b)?
            } else {
                bnd_lib::add_cone(&c, v1, v2, t, b)?
            }
        }
        18 | 19 => {
            let c = Sphere3::new(a, r.real())?;
            if kind == 18 {
                bnd_lib::add_sphere(&c, r.real(), b);
            } else {
                let patch = [r.real(), r.real(), r.real(), r.real()];
                bnd_lib::add_sphere_patch(&c, patch, r.real(), b)?;
            }
        }
        _ => {
            let ra = r.real();
            let c = Torus3::new(a, ra, r.real())?;
            if kind == 20 {
                bnd_lib::add_torus(&c, r.real(), b);
            } else {
                let patch = [r.real(), r.real(), r.real(), r.real()];
                bnd_lib::add_torus_patch(&c, patch, r.real(), b)?;
            }
        }
    }
    Ok(())
}

pub fn gp_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (lib, kind, npre) = (r.integer(), r.integer(), r.integer());
    let (mut b, mut b2) = (BoundingBox3::default(), BoundingBox2::default());
    for _ in 0..npre {
        if flat(kind) {
            b2.add_point(r.pnt2d())
        } else {
            b.add_point(r.pnt())
        }
    }
    attempt(|| {
        match kind {
            1 | 4 | 5 | 8 | 9 | 11 | 13 => curve2d(&mut r, lib, kind, &mut b2)?,
            0..=13 => curve3d(&mut r, lib, kind, &mut b)?,
            _ => surface_kind(&mut r, kind, &mut b)?,
        }
        if flat(kind) {
            bnd_box2d(&b2)
        } else {
            bnd_box(&b)
        }
        Ok(())
    });
    end();
}

// The 3D curve modes 0-3 of bnd_helpers.
fn helpers3d(a: &GeomAdaptorCurve, mode: i32, h: &Helpers) -> Result<(), Failure> {
    let a: &dyn Adaptor3dCurve = a;
    match mode {
        0 => p::int(sampling::compute_nb_samples(a, h.u1, h.u2)?),
        1 => {
            p::real(optimization::adjust_extr_curve(
                a, h.u1, h.u2, h.extr, h.index, h.tol, true,
            )?);
            p::real(optimization::adjust_extr_curve(
                a, h.u1, h.u2, h.extr, h.index, h.tol, false,
            )?);
        }
        2 => bnd_box(&spline::curve_box_optimal(a, h.u1, h.u2, h.tol)?),
        _ => {
            let mut b = BoundingBox3::default();
            p::real(spline::fill_box(&mut b, a, h.u1, h.u2, h.n)?);
            bnd_box(&b);
        }
    }
    Ok(())
}

// The 2D curve modes 4-6 of bnd_helpers.
fn helpers2d(a: &Geom2dAdaptorCurve, mode: i32, h: &Helpers) -> Result<(), Failure> {
    let a: &dyn Adaptor2dCurve2d = a;
    match mode {
        4 => p::int(sampling::compute_nb_samples_2d(a, h.u1, h.u2)?),
        5 => {
            p::real(optimization::adjust_extr_curve(
                a, h.u1, h.u2, h.extr, h.index, h.tol, true,
            )?);
            p::real(optimization::adjust_extr_curve(
                a, h.u1, h.u2, h.extr, h.index, h.tol, false,
            )?);
        }
        _ => bnd_box2d(&spline::curve_box_optimal(a, h.u1, h.u2, h.tol)?),
    }
    Ok(())
}

// The surface modes 7-8 of bnd_helpers.
fn helpers_surface(a: &GeomAdaptorSurface, mode: i32, h: &Helpers) -> Result<(), Failure> {
    let a: &dyn Adaptor3dSurface = a;
    if mode == 7 {
        p::int(sampling::compute_nb_u_samples(a, h.u1, h.u2)?);
        p::int(sampling::compute_nb_v_samples(a, h.v1, h.v2)?);
        p::int(sampling::compute_nb_u_samples_full(a)?);
        p::int(sampling::compute_nb_v_samples_full(a)?);
        return Ok(());
    }
    let patch = [h.u1, h.u2, h.v1, h.v2];
    p::real(optimization::adjust_extr_surf(
        a, patch, h.extr, h.index, h.tol, true,
    )?);
    p::real(optimization::adjust_extr_surf(
        a, patch, h.extr, h.index, h.tol, false,
    )?);
    Ok(())
}

// The shared arguments of bnd_helpers.
struct Helpers {
    u1: f64,
    u2: f64,
    v1: f64,
    v2: f64,
    extr: f64,
    tol: f64,
    index: i32,
    n: i32,
}

pub fn helpers_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mode = r.integer();
    let h = Helpers {
        u1: r.real(),
        u2: r.real(),
        v1: r.real(),
        v2: r.real(),
        extr: r.real(),
        tol: r.real(),
        index: r.integer(),
        n: r.integer(),
    };
    attempt(|| {
        if mode <= 3 {
            let a = GeomAdaptorCurve::with_curve(&build::curve3d(&mut r)?)?;
            return helpers3d(&a, mode, &h);
        }
        if mode <= 6 {
            let a = Geom2dAdaptorCurve::with_curve(&build::curve2d(&mut r)?)?;
            return helpers2d(&a, mode, &h);
        }
        let a = GeomAdaptorSurface::with_surface(&surface(&mut r)?)?;
        helpers_surface(&a, mode, &h)
    });
    end();
}
