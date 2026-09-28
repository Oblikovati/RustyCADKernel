//! gb_ia_quadric, gb_ia_conic_quad, gb_ia_conic_plane and gb_ia_lin_torus.
use super::{Surface, quadric, surface};
use crate::gb_probe::common::{self as p, Reader, attempt, end, flag, int, real};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana::{IntConicQuad, IntLinTorus, Quadric};
use rustycad_tkmath::gp::{Circle3, Ellipse3, Hyperbola3, Line3, Parabola3, Plane3, Torus3};

/// One conic of the corpus (GbConic3d).
enum Conic3d {
    Line(Line3),
    Circle(Circle3),
    Ellipse(Ellipse3),
    Parabola(Parabola3),
    Hyperbola(Hyperbola3),
}

fn conic3d(r: &mut Reader) -> Result<Conic3d, Failure> {
    let kind = r.integer();
    if kind == 0 {
        let point = r.pnt();
        return Ok(Conic3d::Line(Line3::new(point, r.dir())));
    }
    let a = r.ax2();
    Ok(match kind {
        1 => Conic3d::Circle(Circle3::new(a, r.real())?),
        2 => {
            let major = r.real();
            Conic3d::Ellipse(Ellipse3::new(a, major, r.real())?)
        }
        3 => Conic3d::Parabola(Parabola3::new(a, r.real())?),
        _ => {
            let major = r.real();
            Conic3d::Hyperbola(Hyperbola3::new(a, major, r.real())?)
        }
    })
}

pub fn ia_quadric_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = surface(&mut r)?;
        let axis = r.ax3();
        let mut q: Quadric = quadric(&s)?;
        q.coefficients().into_iter().for_each(real);
        match s {
            Surface::Plane(x) => q.set_plane(x)?,
            Surface::Cylinder(x) => q.set_cylinder(x),
            Surface::Sphere(x) => q.set_sphere(x),
            Surface::Cone(x) => q.set_cone(x),
            Surface::Torus(_) => {}
        }
        int(q.special_points().len() as i32);
        q.special_points().iter().for_each(|point| p::pnt(*point));
        q.new_coefficients(q.coefficients(), axis)?
            .into_iter()
            .for_each(real);
        Quadric::new().coefficients().into_iter().for_each(real);
        Ok(())
    });
    end();
}

fn conic_quad(q: &IntConicQuad) -> Result<(), Failure> {
    flag(q.is_done());
    if !q.is_done() {
        p::show(q.nb_points(), int);
        return Ok(());
    }
    flag(q.is_in_quadric()?);
    flag(q.is_parallel()?);
    attempt(|| {
        let n = q.nb_points()?;
        int(n);
        for i in 1..=n {
            p::pnt(q.point(i)?);
            real(q.param_on_conic(i)?);
        }
        real(q.param_on_conic(n + 1)?);
        Ok(())
    });
    Ok(())
}

pub fn ia_conic_quad_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let g = conic3d(&mut r)?;
        let q = quadric(&surface(&mut r)?)?;
        let mut i = IntConicQuad::new();
        match g {
            Conic3d::Line(x) => i.perform_line_quadric(x, &q)?,
            Conic3d::Circle(x) => i.perform_circle_quadric(x, &q)?,
            Conic3d::Ellipse(x) => i.perform_ellipse_quadric(x, &q)?,
            Conic3d::Parabola(x) => i.perform_parabola_quadric(x, &q)?,
            Conic3d::Hyperbola(x) => i.perform_hyperbola_quadric(x, &q)?,
        }
        conic_quad(&i)
    });
    end();
}

pub fn ia_conic_plane_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let g = conic3d(&mut r)?;
        let plane = Plane3::from_position(r.ax3());
        let (ta, t, len) = (r.real(), r.real(), r.real());
        let mut i = IntConicQuad::new();
        match g {
            Conic3d::Line(x) => i.perform_line_plane(x, plane, ta, t, len)?,
            Conic3d::Circle(x) => i.perform_circle_plane(x, plane, ta, t)?,
            Conic3d::Ellipse(x) => i.perform_ellipse_plane(x, plane, ta, t)?,
            Conic3d::Parabola(x) => i.perform_parabola_plane(x, plane, ta)?,
            Conic3d::Hyperbola(x) => i.perform_hyperbola_plane(x, plane, ta)?,
        }
        conic_quad(&i)
    });
    end();
}

pub fn ia_lin_torus_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let point = r.pnt();
        let line = Line3::new(point, r.dir());
        let (a, major) = (r.ax3(), r.real());
        let torus = Torus3::new(a, major, r.real())?;
        let i = IntLinTorus::with(line, torus)?;
        flag(i.is_done());
        attempt(|| {
            let n = i.nb_points()?;
            int(n);
            for k in 1..=n {
                p::pnt(i.value(k)?);
                real(i.param_on_line(k)?);
                let (fi, theta) = i.param_on_torus(k)?;
                real(fi);
                real(theta);
            }
            real(i.param_on_line(n + 1)?);
            Ok(())
        });
        Ok(())
    });
    end();
}
