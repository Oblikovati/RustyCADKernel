//! gb_ia_qq and gb_ia_curve: the analytic intersection and its curves.
use super::{quadric, surface};
use crate::gb_probe::common::{self as p, Reader, attempt, end, flag, int, real};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana::{Curve, IntQuadQuad};
use rustycad_tkmath::gp::{Axis3, Cone3, Cylinder3, Point3, Vector3};

/// FindParameter of `at` on the curve: the count, then the parameters.
fn find_parameter(c: &Curve, at: Point3) -> Result<(), Failure> {
    let mut params: List<f64> = List::default();
    c.find_parameter(at, &mut params)?;
    int(params.len() as i32);
    params.iter().for_each(|&value| real(value));
    Ok(())
}

/// gb_ia_curve: the flags, the domain, five points, the tangent at 37 %, a point
/// past the domain and the parameters of two points.
fn curve(c: &Curve) -> Result<(), Failure> {
    flag(c.is_open());
    flag(c.is_constant());
    flag(c.is_first_open());
    flag(c.is_last_open());
    let (f, l) = c.domain()?;
    real(f);
    real(l);
    for k in 0..=4 {
        let t = f + (l - f) * f64::from(k) / 4.0;
        p::show(c.value(t), p::pnt);
    }
    let t = f + (l - f) * 0.37;
    let (mut point, mut vector) = (Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0));
    attempt(|| {
        flag(c.d1u(t, &mut point, &mut vector)?);
        p::pnt(point);
        p::vec(vector);
        Ok(())
    });
    p::show(c.value(l + 1.0), p::pnt);
    attempt(|| find_parameter(c, c.value(t)?));
    attempt(|| find_parameter(c, c.value(f)?));
    Ok(())
}

fn links(q: &IntQuadQuad, i: i32) -> Result<(), Failure> {
    flag(q.has_next_curve(i)?);
    flag(q.has_previous_curve(i)?);
    p::show(q.next_curve(i), |(j, o)| {
        int(j);
        flag(o);
    });
    p::show(q.previous_curve(i), |(j, o)| {
        int(j);
        flag(o);
    });
    Ok(())
}

fn perform(r: &mut Reader, k: i32) -> Result<IntQuadQuad, Failure> {
    let a: Axis3 = r.ax3();
    let mut q = IntQuadQuad::new();
    if k == 0 {
        let cylinder = Cylinder3::new(a, r.real())?;
        let quad = quadric(&surface(r)?)?;
        q.perform_cylinder(cylinder, &quad, r.real())?;
    } else {
        let angle = r.real();
        let cone = Cone3::new(a, angle, r.real())?;
        let quad = quadric(&surface(r)?)?;
        q.perform_cone(cone, &quad, r.real())?;
    }
    Ok(q)
}

fn qq(r: &mut Reader) -> Result<(), Failure> {
    let k = r.integer();
    if k == 2 {
        let q = IntQuadQuad::new();
        flag(q.is_done());
        p::show(q.nb_curve(), int);
        p::show(q.has_next_curve(1), flag);
        return Ok(());
    }
    let q = perform(r, k)?;
    flag(q.is_done());
    if !q.is_done() {
        p::show(q.nb_curve(), int);
        return Ok(());
    }
    flag(q.identical_elements()?);
    let n = q.nb_curve()?;
    int(n);
    int(q.nb_pnt()?);
    for i in 1..=n {
        curve(q.curve(i)?)?;
        links(&q, i)?;
    }
    attempt(|| curve(q.curve(n + 1)?));
    p::show(q.point(1), p::pnt);
    p::show(q.has_previous_curve(0), flag);
    Ok(())
}

pub fn ia_qq_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| qq(&mut r));
    end();
}

fn curve_direct(r: &mut Reader) -> Result<(), Failure> {
    let k = r.integer();
    let a = r.ax3();
    let (cylinder, cone) = if k == 0 {
        (Some(Cylinder3::new(a, r.real())?), None)
    } else {
        let angle = r.real();
        (None, Some(Cone3::new(a, angle, r.real())?))
    };
    let mut q = [0.0; 10];
    q.iter_mut().for_each(|v| *v = r.real());
    let (tol, inf, sup) = (r.real(), r.real(), r.real());
    let (two, pos) = (r.flag(), r.flag());
    let (first, last) = (r.real(), r.real());
    let (first_open, last_open) = (r.flag(), r.flag());
    let mut c = Curve::new();
    p::show(c.domain(), |_| {});
    flag(c.is_open());
    match (cylinder, cone) {
        (Some(x), _) => c.set_cylinder_quad_values(x, q, tol, (inf, sup), two, pos),
        (_, Some(x)) => c.set_cone_quad_values(x, q, tol, (inf, sup), two, pos),
        _ => {}
    }
    curve(&c)?;
    attempt(|| {
        c.set_domain(first, last)?;
        curve(&c)
    });
    c.set_is_first_open(first_open);
    c.set_is_last_open(last_open);
    curve(&c)
}

pub fn ia_curve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| curve_direct(&mut r));
    end();
}
