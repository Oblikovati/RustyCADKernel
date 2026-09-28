//! Points, directions, vectors, axis placements, transformations and the geometry moves;
//! the counterpart of tools/oracle/tkg2d_probe_simple.cxx.
use super::common::{Reader, attempt, ax, dir, dump, end, flag, int, pnt, real, show, trsf, vec};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dAxisPlacement, Geom2dCartesianPoint, Geom2dDirection, Geom2dGeometry, Geom2dPoint,
    Geom2dTransformation, Geom2dVector, Geom2dVectorWithMagnitude,
};
use rustycad_tkmath::gp::{Direction2, Transform2};

/// The show callback of g2_moves.
pub type Show = fn(&Handle<dyn Geom2dGeometry>);

fn shown(show: Show, result: Result<Handle<dyn Geom2dGeometry>, Failure>) -> Result<(), Failure> {
    show(&result?);
    Ok(())
}

/// The Mirror/Rotate/Scale/Translate members and their copying twins (g2_moves).
pub fn moves(g: &Handle<dyn Geom2dGeometry>, r: &mut Reader, show: Show) {
    let (p, a) = (r.pnt(), r.ax());
    let (angle, factor) = (r.real(), r.real());
    let (v, q) = (r.vec(), r.pnt());
    let t: Transform2 = trsf(r);
    attempt(|| shown(show, g.mirrored_point(p)));
    attempt(|| shown(show, g.mirrored_axis(a)));
    attempt(|| shown(show, g.rotated(p, angle)));
    attempt(|| shown(show, g.scaled(p, factor)));
    attempt(|| shown(show, g.transformed(t)));
    attempt(|| shown(show, g.translated(v)));
    attempt(|| shown(show, g.translated_between(p, q)));
    let steps: [&dyn Fn() -> Result<(), Failure>; 7] = [
        &|| g.mirror_point(p),
        &|| g.mirror_axis(a),
        &|| g.rotate(p, angle),
        &|| g.scale(p, factor),
        &|| g.translate(v),
        &|| g.translate_between(p, q),
        &|| g.transform(t),
    ];
    for step in steps {
        attempt(|| {
            step()?;
            show(g);
            Ok(())
        });
    }
}

fn show_point(g: &Handle<dyn Geom2dGeometry>) {
    let p: Handle<Geom2dCartesianPoint> = g.cast();
    let (x, y) = p.coord();
    [x, y].into_iter().for_each(real);
    pnt(p.pnt2d());
    real(p.x());
    real(p.y());
    dump(&*p, -1);
}

pub fn point_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (a, b) = (r.pnt(), r.pnt());
    let (x, y) = (r.real(), r.real());
    let p = Handle::new(Geom2dCartesianPoint::new(a.x(), a.y()));
    let o = Handle::new(Geom2dCartesianPoint::from_pnt2d(b));
    real(p.distance(&*o));
    real(p.square_distance(&*o));
    show_point(&p.copy().unwrap());
    p.set_coord(x, y);
    show_point(&p.cast());
    p.set_x(y);
    p.set_y(x);
    show_point(&p.cast());
    p.set_pnt2d(b);
    show_point(&p.cast());
    dump(&*p, 0);
    dump(&*p, 1);
    moves(&p.cast(), &mut r, show_point);
    end();
}

fn show_vector(g: &Handle<dyn Geom2dGeometry>) {
    let v: Handle<dyn Geom2dVector> = g.cast();
    let (x, y) = v.coord();
    [x, y].into_iter().for_each(real);
    vec(v.vec2d());
    [v.x(), v.y(), v.magnitude(), v.square_magnitude()]
        .into_iter()
        .for_each(real);
    dump(&*v, -1);
}

fn vector_pair(v: &dyn Geom2dVector, o: &dyn Geom2dVector) {
    show(v.angle(o), real);
    real(v.dot(o));
    real(v.crossed(o));
    show(v.reversed(), |r| show_vector(&r.cast()));
}

pub fn direction_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (x, y, a, b) = (r.real(), r.real(), r.real(), r.real());
    let other = r.vec();
    let d = match Geom2dDirection::new(x, y) {
        Ok(d) => Handle::new(d),
        Err(error) => return println!("{}", error.exception_type()),
    };
    let o = Handle::new(Geom2dVectorWithMagnitude::from_vec2d(other));
    show_vector(&d.cast());
    dir(d.dir2d().unwrap());
    vector_pair(&*d, &*o);
    real(d.crossed(&*o));
    attempt(|| d.set_coord(a, b));
    show_vector(&d.cast());
    attempt(|| d.set_x(a));
    show_vector(&d.cast());
    attempt(|| d.set_y(b));
    show_vector(&d.cast());
    d.set_dir2d(Direction2::new(1.0, 1.0).unwrap());
    show_vector(&d.cast());
    d.reverse();
    show_vector(&d.cast());
    show_vector(&d.copy().unwrap());
    moves(&d.cast(), &mut r, show_vector);
    end();
}

fn show_vwm(v: &Handle<Geom2dVectorWithMagnitude>) {
    show_vector(&v.cast());
}

pub fn vector_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (a, b) = (r.vec(), r.vec());
    let s = r.real();
    let (p, q) = (r.pnt(), r.pnt());
    let v = Handle::new(Geom2dVectorWithMagnitude::new(a.x(), a.y()));
    let o = Handle::new(Geom2dVectorWithMagnitude::from_vec2d(b));
    show_vwm(&Handle::new(Geom2dVectorWithMagnitude::between(p, q)));
    vector_pair(&*v, &*o);
    show_vwm(&v.added(&*o));
    show_vwm(&v.subtracted(&*o));
    show_vwm(&v.multiplied(s));
    show_vwm(&v.divided(s));
    show(v.normalized(), |n| show_vwm(&n));
    show_vwm(&v.added(&*o));
    show_vwm(&v.subtracted(&*o));
    show_vwm(&v.divided(s));
    real(v.crossed(&*o));
    vector_updates(&v, &o, s);
    v.set_coord(s, -s);
    show_vwm(&v);
    v.set_x(a.x());
    v.set_y(a.y());
    show_vwm(&v);
    v.set_vec2d(b);
    show_vwm(&v);
    moves(&v.cast(), &mut r, show_vector);
    end();
}

// The in-place arithmetic block of g2_vector_case, operators included.
fn vector_updates(
    v: &Handle<Geom2dVectorWithMagnitude>,
    o: &Handle<Geom2dVectorWithMagnitude>,
    s: f64,
) {
    v.add(&**o);
    show_vwm(v);
    v.subtract(&**o);
    show_vwm(v);
    v.multiply(s);
    show_vwm(v);
    v.divide(s);
    show_vwm(v);
    attempt(|| v.normalize());
    show_vwm(v);
    v.add(&**o);
    v.subtract(&**o);
    v.multiply(s);
    v.divide(s);
    show_vwm(v);
}

fn show_axis(g: &Handle<dyn Geom2dGeometry>) {
    let a: Handle<Geom2dAxisPlacement> = g.cast();
    ax(a.ax2d());
    dir(a.direction());
    pnt(a.location());
    dump(&*a, -1);
}

pub fn axis_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let first = r.ax();
    let (p, d) = (r.pnt(), r.dir());
    let other = r.ax();
    let a = Handle::new(Geom2dAxisPlacement::new(first));
    let o = Handle::new(Geom2dAxisPlacement::from_location(p, d));
    real(a.angle(&o));
    show_axis(&a.reversed().cast());
    a.reverse();
    show_axis(&a.cast());
    a.set_location(p);
    show_axis(&a.cast());
    a.set_direction(d);
    show_axis(&a.cast());
    a.set_axis(other);
    show_axis(&a.cast());
    show_axis(&a.copy().unwrap());
    moves(&a.cast(), &mut r, show_axis);
    end();
}

fn trsf_state(t: &Geom2dTransformation) {
    flag(t.is_negative());
    int(t.form() as i32);
    real(t.scale_factor());
    for row in 0..=3 {
        for col in 0..=4 {
            show(t.value(row, col), real);
        }
    }
    let (x, y) = t.transforms(1.25, -3.5);
    real(x);
    real(y);
}

fn initial_trsf(
    r: &mut Reader,
) -> (
    Handle<Geom2dTransformation>,
    i32,
    rustycad_tkmath::gp::Point2,
    f64,
) {
    let kind = r.integer();
    let (p, q) = (r.pnt(), r.pnt());
    let (a, b) = (r.ax(), r.ax());
    let s = r.real();
    let n = r.integer();
    let t = Handle::new(Geom2dTransformation::default());
    match kind {
        0 => t.set_mirror_point(p),
        1 => t.set_mirror_axis(a),
        2 => t.set_rotation(p, s),
        3 => t.set_scale(p, s),
        4 => t.set_transformation_between(a, b),
        5 => t.set_transformation_to(a),
        6 => t.set_translation(rustycad_tkmath::gp::Vector2::between(p, q)),
        7 => t.set_translation_between(p, q),
        8 => {
            let mut g = Transform2::default();
            g.set_rotation(q, s);
            t.set_trsf2d(g);
        }
        _ => {}
    }
    (t, n, q, s)
}

pub fn trsf_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (t, n, q, _) = initial_trsf(&mut r);
    trsf_state(&t);
    let o = Handle::new(Geom2dTransformation::new(t.trsf2d()));
    o.set_rotation(q, 0.75);
    show(t.inverted(), |i| trsf_state(&i));
    trsf_state(&t.multiplied(&o));
    trsf_state(&t.multiplied(&o));
    show(t.powered(n), |p| trsf_state(&p));
    trsf_state(&t.copy());
    let c = t.copy();
    c.multiply(&o);
    trsf_state(&c);
    let c = t.copy();
    c.multiply(&o);
    trsf_state(&c);
    let c = t.copy();
    c.pre_multiply(&o);
    trsf_state(&c);
    let c = t.copy();
    attempt(|| c.power(n));
    trsf_state(&c);
    let c = t.copy();
    attempt(|| c.invert());
    trsf_state(&c);
    end();
}
