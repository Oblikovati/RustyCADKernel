//! ElCLib requests: elclib3, elclib2, elperiod and elto3d.
use super::{
    Fields, attempt, ax1, ax2, ax22d, circ, dir, direction, end, fail, frame3, lin, pnt, pnt2,
    real, vec, vec2, vecs, vecs2,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib as el;
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis2d, Axis22d, Circle2, Circle3, Direction2, Ellipse2, Ellipse3, Hyperbola2,
    Hyperbola3, Line2, Line3, Parabola2, Parabola3, Point2, Point3, Vector2,
};

struct Shape {
    u: f64,
    r: f64,
    major: f64,
    minor: f64,
    focal: f64,
    n: i32,
}
fn shape(fields: &mut Fields) -> Shape {
    Shape {
        u: fields.real(),
        r: fields.real(),
        major: fields.real(),
        minor: fields.real(),
        focal: fields.real(),
        n: fields.int(),
    }
}

fn curve3(a: Axis2, s: &Shape, p: Point3) {
    let l = Axis1::new(a.location(), a.direction());
    let (u, n) = (s.u, s.n);
    pnt(el::line_value(u, l));
    let (q, v1) = el::line_d1(u, l);
    pnt(q);
    vec(v1);
    vec(el::line_dn(u, l, n));
    real(el::line_parameter(l, p));
    pnt(el::circle_value(u, a, s.r));
    let (q, v1) = el::circle_d1(u, a, s.r);
    pnt(q);
    vec(v1);
    let (q, v1, v2) = el::circle_d2(u, a, s.r);
    pnt(q);
    vecs(&[v1, v2]);
    let (q, v1, v2, v3) = el::circle_d3(u, a, s.r);
    pnt(q);
    vecs(&[v1, v2, v3]);
    vec(el::circle_dn(u, a, s.r, n));
    real(el::circle_parameter(a, p));
    conic3(a, s, p);
}
fn conic3(a: Axis2, s: &Shape, p: Point3) {
    let (u, n, big, small) = (s.u, s.n, s.major, s.minor);
    pnt(el::ellipse_value(u, a, big, small));
    let (q, v1) = el::ellipse_d1(u, a, big, small);
    pnt(q);
    vec(v1);
    let (q, v1, v2) = el::ellipse_d2(u, a, big, small);
    pnt(q);
    vecs(&[v1, v2]);
    let (q, v1, v2, v3) = el::ellipse_d3(u, a, big, small);
    pnt(q);
    vecs(&[v1, v2, v3]);
    vec(el::ellipse_dn(u, a, big, small, n));
    attempt(|| {
        real(el::ellipse_parameter(a, big, small, p)?);
        Ok(())
    });
    pnt(el::hyperbola_value(u, a, big, small));
    let (q, v1) = el::hyperbola_d1(u, a, big, small);
    pnt(q);
    vec(v1);
    let (q, v1, v2) = el::hyperbola_d2(u, a, big, small);
    pnt(q);
    vecs(&[v1, v2]);
    let (q, v1, v2, v3) = el::hyperbola_d3(u, a, big, small);
    pnt(q);
    vecs(&[v1, v2, v3]);
    vec(el::hyperbola_dn(u, a, big, small, n));
    real(el::hyperbola_parameter(a, big, small, p));
    pnt(el::parabola_value(u, a, s.focal));
    let (q, v1) = el::parabola_d1(u, a, s.focal);
    pnt(q);
    vec(v1);
    let (q, v1, v2) = el::parabola_d2(u, a, s.focal);
    pnt(q);
    vecs(&[v1, v2]);
    vec(el::parabola_dn(u, a, s.focal, n));
    real(el::parabola_parameter(a, p));
}
fn overloads3(a: Axis2, s: &Shape, p: Point3) {
    let (u, n) = (s.u, s.n);
    let line = Line3::new(a.location(), a.direction());
    pnt(el::value(u, &line));
    vec(el::dn(u, &line, n));
    real(el::parameter(&line, p).unwrap());
    attempt(|| {
        let c = Circle3::new(a, s.r)?;
        pnt(el::value(u, &c));
        vec(el::d3(u, &c).3);
        real(el::parameter(&c, p)?);
        Ok(())
    });
    attempt(|| {
        let c = Ellipse3::new(a, s.major, s.minor)?;
        pnt(el::value(u, &c));
        vec(el::d2(u, &c).2);
        real(el::parameter(&c, p)?);
        Ok(())
    });
    attempt(|| {
        let c = Hyperbola3::new(a, s.major, s.minor)?;
        pnt(el::value(u, &c));
        vec(el::d1(u, &c).1);
        real(el::parameter(&c, p)?);
        Ok(())
    });
    attempt(|| {
        let c = Parabola3::new(a, s.focal)?;
        pnt(el::value(u, &c));
        vec(el::d2(u, &c).2);
        vec(el::dn(u, &c, n));
        real(el::parameter(&c, p)?);
        Ok(())
    });
}
pub fn elclib3_case(values: &[f64]) {
    let mut fields = Fields::new(values);
    let (origin, normal, x) = frame3(&mut fields);
    let s = shape(&mut fields);
    let p = fields.point();
    attempt(|| {
        let a = Axis2::with_x_direction(origin, direction(normal)?, direction(x)?)?;
        ax2(a);
        curve3(a, &s, p);
        overloads3(a, &s, p);
        Ok(())
    });
    end();
}

fn curve2(a: Axis22d, l: Axis2d, s: &Shape, p: Point2) {
    let (u, n) = (s.u, s.n);
    pnt2(el::line_value_2d(u, l));
    let (q, v1) = el::line_d1_2d(u, l);
    pnt2(q);
    vec2(v1);
    vec2(el::line_dn_2d(u, l, n));
    real(el::line_parameter_2d(l, p));
    pnt2(el::circle_value_2d(u, a, s.r));
    let (q, v1) = el::circle_d1_2d(u, a, s.r);
    pnt2(q);
    vec2(v1);
    let (q, v1, v2) = el::circle_d2_2d(u, a, s.r);
    pnt2(q);
    vecs2(&[v1, v2]);
    let (q, v1, v2, v3) = el::circle_d3_2d(u, a, s.r);
    pnt2(q);
    vecs2(&[v1, v2, v3]);
    vec2(el::circle_dn_2d(u, a, s.r, n));
    attempt(|| {
        real(el::circle_parameter_2d(a, p)?);
        Ok(())
    });
    conic2(a, s, p);
}
fn conic2(a: Axis22d, s: &Shape, p: Point2) {
    let (u, n, big, small) = (s.u, s.n, s.major, s.minor);
    pnt2(el::ellipse_value_2d(u, a, big, small));
    let (q, v1) = el::ellipse_d1_2d(u, a, big, small);
    pnt2(q);
    vec2(v1);
    let (q, v1, v2) = el::ellipse_d2_2d(u, a, big, small);
    pnt2(q);
    vecs2(&[v1, v2]);
    let (q, v1, v2, v3) = el::ellipse_d3_2d(u, a, big, small);
    pnt2(q);
    vecs2(&[v1, v2, v3]);
    vec2(el::ellipse_dn_2d(u, a, big, small, n));
    attempt(|| {
        real(el::ellipse_parameter_2d(a, big, small, p)?);
        Ok(())
    });
    pnt2(el::hyperbola_value_2d(u, a, big, small));
    let (q, v1) = el::hyperbola_d1_2d(u, a, big, small);
    pnt2(q);
    vec2(v1);
    let (q, v1, v2) = el::hyperbola_d2_2d(u, a, big, small);
    pnt2(q);
    vecs2(&[v1, v2]);
    let (q, v1, v2, v3) = el::hyperbola_d3_2d(u, a, big, small);
    pnt2(q);
    vecs2(&[v1, v2, v3]);
    vec2(el::hyperbola_dn_2d(u, a, big, small, n));
    real(el::hyperbola_parameter_2d(a, big, small, p));
    pnt2(el::parabola_value_2d(u, a, s.focal));
    let (q, v1) = el::parabola_d1_2d(u, a, s.focal);
    pnt2(q);
    vec2(v1);
    let (q, v1, v2) = el::parabola_d2_2d(u, a, s.focal);
    pnt2(q);
    vecs2(&[v1, v2]);
    vec2(el::parabola_dn_2d(u, a, s.focal, n));
    real(el::parabola_parameter_2d(a, p));
}
fn overloads2(a: Axis22d, l: Axis2d, s: &Shape, p: Point2) {
    let (u, n) = (s.u, s.n);
    let line = Line2::new(l.location(), l.direction());
    pnt2(el::value(u, &line));
    vec2(el::dn(u, &line, n));
    real(el::parameter(&line, p).unwrap());
    attempt(|| {
        let c = Circle2::new(a, s.r)?;
        pnt2(el::value(u, &c));
        vec2(el::d3(u, &c).3);
        real(el::parameter(&c, p)?);
        Ok(())
    });
    attempt(|| {
        let c = Ellipse2::new(a, s.major, s.minor)?;
        pnt2(el::value(u, &c));
        vec2(el::d2(u, &c).2);
        real(el::parameter(&c, p)?);
        Ok(())
    });
    attempt(|| {
        let c = Hyperbola2::new(a, s.major, s.minor)?;
        pnt2(el::value(u, &c));
        vec2(el::d1(u, &c).1);
        real(el::parameter(&c, p)?);
        Ok(())
    });
    attempt(|| {
        let c = Parabola2::new(a, s.focal)?;
        pnt2(el::value(u, &c));
        vec2(el::d2(u, &c).2);
        vec2(el::dn(u, &c, n));
        real(el::parameter(&c, p)?);
        Ok(())
    });
}
pub fn elclib2_case(values: &[f64]) {
    let mut fields = Fields::new(values);
    let (origin, x) = (fields.point2(), fields.xy());
    let sense = fields.int() != 0;
    let s = shape(&mut fields);
    let p = fields.point2();
    attempt(|| {
        let d = Direction2::from_xy(x)?;
        let (a, l) = (Axis22d::new(origin, d, sense)?, Axis2d::new(origin, d));
        ax22d(a);
        curve2(a, l, &s, p);
        overloads2(a, l, &s, p);
        Ok(())
    });
    end();
}

pub fn elperiod_case(values: &[f64]) {
    let mut fields = Fields::new(values);
    let (u, first, last, precision) = (fields.real(), fields.real(), fields.real(), fields.real());
    let (u1, u2) = (fields.real(), fields.real());
    real(el::in_period(u, first, last));
    let (u1, u2) = el::adjust_periodic(first, last, precision, u1, u2);
    real(u1);
    real(u2);
    end();
}

struct Planar {
    frame: Axis22d,
    axis: Axis2d,
    point: Point2,
    vector: Vector2,
}
fn embed(a: Axis2, planar: &Planar, s: &Shape) {
    let (f, l) = (planar.frame, planar.axis);
    pnt(el::to3d_point(a, planar.point));
    vec(el::to3d_vector(a, planar.vector));
    attempt(|| {
        dir(el::to3d_direction(a, l.direction())?);
        Ok(())
    });
    attempt(|| {
        ax1(el::to3d_axis(a, l)?);
        Ok(())
    });
    attempt(|| {
        ax2(el::to3d_frame(a, f)?);
        Ok(())
    });
    attempt(|| {
        lin(el::to3d_line(a, Line2::new(l.location(), l.direction()))?);
        Ok(())
    });
    attempt(|| {
        circ(el::to3d_circle(a, Circle2::new(f, s.r)?)?);
        Ok(())
    });
    attempt(|| {
        let e = el::to3d_ellipse(a, Ellipse2::new(f, s.major, s.minor)?)?;
        ax2(e.position());
        real(e.major_radius());
        real(e.minor_radius());
        Ok(())
    });
    attempt(|| {
        let h = el::to3d_hyperbola(a, Hyperbola2::new(f, s.major, s.minor)?)?;
        ax2(h.position());
        real(h.major_radius());
        real(h.minor_radius());
        Ok(())
    });
    attempt(|| {
        let p = el::to3d_parabola(a, Parabola2::new(f, s.focal)?)?;
        ax2(p.position());
        real(p.focal());
        Ok(())
    });
}
pub fn elto3d_case(values: &[f64]) {
    let mut fields = Fields::new(values);
    let (origin, normal, x) = frame3(&mut fields);
    let (origin2, x2) = (fields.point2(), fields.xy());
    let sense = fields.int() != 0;
    let s = Shape {
        u: 0.0,
        r: fields.real(),
        major: fields.real(),
        minor: fields.real(),
        focal: fields.real(),
        n: 0,
    };
    let (point, vector) = (fields.point2(), fields.xy());
    let result = (|| -> Result<(), Failure> {
        let a = Axis2::with_x_direction(origin, direction(normal)?, direction(x)?)?;
        let d = Direction2::from_xy(x2)?;
        let planar = Planar {
            frame: Axis22d::new(origin2, d, sense)?,
            axis: Axis2d::new(origin2, d),
            point,
            vector: Vector2::from(vector),
        };
        embed(a, &planar, &s);
        Ok(())
    })();
    if let Err(error) = result {
        fail(&error);
    }
    end();
}
