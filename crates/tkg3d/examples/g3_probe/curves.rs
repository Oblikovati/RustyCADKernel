//! Curve evaluation, curve transformations, lines and conics; the counterpart of
//! tools/oracle/tkg3d_probe_curves.cxx.
use super::common::{
    Reader, attempt, ax1, ax2, built_curve, curve_summary, dump, end, name, pnt, real, show, trsf,
};
use super::common::{flag, vec};
use super::simple::moves;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::offset_curve_utils as utils;
use rustycad_tkg3d::geom::{GeomBoundedCurve, GeomCurve, GeomGeometry, GeomLine, GeomTrimmedCurve};
use rustycad_tkmath::gp::Line3;
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

fn trimmed_state(t: &GeomTrimmedCurve, u: &[f64]) {
    real(t.first_parameter());
    real(t.last_parameter());
    show(t.start_point(), pnt);
    show(t.end_point(), pnt);
    name(&t.basis_curve());
    curve_summary(t, u);
    dump(t, -1);
}

fn copied_state(t: &GeomTrimmedCurve, u: &[f64]) {
    show(t.copy(), |c| {
        trimmed_state(&c.cast::<GeomTrimmedCurve>(), u)
    });
}

pub fn trimmed_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c) = built_curve(&mut r) else {
        return;
    };
    let u = r.reals();
    let t: Handle<GeomTrimmedCurve> = c.cast();
    let (u1, u2) = (r.real(), r.real());
    let (sense, adjust) = (r.flag(), r.flag());
    let m = trsf(&mut r);
    trimmed_state(&t, &u);
    copied_state(&t, &u);
    attempt(|| t.set_trim(u1, u2, sense, adjust));
    trimmed_state(&t, &u);
    copied_state(&t, &u);
    attempt(|| t.reverse());
    trimmed_state(&t, &u);
    attempt(|| t.transform(m));
    trimmed_state(&t, &u);
    real(t.transformed_parameter(0.5, m));
    real(t.parametric_transformation(m));
    end();
}

/// Geom_OffsetCurveUtils::CalculateD0..D3 directly (g3_offset_utils).
pub fn offset_utils_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let p = r.pnt();
    let (d1, d2, d3, d4) = (r.vec(), r.vec(), r.vec(), r.vec());
    let dir = *r.vec().xyz();
    let (offset, change, tol) = (r.real(), r.flag(), r.real());
    let mut v = p;
    flag(utils::calculate_d0(&mut v, d1, dir, offset, tol));
    pnt(v);
    let (mut v, mut a1) = (p, d1);
    flag(utils::calculate_d1(&mut v, &mut a1, d2, dir, offset, tol));
    pnt(v);
    vec(a1);
    let (mut v, mut a1, mut a2) = (p, d1, d2);
    flag(utils::calculate_d2(
        &mut v,
        [&mut a1, &mut a2],
        d3,
        dir,
        offset,
        change,
        tol,
    ));
    pnt(v);
    [a1, a2].into_iter().for_each(vec);
    let (mut v, mut a1, mut a2, mut a3) = (p, d1, d2, d3);
    let derivatives = [&mut a1, &mut a2, &mut a3];
    flag(utils::calculate_d3(
        &mut v,
        derivatives,
        d4,
        dir,
        offset,
        change,
        tol,
    ));
    pnt(v);
    [a1, a2, a3].into_iter().for_each(vec);
    end();
}
