//! gb_ia_qqgeo: the geometric quadric-quadric intersection.
use super::{Surface, surface};
use crate::gb_probe::common::{self as p, Reader, attempt, end, flag, int};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana::{QuadQuadGeo, ResultType};

fn solution(q: &QuadQuadGeo, i: i32) -> Result<(), Failure> {
    match q.type_inter()? {
        ResultType::Point => p::show(q.point(i), p::pnt),
        ResultType::Line => p::show(q.line(i), p::lin),
        ResultType::Circle => p::show(q.circle(i), p::circ),
        ResultType::PointAndCircle => {
            p::show(q.point(1), p::pnt);
            p::show(q.circle(1), p::circ);
        }
        ResultType::Ellipse => p::show(q.ellipse(i), p::elips),
        ResultType::Parabola => p::show(q.parabola(i), p::parab),
        ResultType::Hyperbola => p::show(q.hyperbola(i), p::hypr),
        _ => {}
    }
    Ok(())
}

fn qqgeo(q: &QuadQuadGeo) -> Result<(), Failure> {
    flag(q.is_done());
    flag(q.has_common_gen());
    p::pnt(q.p_char());
    if !q.is_done() {
        p::show(q.type_inter(), |t| int(t as i32));
        return Ok(());
    }
    int(q.type_inter()? as i32);
    let n = q.nb_solutions()?;
    int(n);
    for i in 1..=n {
        solution(q, i)?;
    }
    p::show(q.point(1), p::pnt);
    p::show(q.line(n + 1), p::lin);
    p::show(q.circle(n + 1), p::circ);
    p::show(q.ellipse(0), p::elips);
    p::show(q.parabola(2), p::parab);
    p::show(q.hyperbola(n + 1), p::hypr);
    Ok(())
}

/// The overload of the pair of surfaces, with its tolerances read in order.
fn perform(q: &mut QuadQuadGeo, a: Surface, b: Surface, r: &mut Reader) -> Result<(), Failure> {
    use Surface::*;
    match (a, b) {
        (Plane(x), Plane(y)) => {
            let ta = r.real();
            q.perform_planes(x, y, ta, r.real())
        }
        (Plane(x), Cylinder(y)) => {
            let (ta, t) = (r.real(), r.real());
            q.perform_plane_cylinder(x, y, ta, t, r.real())
        }
        (Plane(x), Sphere(y)) => q.perform_plane_sphere(x, y),
        (Plane(x), Cone(y)) => {
            let ta = r.real();
            q.perform_plane_cone(x, y, ta, r.real())
        }
        (a, b) => perform_with_tol(q, a, b, r.real()),
    }
}

fn perform_with_tol(q: &mut QuadQuadGeo, a: Surface, b: Surface, t: f64) -> Result<(), Failure> {
    use Surface::*;
    match (a, b) {
        (Cylinder(x), Cylinder(y)) => q.perform_cylinders(x, y, t),
        (Cylinder(x), Sphere(y)) => q.perform_cylinder_sphere(x, y, t),
        (Cylinder(x), Cone(y)) => q.perform_cylinder_cone(x, y, t),
        (Sphere(x), Sphere(y)) => q.perform_spheres(x, y, t),
        (Sphere(x), Cone(y)) => q.perform_sphere_cone(x, y, t),
        (Cone(x), Cone(y)) => q.perform_cones(x, y, t),
        (Plane(x), Torus(y)) => q.perform_plane_torus(x, y, t),
        (Cylinder(x), Torus(y)) => q.perform_cylinder_torus(x, y, t),
        (Cone(x), Torus(y)) => q.perform_cone_torus(x, y, t),
        (Sphere(x), Torus(y)) => q.perform_sphere_torus(x, y, t),
        (Torus(x), Torus(y)) => q.perform_tori(x, y, t),
        _ => Ok(()),
    }
}

pub fn ia_qqgeo_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let a = surface(&mut r)?;
        let b = surface(&mut r)?;
        let mut q = QuadQuadGeo::new();
        perform(&mut q, a, b, &mut r)?;
        qqgeo(&q)
    });
    end();
}
