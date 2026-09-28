//! The Extrema elementary requests; the counterpart of
//! tools/oracle/tkgeombase_probe_extrema_el.cxx (same encoding and token order).
#![allow(clippy::unit_arg, clippy::type_complexity)]
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, real};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::extrema::{
    ElementType, ExtElC, ExtElC2d, ExtElCS, ExtElSS, ExtPElC, ExtPElC2d, ExtPElS, POnCurv,
    POnCurv2d, POnSurf, POnSurfParams,
};
use rustycad_tkmath::gp::{
    Axis3, Circle2, Circle3, Cone3, Cylinder3, Ellipse2, Ellipse3, Hyperbola2, Hyperbola3, Line2,
    Line3, Parabola2, Parabola3, Plane3, Point2, Sphere3, Torus3,
};

pub fn poncurv(p: POnCurv) {
    real(p.parameter());
    pnt(*p.value());
}
pub fn poncurv2d(p: POnCurv2d) {
    real(p.parameter());
    pnt2d(*p.value());
}
pub fn ponsurf(p: POnSurf) {
    let (u, v) = p.parameter();
    real(u);
    real(v);
    pnt(*p.value());
}

/// The accessors of an ExtPElC-like result (gb_pelc_result).
pub trait PelcView {
    type Pon: Copy;
    fn done(&self) -> bool;
    fn count(&self) -> Result<i32, Failure>;
    fn sq(&self, n: i32) -> Result<f64, Failure>;
    fn min(&self, n: i32) -> Result<bool, Failure>;
    fn pon(&self, n: i32) -> Result<Self::Pon, Failure>;
}
macro_rules! pelc_view {
    ($t:ty, $pon:ty) => {
        impl PelcView for $t {
            type Pon = $pon;
            fn done(&self) -> bool {
                self.is_done()
            }
            fn count(&self) -> Result<i32, Failure> {
                self.nb_ext()
            }
            fn sq(&self, n: i32) -> Result<f64, Failure> {
                self.square_distance(n)
            }
            fn min(&self, n: i32) -> Result<bool, Failure> {
                self.is_min(n)
            }
            fn pon(&self, n: i32) -> Result<$pon, Failure> {
                self.point(n)
            }
        }
    };
}
pelc_view!(ExtPElC, POnCurv);
pelc_view!(ExtPElC2d, POnCurv2d);

fn pelc_result<E: PelcView>(e: &E, point: fn(E::Pon)) {
    flag(e.done());
    attempt(|| {
        let n = e.count()?;
        int(n);
        for i in 1..=n {
            real(e.sq(i)?);
            flag(e.min(i)?);
            point(e.pon(i)?);
        }
        Ok(())
    });
    attempt(|| Ok(real(e.sq(0)?)));
    attempt(|| Ok(flag(e.min(5)?)));
}

enum Conic3 {
    Line(Line3),
    Circle(Circle3),
    Ellipse(Ellipse3),
    Hyperbola(Hyperbola3),
    Parabola(Parabola3),
}

fn conic3(r: &mut Reader) -> Result<Conic3, Failure> {
    let kind = r.integer();
    if kind == 0 {
        let o = r.pnt();
        return Ok(Conic3::Line(Line3::new(o, r.dir())));
    }
    let a = r.ax2();
    let x = r.real();
    Ok(match kind {
        1 => Conic3::Circle(Circle3::new(a, x)?),
        4 => Conic3::Parabola(Parabola3::new(a, x)?),
        2 => Conic3::Ellipse(Ellipse3::new(a, x, r.real())?),
        _ => Conic3::Hyperbola(Hyperbola3::new(a, x, r.real())?),
    })
}

fn pelc_step(
    c: &Conic3,
    reused: &mut ExtPElC,
    q: (rustycad_tkmath::gp::Point3, f64, f64, f64),
) -> Result<(), Failure> {
    let (pt, tol, u1, u2) = q;
    let fresh = match c {
        Conic3::Line(l) => ExtPElC::with_line(pt, *l, tol, u1, u2),
        Conic3::Circle(x) => ExtPElC::with_circle(pt, *x, tol, u1, u2)?,
        Conic3::Ellipse(x) => ExtPElC::with_ellipse(pt, *x, tol, u1, u2)?,
        Conic3::Hyperbola(x) => ExtPElC::with_hyperbola(pt, *x, tol, u1, u2)?,
        Conic3::Parabola(x) => ExtPElC::with_parabola(pt, *x, tol, u1, u2)?,
    };
    pelc_result(&fresh, poncurv);
    match c {
        Conic3::Line(l) => reused.perform_line(pt, l, tol, u1, u2),
        Conic3::Circle(x) => reused.perform_circle(pt, x, tol, u1, u2)?,
        Conic3::Ellipse(x) => reused.perform_ellipse(pt, x, tol, u1, u2)?,
        Conic3::Hyperbola(x) => reused.perform_hyperbola(pt, x, tol, u1, u2)?,
        Conic3::Parabola(x) => reused.perform_parabola(pt, x, tol, u1, u2)?,
    }
    Ok(())
}

pub fn pelc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let c = conic3(&mut r)?;
        let n = r.integer();
        let mut reused = ExtPElC::new();
        pelc_result(&reused, poncurv);
        for _ in 0..n {
            let pt = r.pnt();
            let (tol, u1, u2) = (r.real(), r.real(), r.real());
            attempt(|| pelc_step(&c, &mut reused, (pt, tol, u1, u2)));
            pelc_result(&reused, poncurv);
        }
        Ok(())
    });
    end();
}

enum Conic2 {
    Line(Line2),
    Circle(Circle2),
    Ellipse(Ellipse2),
    Hyperbola(Hyperbola2),
    Parabola(Parabola2),
}

fn conic2(r: &mut Reader) -> Result<Conic2, Failure> {
    let kind = r.integer();
    if kind == 0 {
        let o = r.pnt2d();
        return Ok(Conic2::Line(Line2::new(o, r.dir2d())));
    }
    let a = r.ax22d();
    let x = r.real();
    Ok(match kind {
        1 => Conic2::Circle(Circle2::new(a, x)?),
        4 => Conic2::Parabola(Parabola2::new(a, x)?),
        2 => Conic2::Ellipse(Ellipse2::new(a, x, r.real())?),
        _ => Conic2::Hyperbola(Hyperbola2::new(a, x, r.real())?),
    })
}

fn pelc2d_step(
    c: &Conic2,
    reused: &mut ExtPElC2d,
    q: (Point2, f64, f64, f64),
) -> Result<(), Failure> {
    let (pt, tol, u1, u2) = q;
    let fresh = match c {
        Conic2::Line(l) => ExtPElC2d::with_line(pt, *l, tol, u1, u2),
        Conic2::Circle(x) => ExtPElC2d::with_circle(pt, *x, tol, u1, u2)?,
        Conic2::Ellipse(x) => ExtPElC2d::with_ellipse(pt, *x, tol, u1, u2)?,
        Conic2::Hyperbola(x) => ExtPElC2d::with_hyperbola(pt, *x, tol, u1, u2)?,
        Conic2::Parabola(x) => ExtPElC2d::with_parabola(pt, *x, tol, u1, u2)?,
    };
    pelc_result(&fresh, poncurv2d);
    match c {
        Conic2::Line(l) => reused.perform_line(pt, l, tol, u1, u2),
        Conic2::Circle(x) => reused.perform_circle(pt, x, tol, u1, u2)?,
        Conic2::Ellipse(x) => reused.perform_ellipse(pt, x, tol, u1, u2)?,
        Conic2::Hyperbola(x) => reused.perform_hyperbola(pt, x, tol, u1, u2)?,
        Conic2::Parabola(x) => reused.perform_parabola(pt, x, tol, u1, u2)?,
    }
    Ok(())
}

pub fn pelc2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let c = conic2(&mut r)?;
        let n = r.integer();
        let mut reused = ExtPElC2d::new();
        pelc_result(&reused, poncurv2d);
        for _ in 0..n {
            let pt = r.pnt2d();
            let (tol, u1, u2) = (r.real(), r.real(), r.real());
            attempt(|| pelc2d_step(&c, &mut reused, (pt, tol, u1, u2)));
            pelc_result(&reused, poncurv2d);
        }
        Ok(())
    });
    end();
}

fn elc_result(e: &ExtElC) {
    flag(e.is_done());
    attempt(|| Ok(flag(e.is_parallel()?)));
    attempt(|| {
        let n = e.nb_ext()?;
        int(n);
        for i in 1..=n {
            real(e.square_distance(i)?);
            let (a, b) = e.points(i)?;
            poncurv(a);
            poncurv(b);
        }
        Ok(())
    });
    attempt(|| Ok(real(e.square_distance(0)?)));
    attempt(|| Ok(real(e.square_distance(9)?)));
}

fn elc2d_result(e: &ExtElC2d) {
    flag(e.is_done());
    attempt(|| Ok(flag(e.is_parallel()?)));
    attempt(|| {
        let n = e.nb_ext()?;
        int(n);
        for i in 1..=n {
            real(e.square_distance(i)?);
            let (a, b) = e.points(i)?;
            poncurv2d(a);
            poncurv2d(b);
        }
        Ok(())
    });
    attempt(|| Ok(real(e.square_distance(0)?)));
    attempt(|| Ok(real(e.square_distance(9)?)));
}

fn read_circ(r: &mut Reader) -> Result<Circle3, Failure> {
    let a = r.ax2();
    Circle3::new(a, r.real())
}
fn read_lin(r: &mut Reader) -> Line3 {
    let o = r.pnt();
    Line3::new(o, r.dir())
}
fn read_circ2d(r: &mut Reader) -> Result<Circle2, Failure> {
    let a = r.ax22d();
    Circle2::new(a, r.real())
}
fn read_lin2d(r: &mut Reader) -> Line2 {
    let o = r.pnt2d();
    Line2::new(o, r.dir2d())
}

fn elc(r: &mut Reader) -> Result<(), Failure> {
    let pair = r.integer();
    if pair == 5 {
        let a = read_circ(r)?;
        let b = read_circ(r)?;
        elc_result(&ExtElC::circles(a, b)?);
        return Ok(());
    }
    let l = read_lin(r);
    let a = r.ax2();
    let x1 = r.real();
    let e = match pair {
        0 => ExtElC::lines(l, Line3::new(a.location(), a.direction()), x1),
        1 => {
            let tol = r.real();
            ExtElC::line_circle(l, Circle3::new(a, x1)?, tol)?
        }
        2 => ExtElC::line_ellipse(l, Ellipse3::new(a, x1, r.real())?)?,
        3 => ExtElC::line_hyperbola(l, Hyperbola3::new(a, x1, r.real())?)?,
        _ => ExtElC::line_parabola(l, Parabola3::new(a, x1)?)?,
    };
    elc_result(&e);
    Ok(())
}

pub fn elc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| elc(&mut r));
    end();
}

fn elc2d(r: &mut Reader) -> Result<(), Failure> {
    let pair = r.integer();
    if pair == 0 {
        let a = read_lin2d(r);
        let b = read_lin2d(r);
        elc2d_result(&ExtElC2d::lines(a, b, r.real())?);
        return Ok(());
    }
    if pair <= 4 {
        let l = read_lin2d(r);
        let a = r.ax22d();
        let x1 = r.real();
        let e = match pair {
            1 => ExtElC2d::line_circle(l, Circle2::new(a, x1)?, r.real())?,
            2 => ExtElC2d::line_ellipse(l, Ellipse2::new(a, x1, r.real())?)?,
            3 => ExtElC2d::line_hyperbola(l, Hyperbola2::new(a, x1, r.real())?)?,
            _ => ExtElC2d::line_parabola(l, Parabola2::new(a, x1)?)?,
        };
        elc2d_result(&e);
        return Ok(());
    }
    let c = read_circ2d(r)?;
    let a = r.ax22d();
    let x1 = r.real();
    let e = match pair {
        5 => ExtElC2d::circles(c, Circle2::new(a, x1)?)?,
        6 => ExtElC2d::circle_ellipse(c, Ellipse2::new(a, x1, r.real())?)?,
        7 => ExtElC2d::circle_hyperbola(c, Hyperbola2::new(a, x1, r.real())?)?,
        _ => ExtElC2d::circle_parabola(c, Parabola2::new(a, x1)?)?,
    };
    elc2d_result(&e);
    Ok(())
}

pub fn elc2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| elc2d(&mut r));
    end();
}

enum Surface {
    Plane(Plane3),
    Cylinder(Cylinder3),
    Cone(Cone3),
    Sphere(Sphere3),
    Torus(Torus3),
}

fn surface(kind: i32, a: Axis3, r: &mut Reader) -> Result<Surface, Failure> {
    Ok(match kind {
        1 => Surface::Cylinder(Cylinder3::new(a, r.real())?),
        2 => {
            let ang = r.real();
            Surface::Cone(Cone3::new(a, ang, r.real())?)
        }
        3 => Surface::Sphere(Sphere3::new(a, r.real())?),
        4 => {
            let big = r.real();
            Surface::Torus(Torus3::new(a, big, r.real())?)
        }
        _ => Surface::Plane(Plane3::from_position(a)),
    })
}

fn pels_show(e: &ExtPElS) {
    flag(e.is_done());
    attempt(|| {
        let n = e.nb_ext()?;
        int(n);
        for i in 1..=n {
            real(e.square_distance(i)?);
            ponsurf(e.point(i)?);
        }
        Ok(())
    });
    attempt(|| Ok(real(e.square_distance(0)?)));
    attempt(|| Ok(ponsurf(e.point(5)?)));
}

fn pels_step(
    s: &Surface,
    reused: &mut ExtPElS,
    pt: rustycad_tkmath::gp::Point3,
    tol: f64,
) -> Result<(), Failure> {
    match s {
        Surface::Plane(x) => {
            pels_show(&ExtPElS::with_plane(pt, *x, tol));
            reused.perform_plane(pt, x, tol);
        }
        Surface::Cylinder(x) => {
            pels_show(&ExtPElS::with_cylinder(pt, *x, tol)?);
            reused.perform_cylinder(pt, x, tol)?;
        }
        Surface::Cone(x) => {
            pels_show(&ExtPElS::with_cone(pt, *x, tol)?);
            reused.perform_cone(pt, x, tol)?;
        }
        Surface::Sphere(x) => {
            pels_show(&ExtPElS::with_sphere(pt, *x, tol)?);
            reused.perform_sphere(pt, x, tol)?;
        }
        Surface::Torus(x) => {
            pels_show(&ExtPElS::with_torus(pt, *x, tol)?);
            reused.perform_torus(pt, x, tol)?;
        }
    }
    Ok(())
}

pub fn pels_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let kind = r.integer();
        let a = r.ax3();
        let s = surface(kind, a, &mut r)?;
        let n = r.integer();
        let mut reused = ExtPElS::new();
        pels_show(&reused);
        for _ in 0..n {
            let pt = r.pnt();
            let tol = r.real();
            attempt(|| pels_step(&s, &mut reused, pt, tol));
            pels_show(&reused);
        }
        Ok(())
    });
    end();
}

fn elcs_perform(e: &mut ExtElCS, curve: &Conic3, s: &Surface) -> Result<(), Failure> {
    match (curve, s) {
        (Conic3::Line(l), Surface::Plane(p)) => e.perform_line_plane(l, p),
        (Conic3::Line(l), Surface::Cylinder(p)) => e.perform_line_cylinder(l, p)?,
        (Conic3::Line(l), Surface::Cone(p)) => e.perform_line_cone(l, p)?,
        (Conic3::Line(l), Surface::Sphere(p)) => e.perform_line_sphere(l, p)?,
        (Conic3::Line(l), Surface::Torus(p)) => e.perform_line_torus(l, p)?,
        (Conic3::Circle(c), Surface::Plane(p)) => e.perform_circle_plane(c, p)?,
        (Conic3::Circle(c), Surface::Cylinder(p)) => e.perform_circle_cylinder(c, p)?,
        (Conic3::Circle(c), Surface::Cone(p)) => e.perform_circle_cone(c, p)?,
        (Conic3::Circle(c), Surface::Sphere(p)) => e.perform_circle_sphere(c, p)?,
        (Conic3::Circle(c), Surface::Torus(p)) => e.perform_circle_torus(c, p)?,
        (Conic3::Hyperbola(h), Surface::Plane(p)) => e.perform_hyperbola_plane(h, p),
        _ => {}
    }
    Ok(())
}

fn elcs_show(e: &ExtElCS, allocated: &mut bool) {
    flag(e.is_done());
    let mut par = false;
    attempt(|| {
        par = e.is_parallel()?;
        flag(par);
        Ok(())
    });
    attempt(|| {
        let m = e.nb_ext()?;
        int(m);
        if !par {
            for i in 1..=m {
                real(e.square_distance(i)?);
                let (a, b) = e.points(i)?;
                poncurv(a);
                ponsurf(b);
            }
            if m > 0 {
                *allocated = true;
            }
        } else {
            real(e.square_distance(1)?);
            if *allocated {
                let (a, b) = e.points(1)?;
                poncurv(a);
                ponsurf(b);
            }
        }
        Ok(())
    });
    attempt(|| Ok(real(e.square_distance(0)?)));
}

fn elcs_curve(curve: i32, r: &mut Reader) -> Result<Conic3, Failure> {
    Ok(match curve {
        0 => Conic3::Line(read_lin(r)),
        1 => Conic3::Circle(read_circ(r)?),
        _ => {
            let a = r.ax2();
            let x = r.real();
            Conic3::Hyperbola(Hyperbola3::new(a, x, r.real())?)
        }
    })
}

pub fn elcs_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let n = r.integer();
        let mut e = ExtElCS::new();
        let mut allocated = false;
        for _ in 0..n {
            let kind = r.integer();
            let c = elcs_curve(kind / 5, &mut r)?;
            let a = r.ax3();
            let s = surface(kind % 5, a, &mut r)?;
            attempt(|| elcs_perform(&mut e, &c, &s));
            elcs_show(&e, &mut allocated);
        }
        Ok(())
    });
    end();
}

fn elss_step(e: &mut ExtElSS, kind: i32, r: &mut Reader) -> Result<(), Failure> {
    let a = r.ax3();
    if kind == 0 {
        let b = r.ax3();
        e.perform_planes(&Plane3::from_position(a), &Plane3::from_position(b));
        return Ok(());
    }
    if kind == 1 {
        let b = r.ax3();
        let s = Sphere3::new(b, r.real())?;
        attempt(|| e.perform_plane_sphere(&Plane3::from_position(a), &s));
        return Ok(());
    }
    let s = Sphere3::new(a, r.real())?;
    let b = r.ax3();
    let x = r.real();
    match kind {
        2 => {
            let t = Sphere3::new(b, x)?;
            attempt(|| e.perform_spheres(&s, &t));
        }
        3 => {
            let t = Cylinder3::new(b, x)?;
            attempt(|| e.perform_sphere_cylinder(&s, &t));
        }
        4 => {
            let t = Cone3::new(b, x, r.real())?;
            attempt(|| e.perform_sphere_cone(&s, &t));
        }
        _ => {
            let t = Torus3::new(b, x, r.real())?;
            attempt(|| e.perform_sphere_torus(&s, &t));
        }
    }
    Ok(())
}

pub fn elss_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let n = r.integer();
        let mut e = ExtElSS::new();
        for _ in 0..n {
            let kind = r.integer();
            elss_step(&mut e, kind, &mut r)?;
            flag(e.is_done());
            attempt(|| Ok(flag(e.is_parallel()?)));
            attempt(|| {
                let m = e.nb_ext()?;
                int(m);
                for i in 1..=m {
                    real(e.square_distance(i)?);
                }
                Ok(())
            });
            attempt(|| Ok(real(e.square_distance(0)?)));
        }
        Ok(())
    });
    end();
}

pub fn pon_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (u, v) = (r.real(), r.real());
    let p = r.pnt();
    let sq = r.real();
    let (kind, iu, iv) = (r.integer(), r.integer(), r.integer());
    let mut c = POnCurv::default();
    poncurv(c);
    c.set_values(u, p);
    poncurv(c);
    poncurv(POnCurv::new(v, p));
    let mut c2 = POnCurv2d::default();
    poncurv2d(c2);
    c2.set_values(u, Point2::new(p.x(), p.y()));
    poncurv2d(c2);
    let mut s = POnSurf::default();
    ponsurf(s);
    s.set_parameters(u, v, p);
    ponsurf(s);
    ponsurf(POnSurf::new(v, u, p));
    let sp = POnSurfParams::default();
    ponsurf(*sp);
    real(sp.get_sqr_distance());
    int(sp.get_element_type() as i32);
    let (i1, i2) = sp.get_indices();
    int(i1);
    int(i2);
    let mut sp2 = POnSurfParams::new(u, v, p);
    sp2.set_sqr_distance(sq);
    let types = [
        ElementType::Node,
        ElementType::UIsoEdge,
        ElementType::VIsoEdge,
        ElementType::Face,
    ];
    sp2.set_element_type(types[kind as usize]);
    sp2.set_indices(iu, iv);
    ponsurf(*sp2);
    real(sp2.get_sqr_distance());
    int(sp2.get_element_type() as i32);
    let (i1, i2) = sp2.get_indices();
    int(i1);
    int(i2);
    end();
}
