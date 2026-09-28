//! Points, directions, vectors, axis placements, transformations and the
//! Geom_Geometry moves; the counterpart of tools/oracle/tkg3d_probe_simple.cxx.
use super::common::{
    Reader, attempt, ax1, ax2, dir, dump, end, flag, int, pnt, real, show, trsf, vec,
};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomAxis1Placement, GeomAxis2Placement, GeomAxisPlacement, GeomCartesianPoint, GeomDirection,
    GeomGeometry, GeomPoint, GeomTransformation, GeomVector, GeomVectorWithMagnitude,
};
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Point3};

/// How a probe prints one geometry after each move.
pub type Show = fn(&Handle<dyn GeomGeometry>);

/// The Mirror/Rotate/Scale/Translate members and their copying twins (g3_moves).
pub fn moves(g: &Handle<dyn GeomGeometry>, r: &mut Reader, show_geometry: Show) {
    let (p, a, f) = (r.pnt(), r.ax1(), r.ax2());
    let (angle, factor) = (r.real(), r.real());
    let (v, q) = (r.vec(), r.pnt());
    let t = trsf(r);
    let copies: [&dyn Fn() -> Result<Handle<dyn GeomGeometry>, Failure>; 8] = [
        &|| g.mirrored_point(p),
        &|| g.mirrored_axis(a),
        &|| g.mirrored_plane(f),
        &|| g.rotated(a, angle),
        &|| g.scaled(p, factor),
        &|| g.transformed(t),
        &|| g.translated(v),
        &|| g.translated_between(p, q),
    ];
    for copy in copies {
        show(copy(), |c| show_geometry(&c));
    }
    let steps: [&dyn Fn() -> Result<(), Failure>; 8] = [
        &|| g.mirror_point(p),
        &|| g.mirror_axis(a),
        &|| g.mirror_plane(f),
        &|| g.rotate(a, angle),
        &|| g.scale(p, factor),
        &|| g.translate(v),
        &|| g.translate_between(p, q),
        &|| g.transform(t),
    ];
    for step in steps {
        show(step(), |()| show_geometry(g));
    }
}

fn show_point(g: &Handle<dyn GeomGeometry>) {
    let p: Handle<GeomCartesianPoint> = g.cast();
    let (x, y, z) = p.coord();
    [x, y, z].into_iter().for_each(real);
    pnt(p.pnt());
    [p.x(), p.y(), p.z()].into_iter().for_each(real);
    dump(&*p, -1);
}

pub fn point_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (a, b) = (r.pnt(), r.pnt());
    let (x, y, z) = (r.real(), r.real(), r.real());
    let p = Handle::new(GeomCartesianPoint::new(a.x(), a.y(), a.z()));
    let o = Handle::new(GeomCartesianPoint::from_pnt(b));
    real(p.distance(&*o));
    real(p.square_distance(&*o));
    show_point(&p.copy().unwrap());
    p.set_coord(x, y, z);
    show_point(&p.cast());
    p.set_x(y);
    p.set_y(z);
    p.set_z(x);
    show_point(&p.cast());
    p.set_pnt(b);
    show_point(&p.cast());
    dump(&*p, 0);
    dump(&*p, 1);
    moves(&p.cast(), &mut r, show_point);
    end();
}

fn show_vector(g: &Handle<dyn GeomGeometry>) {
    let v: Handle<dyn GeomVector> = g.cast();
    let (x, y, z) = v.coord();
    [x, y, z].into_iter().for_each(real);
    vec(v.vec());
    [v.x(), v.y(), v.z(), v.magnitude(), v.square_magnitude()]
        .into_iter()
        .for_each(real);
    dump(&*v, -1);
}

fn vector_pair(v: &dyn GeomVector, o: &dyn GeomVector, reference: &dyn GeomVector) {
    show(v.angle(o), real);
    show(v.angle_with_ref(o, reference), real);
    real(v.dot(o));
    real(v.dot_cross(o, reference));
    show(v.crossed(o), |c| show_vector(&c.cast()));
    show(v.cross_crossed(o, reference), |c| show_vector(&c.cast()));
    show(v.reversed(), |c| show_vector(&c.cast()));
    show(v.copy(), |c| show_vector(&c));
}

fn vector_cross(v: &dyn GeomVector, o: &dyn GeomVector, reference: &dyn GeomVector) {
    let c: Handle<dyn GeomVector> = v.copy().unwrap().cast();
    attempt(|| c.cross(o));
    show_vector(&c.cast());
    let c: Handle<dyn GeomVector> = v.copy().unwrap().cast();
    attempt(|| c.cross_cross(o, reference));
    show_vector(&c.cast());
    let c: Handle<dyn GeomVector> = v.copy().unwrap().cast();
    attempt(|| c.cross(&*c));
    show_vector(&c.cast());
}

pub fn direction_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (x, y, z) = (r.real(), r.real(), r.real());
    let (a, b, c) = (r.real(), r.real(), r.real());
    let (other, reference) = (r.vec(), r.vec());
    let d = match GeomDirection::new(x, y, z) {
        Ok(d) => Handle::new(d),
        Err(error) => {
            println!("{}", error.exception_type());
            return;
        }
    };
    let o = GeomVectorWithMagnitude::from_vec(other);
    let f = GeomVectorWithMagnitude::from_vec(reference);
    show_vector(&d.cast());
    dir(d.dir().unwrap());
    vector_pair(&*d, &o, &f);
    vector_cross(&*d, &o, &f);
    attempt(|| d.set_coord(a, b, c));
    show_vector(&d.cast());
    attempt(|| d.set_x(a));
    show_vector(&d.cast());
    attempt(|| d.set_y(b));
    show_vector(&d.cast());
    attempt(|| d.set_z(c));
    show_vector(&d.cast());
    d.set_dir(Direction3::new(1.0, 1.0, 1.0).unwrap());
    show_vector(&d.cast());
    d.reverse();
    show_vector(&d.cast());
    moves(&d.cast(), &mut r, show_vector);
    end();
}

pub fn vector_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (a, b, reference) = (r.vec(), r.vec(), r.vec());
    let s = r.real();
    let (p, q) = (r.pnt(), r.pnt());
    let v = Handle::new(GeomVectorWithMagnitude::new(a.x(), a.y(), a.z()));
    let o = GeomVectorWithMagnitude::from_vec(b);
    let f = GeomVectorWithMagnitude::from_vec(reference);
    show_vector(&Handle::new(GeomVectorWithMagnitude::between(p, q)).cast());
    vector_pair(&*v, &o, &f);
    vector_cross(&*v, &o, &f);
    show_vector(&v.added(&o).cast());
    show_vector(&v.subtracted(&o).cast());
    show_vector(&v.multiplied(s).cast());
    show_vector(&v.divided(s).cast());
    show(v.normalized(), |n| show_vector(&n.cast()));
    vector_in_place(&v, &o, s);
    v.set_coord(s, -s, 2.0 * s);
    show_vector(&v.cast());
    v.set_x(a.x());
    v.set_y(a.y());
    v.set_z(a.z());
    show_vector(&v.cast());
    v.set_vec(b);
    show_vector(&v.cast());
    moves(&v.cast(), &mut r, show_vector);
    end();
}

fn vector_in_place(v: &Handle<GeomVectorWithMagnitude>, o: &GeomVectorWithMagnitude, s: f64) {
    v.add(o);
    show_vector(&v.cast());
    v.subtract(o);
    show_vector(&v.cast());
    v.multiply(s);
    show_vector(&v.cast());
    v.divide(s);
    show_vector(&v.cast());
    attempt(|| v.normalize());
    show_vector(&v.cast());
}

fn show_axis1(g: &Handle<dyn GeomGeometry>) {
    let a: Handle<GeomAxis1Placement> = g.cast();
    ax1(a.ax1());
    ax1(a.axis());
    dir(a.direction());
    pnt(a.location());
    dump(&*a, -1);
}

pub fn axis1_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (a, b, d, p) = (r.ax1(), r.ax1(), r.dir(), r.pnt());
    let x = Handle::new(GeomAxis1Placement::new(a));
    let y = GeomAxis1Placement::from_location(b.location(), b.direction());
    show_axis1(&x.cast());
    real(x.angle(&y));
    show_axis1(&x.reversed().cast());
    show_axis1(&x.copy().unwrap());
    x.reverse();
    show_axis1(&x.cast());
    x.set_direction(d).unwrap();
    show_axis1(&x.cast());
    x.set_location(p);
    show_axis1(&x.cast());
    x.set_axis(b);
    show_axis1(&x.cast());
    moves(&x.cast(), &mut r, show_axis1);
    end();
}

fn show_axis2(g: &Handle<dyn GeomGeometry>) {
    let a: Handle<GeomAxis2Placement> = g.cast();
    show(a.ax2(), ax2);
    ax1(a.axis());
    dir(a.x_direction());
    dir(a.y_direction());
    dump(&*a, -1);
}

pub fn axis2_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (f, p) = (r.ax2(), r.pnt());
    let (n, vx, d, e) = (r.dir(), r.dir(), r.dir(), r.dir());
    let a = r.ax1();
    let x = Handle::new(GeomAxis2Placement::new(f));
    let y = match GeomAxis2Placement::from_directions(p, n, vx) {
        Ok(y) => Handle::new(y),
        Err(error) => {
            print!("{} ", error.exception_type());
            x.clone()
        }
    };
    show_axis2(&x.cast());
    show_axis2(&y.cast());
    real(x.angle(&*y));
    show_axis2(&y.copy().unwrap());
    attempt(|| x.set_direction(d));
    show_axis2(&x.cast());
    attempt(|| x.set_x_direction(e));
    show_axis2(&x.cast());
    attempt(|| x.set_y_direction(d));
    show_axis2(&x.cast());
    x.set_location(p);
    show_axis2(&x.cast());
    x.set_axis(a);
    show_axis2(&x.cast());
    x.set_ax2(f);
    show_axis2(&x.cast());
    moves(&x.cast(), &mut r, show_axis2);
    end();
}

fn show_trsf(t: &GeomTransformation) {
    flag(t.is_negative());
    int(t.form() as i32);
    real(t.scale_factor());
    for i in 1..=3 {
        for j in 1..=4 {
            real(t.value(i, j).unwrap());
        }
    }
    show(t.value(0, 1), real);
    let (x, y, z) = t.transforms(1.5, -2., 0.25);
    [x, y, z].into_iter().for_each(real);
    trsf_dump(t, -1);
}

fn trsf_dump(t: &GeomTransformation, depth: i32) {
    let mut stream = DumpStream::default();
    t.dump_json(&mut stream, depth).unwrap();
    print!("{{{}}} ", stream.text());
}

pub fn trsf_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (a, b) = (trsf(&mut r), trsf(&mut r));
    let n = r.integer();
    let (f, h, p, q) = (r.ax3(), r.ax3(), r.pnt(), r.pnt());
    let (ax, pl) = (r.ax1(), r.ax2());
    let (angle, factor, v) = (r.real(), r.real(), r.vec());
    let t = GeomTransformation::new(a);
    let o = GeomTransformation::new(b);
    show_trsf(&GeomTransformation::default());
    show_trsf(&t);
    show_trsf(&t.copy());
    show_trsf(&t.multiplied(&o));
    show(t.inverted(), |i| show_trsf(&i));
    show(t.powered(n), |i| show_trsf(&i));
    let c: Handle<GeomTransformation> = t.copy();
    c.multiply(&o);
    show_trsf(&c);
    c.pre_multiply(&o);
    show_trsf(&c);
    attempt(|| c.power(n));
    show_trsf(&c);
    attempt(|| c.invert());
    show_trsf(&c);
    trsf_setters(&c, (p, q), (ax, pl), (angle, factor));
    c.set_transformation_between(f, h);
    show_trsf(&c);
    c.set_transformation_to(f);
    show_trsf(&c);
    c.set_translation(v);
    show_trsf(&c);
    c.set_translation_between(p, q);
    show_trsf(&c);
    c.set_trsf(a);
    show_trsf(&c);
    trsf_dump(&t, 0);
    end();
}

type Axes = (Axis1, Axis2);

fn trsf_setters(c: &GeomTransformation, points: (Point3, Point3), axes: Axes, values: (f64, f64)) {
    let (p, _) = points;
    let (ax, pl) = axes;
    c.set_mirror_point(p);
    show_trsf(c);
    c.set_mirror_axis(ax);
    show_trsf(c);
    c.set_mirror_plane(pl);
    show_trsf(c);
    attempt(|| c.set_rotation(ax, values.0));
    show_trsf(c);
    attempt(|| c.set_scale(p, values.1));
    show_trsf(c);
}
