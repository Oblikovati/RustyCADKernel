//! ElSLib requests: every evaluator, parameter and iso-curve on one local frame.
use super::{Fields, attempt, ax3, circ, direction, end, fail, frame3, lin, pnt, real, vec, vecs};
use rustycad_tkmath::elslib::{self as sl, CurvedSurface, ElementarySurface, SurfaceD2, SurfaceD3};
use rustycad_tkmath::gp::{Axis3, Cone3, Cylinder3, Plane3, Point3, Sphere3, Torus3};

struct Request {
    frame: Axis3,
    u: f64,
    v: f64,
    r: f64,
    a: f64,
    major: f64,
    minor: f64,
    nu: i32,
    nv: i32,
    point: Point3,
}
fn d2(value: SurfaceD2) {
    pnt(value.point);
    vecs(&[value.du, value.dv, value.duu, value.dvv, value.duv]);
}
fn d3(value: SurfaceD3) {
    d2(value.second);
    vecs(&[value.duuu, value.dvvv, value.duuv, value.duvv]);
}
fn uv((u, v): (f64, f64)) {
    real(u);
    real(v);
}
fn d1((point, du, dv): sl::SurfaceD1) {
    pnt(point);
    vecs(&[du, dv]);
}

fn plane_functions(q: &Request) {
    pnt(sl::plane_value(q.u, q.v, q.frame));
    pnt(sl::plane_d0(q.u, q.v, q.frame));
    d1(sl::plane_d1(q.u, q.v, q.frame));
    vec(sl::plane_dn(q.u, q.v, q.frame, q.nu, q.nv));
    uv(sl::plane_parameters(q.frame, q.point));
    lin(sl::plane_u_iso(q.frame, q.u));
    lin(sl::plane_v_iso(q.frame, q.v));
}
fn cylinder_functions(q: &Request) {
    pnt(sl::cylinder_value(q.u, q.v, q.frame, q.r));
    pnt(sl::cylinder_d0(q.u, q.v, q.frame, q.r));
    d1(sl::cylinder_d1(q.u, q.v, q.frame, q.r));
    d2(sl::cylinder_d2(q.u, q.v, q.frame, q.r));
    d3(sl::cylinder_d3(q.u, q.v, q.frame, q.r));
    vec(sl::cylinder_dn(q.u, q.v, q.frame, q.r, q.nu, q.nv));
    uv(sl::cylinder_parameters(q.frame, q.r, q.point));
    lin(sl::cylinder_u_iso(q.frame, q.r, q.u));
    circ(sl::cylinder_v_iso(q.frame, q.r, q.v));
}
fn cone_functions(q: &Request) {
    pnt(sl::cone_value(q.u, q.v, q.frame, q.r, q.a));
    pnt(sl::cone_d0(q.u, q.v, q.frame, q.r, q.a));
    d1(sl::cone_d1(q.u, q.v, q.frame, q.r, q.a));
    d2(sl::cone_d2(q.u, q.v, q.frame, q.r, q.a));
    d3(sl::cone_d3(q.u, q.v, q.frame, q.r, q.a));
    vec(sl::cone_dn(q.u, q.v, q.frame, q.r, q.a, q.nu, q.nv));
    uv(sl::cone_parameters(q.frame, q.r, q.a, q.point));
    lin(sl::cone_u_iso(q.frame, q.r, q.a, q.u));
    circ(sl::cone_v_iso(q.frame, q.r, q.a, q.v));
}
fn sphere_functions(q: &Request) {
    pnt(sl::sphere_value(q.u, q.v, q.frame, q.r));
    pnt(sl::sphere_d0(q.u, q.v, q.frame, q.r));
    d1(sl::sphere_d1(q.u, q.v, q.frame, q.r));
    d2(sl::sphere_d2(q.u, q.v, q.frame, q.r));
    d3(sl::sphere_d3(q.u, q.v, q.frame, q.r));
    vec(sl::sphere_dn(q.u, q.v, q.frame, q.r, q.nu, q.nv));
    uv(sl::sphere_parameters(q.frame, q.r, q.point));
    circ(sl::sphere_u_iso(q.frame, q.r, q.u));
    circ(sl::sphere_v_iso(q.frame, q.r, q.v));
}
fn torus_functions(q: &Request) {
    let (big, small) = (q.major, q.minor);
    pnt(sl::torus_value(q.u, q.v, q.frame, big, small));
    pnt(sl::torus_d0(q.u, q.v, q.frame, big, small));
    d1(sl::torus_d1(q.u, q.v, q.frame, big, small));
    d2(sl::torus_d2(q.u, q.v, q.frame, big, small));
    d3(sl::torus_d3(q.u, q.v, q.frame, big, small));
    vec(sl::torus_dn(q.u, q.v, q.frame, big, small, q.nu, q.nv));
    uv(sl::torus_parameters(q.frame, big, small, q.point));
    circ(sl::torus_u_iso(q.frame, big, small, q.u));
    circ(sl::torus_v_iso(q.frame, big, small, q.v));
}
fn surface_overloads<S: ElementarySurface>(s: &S, q: &Request) {
    pnt(sl::value(q.u, q.v, s));
    pnt(sl::d0(q.u, q.v, s));
    vec(sl::d1(q.u, q.v, s).2);
    vec(sl::dn(q.u, q.v, s, q.nu, q.nv));
    uv(sl::parameters(s, q.point));
}
fn curved_overloads<S: CurvedSurface>(s: &S, q: &Request) {
    surface_overloads(s, q);
    vec(sl::d2(q.u, q.v, s).duv);
    vec(sl::d3(q.u, q.v, s).duvv);
}
fn overload_surfaces(q: &Request) {
    surface_overloads(&Plane3::from_position(q.frame), q);
    attempt(|| {
        curved_overloads(&Cylinder3::new(q.frame, q.r)?, q);
        Ok(())
    });
    attempt(|| {
        curved_overloads(&Cone3::new(q.frame, q.a, q.r)?, q);
        Ok(())
    });
    attempt(|| {
        curved_overloads(&Sphere3::new(q.frame, q.r)?, q);
        Ok(())
    });
    attempt(|| {
        curved_overloads(&Torus3::new(q.frame, q.major, q.minor)?, q);
        Ok(())
    });
}
pub fn elslib_case(values: &[f64]) {
    let mut fields = Fields::new(values);
    let (origin, normal, x) = frame3(&mut fields);
    let indirect = fields.int() != 0;
    let (u, v, r, a) = (fields.real(), fields.real(), fields.real(), fields.real());
    let (major, minor, nu, nv) = (fields.real(), fields.real(), fields.int(), fields.int());
    let point = fields.point();
    let frame = match direction(normal).and_then(|n| Axis3::new(origin, n, direction(x)?)) {
        Ok(frame) => frame,
        Err(error) => {
            fail(&error);
            return end();
        }
    };
    let mut frame = frame;
    if indirect {
        frame.y_reverse();
    }
    let q = Request {
        frame,
        u,
        v,
        r,
        a,
        major,
        minor,
        nu,
        nv,
        point,
    };
    ax3(q.frame);
    plane_functions(&q);
    cylinder_functions(&q);
    cone_functions(&q);
    sphere_functions(&q);
    torus_functions(&q);
    overload_surfaces(&q);
    end();
}
