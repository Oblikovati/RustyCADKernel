//! Curve evaluation, curve transformations, lines and conics; the counterpart of
//! tools/oracle/tkg3d_probe_curves.cxx.
use super::common::{
    Reader, attempt, ax1, ax2, built_curve, curve_summary, dump, end, name, pnt, real, show, trsf,
};
use super::simple::moves;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCircle, GeomConic, GeomCurve, GeomGeometry, GeomLine};
use rustycad_tkmath::gp::{Axis2, Circle3, Line3};
use std::cell::RefCell;

pub fn curve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c) = built_curve(&mut r) else { return };
    let u = r.reals();
    name(&c);
    curve_summary(&*c, &u);
    for depth in [-1, 1, 0] {
        dump(&*c, depth);
    }
    end();
}

thread_local! {
    static PROBE_PARAMS: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
}

/// Prints the summary of a moved curve at the parameters of the request (g3_show_curve).
pub fn show_curve(g: &Handle<dyn GeomGeometry>) {
    let c: Handle<dyn GeomCurve> = g.cast();
    PROBE_PARAMS.with(|params| curve_summary(&*c, &params.borrow()));
}

pub fn transform_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c) = built_curve(&mut r) else { return };
    let params = r.reals();
    PROBE_PARAMS.with(|p| *p.borrow_mut() = params.clone());
    let t = trsf(&mut r);
    for &u in &params {
        real(c.transformed_parameter(u, t));
    }
    real(c.parametric_transformation(t));
    show(c.reversed(), |copy| show_curve(&copy.cast()));
    show(c.reverse(), |()| show_curve(&c.cast()));
    moves(&c.cast(), &mut r, show_curve);
    end();
}

pub fn line_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (a, d, p, b) = (r.ax1(), r.dir(), r.pnt(), r.ax1());
    let u = r.reals();
    let l = Handle::new(GeomLine::new(a));
    let m = GeomLine::from_lin(Line3::from_axis(b));
    let n = GeomLine::from_point(p, d);
    ax1(l.position());
    ax1(l.lin().position());
    ax1(m.position());
    ax1(n.position());
    l.set_direction(d);
    ax1(l.position());
    l.set_location(p);
    ax1(l.position());
    l.set_position(b);
    ax1(l.position());
    l.set_lin(Line3::from_axis(a));
    curve_summary(&*l, &u);
    dump(&*l, -1);
    end();
}

fn conic_common(c: &dyn GeomConic) {
    ax1(c.x_axis());
    ax1(c.y_axis());
    ax1(c.axis());
    pnt(c.location());
    ax2(c.position());
    show(c.eccentricity(), real);
}

fn conic_specific(c: &Handle<dyn GeomConic>, value: f64) {
    let circle: Handle<GeomCircle> = c.cast();
    if let Some(e) = circle.get() {
        real(e.radius());
        attempt(|| e.set_radius(value));
        real(e.radius());
        show(e.circ(), |g| {
            ax2(g.position());
            real(g.radius());
            e.set_circ(g);
        });
    }
}

pub fn conic_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(curve) = built_curve(&mut r) else {
        return;
    };
    let u = r.reals();
    let c: Handle<dyn GeomConic> = curve.cast();
    let (value, a, p, f) = (r.real(), r.ax1(), r.pnt(), r.ax2());
    conic_common(&*c);
    conic_specific(&c, value);
    curve_summary(&*c, &u);
    c.set_location(p);
    conic_common(&*c);
    attempt(|| c.set_axis(a));
    conic_common(&*c);
    c.set_position(f);
    conic_common(&*c);
    c.reverse().unwrap();
    conic_common(&*c);
    curve_summary(&*c, &u);
    dump(&*c, -1);
    end();
}

/// The conic built from gp objects, one kind per request (g3_conic_build).
fn build_conic(kind: i32, f: Axis2, a: f64) -> Option<Handle<dyn GeomConic>> {
    match kind {
        1 => {
            let built = Circle3::new(f, a).map(|g| Handle::new(GeomCircle::from_circ(g)).cast());
            report(built)
        }
        _ => None,
    }
}

fn report(
    built: Result<Handle<dyn GeomConic>, rustycad_tkernel::standard::Failure>,
) -> Option<Handle<dyn GeomConic>> {
    match built {
        Ok(conic) => Some(conic),
        Err(error) => {
            print!("{} ", error.exception_type());
            None
        }
    }
}

pub fn conic_build_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let (f, a, _b) = (r.ax2(), r.real(), r.real());
    let (_d, _p) = (r.ax1(), r.pnt());
    if let Some(c) = build_conic(kind, f, a) {
        conic_common(&*c);
        dump(&*c, -1);
    }
    end();
}
