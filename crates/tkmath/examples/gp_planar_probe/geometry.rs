//! gp_Dir2d/gp_Vec2d/gp_Pnt2d/gp_Ax2d/gp_Lin2d requests, including gp_Dir2d::D.
use super::{ax, dir, pair, show};
use crate::elementary_probe::{Fields, attempt, end, flag, real};
use rustycad_tkmath::gp::{Axis2d, Direction2, Line2, Point2, StandardDirection2, Vector2};

const NAMES: [StandardDirection2; 4] = [
    StandardDirection2::X,
    StandardDirection2::Y,
    StandardDirection2::NegX,
    StandardDirection2::NegY,
];
/// Angle, scale, linear tolerance and angular tolerance, as `k[0..4]` in the native probe.
#[derive(Clone, Copy)]
struct Knobs {
    angle: f64,
    scale: f64,
    linear: f64,
    angular: f64,
}
pub fn geom_case(values: &[f64]) {
    let mut fields = Fields::new(values);
    let (p, q) = (fields.point2(), fields.point2());
    let (dv, ev) = (fields.xy(), fields.xy());
    let k = Knobs {
        angle: fields.real(),
        scale: fields.real(),
        linear: fields.real(),
        angular: fields.real(),
    };
    let (la, lb, lc) = (fields.real(), fields.real(), fields.real());
    let (index, name) = (fields.int(), NAMES[fields.int() as usize]);
    dir(Direction2::from_standard(name));
    ax(Axis2d::with_standard(p, name));
    ax(Axis2d::from_standard(name));
    let e = Direction2::from(name);
    vec_block(Vector2::between(p, q), Vector2::from(dv), p, e, k);
    pnt_block(p, q, e, k);
    let mut pc = p;
    attempt(|| pc.set_coord(index, k.scale));
    pair(pc.xy());
    show(Line2::from_equation(la, lb, lc), |l| ax(l.position()));
    attempt(|| {
        let d = Direction2::from_xy(dv)?;
        direction_setters(d, ev, index);
        let e = Direction2::from_xy(ev)?;
        dir_block(d, e, p, k);
        axis_block(Axis2d::new(p, d), Axis2d::new(q, e), k);
        line_block(Line2::new(p, d), Line2::new(q, e), k);
        Ok(())
    });
    end();
}
fn direction_setters(d: Direction2, ev: rustycad_tkmath::gp::Xy, index: i32) {
    dir(d);
    let mut c = d;
    attempt(|| c.set_coord(index, ev.x()));
    dir(c);
    c = d;
    attempt(|| c.set_coordinates(ev.x(), ev.y()));
    dir(c);
    c = d;
    attempt(|| c.set_x(ev.x()));
    dir(c);
    c = d;
    attempt(|| c.set_y(ev.y()));
    dir(c);
    show(d.coord(index), real);
}
fn dir_block(d: Direction2, e: Direction2, p: Point2, k: Knobs) {
    real(d.angle(e));
    flag(d.is_equal(e, k.angular));
    flag(d.is_normal(e, k.angular));
    flag(d.is_opposite(e, k.angular));
    flag(d.is_parallel(e, k.angular));
    [d.crossed(e), d ^ e, d.dot(e), d * e]
        .into_iter()
        .for_each(real);
    dir(d.mirrored_direction(e));
    dir(d.mirrored_axis(Axis2d::new(p, e)));
    dir(d.rotated(k.angle));
    dir(d.reversed());
    dir(-d);
    let mut c = d;
    c.mirror_direction(e);
    dir(c);
    c = d;
    c.rotate(k.angle);
    dir(c);
}
fn vec_block(v: Vector2, w: Vector2, p: Point2, e: Direction2, k: Knobs) {
    real(v.magnitude());
    real(v.square_magnitude());
    show(v.angle(w), real);
    show(v.is_equal(w, k.linear, k.angular), flag);
    show(v.is_normal(w, k.angular), flag);
    show(v.is_opposite(w, k.angular), flag);
    show(v.is_parallel(w, k.angular), flag);
    [
        v.crossed(w),
        v.cross_magnitude(w),
        v.cross_square_magnitude(w),
        v.dot(w),
    ]
    .into_iter()
    .for_each(real);
    pair(v.normal().xy());
    show(v.normalized(), |n| pair(n.xy()));
    let mut c = v;
    attempt(|| c.normalize());
    pair(c.xy());
    vec_images(v, w, p, e, k);
    vec_forms(v, w, k);
}
fn vec_images(v: Vector2, w: Vector2, p: Point2, e: Direction2, k: Knobs) {
    let s = k.scale;
    [
        v.mirrored_vector(w),
        v.mirrored_axis(Axis2d::new(p, e)),
        v.rotated(k.angle),
        v.scaled(s),
        v + w,
        v - w,
        v * s,
        s * v,
        v / s,
        -v,
    ]
    .into_iter()
    .for_each(|x| pair(x.xy()));
}
fn vec_forms(v: Vector2, w: Vector2, k: Knobs) {
    let mut c = v;
    c.set_linear_form_offset(k.scale, v, k.angle, w, v);
    pair(c.xy());
    c.set_linear_form(k.scale, v, k.angle, w);
    pair(c.xy());
    c.set_scaled_sum(k.scale, v, w);
    pair(c.xy());
    c.set_sum(v, w);
    pair(c.xy());
}
fn pnt_block(p: Point2, q: Point2, e: Direction2, k: Knobs) {
    real(p.distance(q));
    real(p.square_distance(q));
    flag(p.is_equal(q, k.linear));
    [
        p.mirrored_point(q),
        p.mirrored_axis(Axis2d::new(q, e)),
        p.rotated(q, k.angle),
        p.scaled(q, k.scale),
        p.translated(Vector2::from(q.xy())),
        p.translated_between(q, p),
    ]
    .into_iter()
    .for_each(|x| pair(x.xy()));
    let mut c = p;
    c.translate_between(q, p);
    pair(c.xy());
}
fn axis_block(a: Axis2d, b: Axis2d, k: Knobs) {
    flag(a.is_coaxial(b, k.angular, k.linear));
    flag(a.is_normal(b, k.angular));
    flag(a.is_opposite(b, k.angular));
    flag(a.is_parallel(b, k.angular));
    real(a.angle(b));
    ax(a.mirrored_point(b.location()));
    ax(a.mirrored_axis(b));
    ax(a.rotated(b.location(), k.angle));
    ax(a.scaled(b.location(), k.scale));
    ax(a.translated_between(a.location(), b.location()));
}
fn line_block(l: Line2, m: Line2, k: Knobs) {
    let (la, lb, lc) = l.coefficients();
    [la, lb, lc].into_iter().for_each(real);
    let q = m.location();
    real(l.distance(q));
    real(l.square_distance(q));
    real(l.distance_line(m));
    real(l.square_distance_line(m));
    flag(l.contains(q, k.linear));
    real(l.angle(m));
    ax(l.normal(q).unwrap().position());
    ax(l.mirrored_point(q).position());
    ax(l.mirrored_axis(m.position()).position());
    ax(l.rotated(q, k.angle).position());
    ax(l.scaled(q, k.scale).position());
    ax(l.translated_between(l.location(), q).position());
    let mut c = l;
    c.translate_between(l.location(), q);
    ax(c.position());
    ax(l.reversed().position());
}
