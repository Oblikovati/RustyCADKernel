//! Curve evaluation, curve moves, conics, trimmed and offset curves, and the offset
//! utilities; the counterpart of tools/oracle/tkg2d_probe_curves.cxx.
use super::common::{
    Reader, attempt, ax, ax22, built, curve, dump, end, flag, int, pnt, real, show, summary, trsf,
    vec,
};
use super::simple::moves;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::offset_curve_utils::{
    calculate_d0, calculate_d1, calculate_d2, calculate_d3,
};
use rustycad_tkg2d::geom2d::{
    Geom2dBoundedCurve, Geom2dCircle, Geom2dConic, Geom2dCurve, Geom2dEllipse, Geom2dGeometry,
    Geom2dHyperbola, Geom2dOffsetCurve, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkmath::gp::Axis22d;
use std::cell::RefCell;

fn type_name(c: &Handle<dyn Geom2dCurve>) {
    print!("{} ", c.dynamic_type().unwrap().name());
}

pub fn curve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c) = built(&mut r) else { return };
    let u = r.reals();
    type_name(&c);
    summary(&*c, &u);
    dump(&*c, -1);
    dump(&*c, 1);
    end();
}

thread_local! {
    static PARAMS: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
}

fn show_curve(g: &Handle<dyn Geom2dGeometry>) {
    let c: Handle<dyn Geom2dCurve> = g.cast();
    PARAMS.with(|params| summary(&*c, &params.borrow()));
}

pub fn transform_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c) = built(&mut r) else { return };
    let u = r.reals();
    PARAMS.with(|params| *params.borrow_mut() = u.clone());
    let t = trsf(&mut r);
    for &v in &u {
        real(c.transformed_parameter(v, t));
    }
    real(c.parametric_transformation(t));
    show(c.reversed(), |r| show_curve(&r.cast()));
    attempt(|| {
        c.reverse()?;
        show_curve(&c.cast());
        Ok(())
    });
    moves(&c.cast(), &mut r, show_curve);
    end();
}

fn conic_common(c: &dyn Geom2dConic) {
    ax(c.x_axis());
    ax(c.y_axis());
    pnt(c.location());
    ax22(c.position());
    show(c.eccentricity(), real);
}

fn circle_specific(e: &Geom2dCircle, value: f64) {
    real(e.radius());
    attempt(|| e.set_radius(value));
    real(e.radius());
    let g = e.circ2d().unwrap();
    ax22(g.axis());
    real(g.radius());
    e.set_circ2d(g);
}

fn ellipse_specific(e: &Geom2dEllipse, value: f64) {
    real(e.major_radius());
    real(e.minor_radius());
    show(e.directrix1(), ax);
    show(e.directrix2(), ax);
    real(e.focal());
    pnt(e.focus1());
    pnt(e.focus2());
    real(e.parameter());
    attempt(|| e.set_minor_radius(value));
    attempt(|| e.set_major_radius(value));
    real(e.major_radius());
    real(e.minor_radius());
    attempt(|| {
        let g = e.elips2d()?;
        ax22(g.axis());
        e.set_elips2d(g);
        Ok(())
    });
}

fn hyperbola_specific(e: &Geom2dHyperbola, value: f64) {
    real(e.major_radius());
    real(e.minor_radius());
    show(e.asymptote1(), ax);
    show(e.asymptote2(), ax);
    show(e.conjugate_branch1(), |h| ax22(h.axis()));
    show(e.conjugate_branch2(), |h| ax22(h.axis()));
    show(e.directrix1(), ax);
    show(e.directrix2(), ax);
    real(e.focal());
    pnt(e.focus1());
    pnt(e.focus2());
    show(e.other_branch(), |h| ax22(h.axis()));
    show(e.parameter(), real);
    attempt(|| e.set_major_radius(value));
    attempt(|| e.set_minor_radius(-value));
    real(e.major_radius());
    real(e.minor_radius());
    attempt(|| {
        let g = e.hypr2d()?;
        ax22(g.axis());
        e.set_hypr2d(g);
        Ok(())
    });
}

fn parabola_specific(e: &Geom2dParabola, value: f64) {
    real(e.focal());
    pnt(e.focus());
    real(e.parameter());
    show(e.directrix(), ax);
    attempt(|| e.set_focal(value));
    real(e.focal());
    attempt(|| {
        let g = e.parab2d()?;
        ax22(g.axis());
        e.set_parab2d(g);
        Ok(())
    });
}

fn conic_specific(c: &Handle<dyn Geom2dCurve>, value: f64) {
    if let Some(e) = c.cast::<Geom2dCircle>().get() {
        circle_specific(e, value);
    }
    if let Some(e) = c.cast::<Geom2dEllipse>().get() {
        ellipse_specific(e, value);
    }
    if let Some(e) = c.cast::<Geom2dHyperbola>().get() {
        hyperbola_specific(e, value);
    }
    if let Some(e) = c.cast::<Geom2dParabola>().get() {
        parabola_specific(e, value);
    }
}

pub fn conic_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(curve) = built(&mut r) else { return };
    let u = r.reals();
    let c: Handle<dyn Geom2dConic> = curve.cast();
    let value = r.real();
    let (a, p) = (r.ax(), r.pnt());
    conic_common(&*c);
    conic_specific(&curve, value);
    summary(&*c, &u);
    c.set_location(p);
    conic_common(&*c);
    attempt(|| c.set_x_axis(a));
    conic_common(&*c);
    attempt(|| c.set_y_axis(a));
    conic_common(&*c);
    c.set_axis(Axis22d::new(p, a.direction(), true).unwrap());
    conic_common(&*c);
    c.reverse().unwrap();
    conic_common(&*c);
    summary(&*c, &u);
    dump(&*c, -1);
    end();
}

fn trimmed_state(t: &Geom2dTrimmedCurve, u: &[f64]) {
    real(t.first_parameter());
    real(t.last_parameter());
    show(t.start_point(), pnt);
    show(t.end_point(), pnt);
    type_name(&t.basis_curve());
    summary(t, u);
    dump(t, -1);
}

fn trimmed_copy(t: &Geom2dTrimmedCurve, u: &[f64]) -> Result<(), Failure> {
    let copy: Handle<Geom2dTrimmedCurve> = t.copy()?.cast();
    trimmed_state(&copy, u);
    Ok(())
}

pub fn trimmed_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c) = built(&mut r) else { return };
    let u = r.reals();
    let t: Handle<Geom2dTrimmedCurve> = c.cast();
    let (u1, u2) = (r.real(), r.real());
    let (sense, adjust) = (r.flag(), r.flag());
    let m = trsf(&mut r);
    trimmed_state(&t, &u);
    attempt(|| trimmed_copy(&t, &u));
    attempt(|| t.set_trim(u1, u2, sense, adjust));
    trimmed_state(&t, &u);
    attempt(|| trimmed_copy(&t, &u));
    attempt(|| t.reverse());
    trimmed_state(&t, &u);
    attempt(|| t.transform(m));
    trimmed_state(&t, &u);
    real(t.transformed_parameter(0.5, m));
    real(t.parametric_transformation(m));
    end();
}

fn offset_state(o: &Geom2dOffsetCurve, u: &[f64]) {
    real(o.offset());
    int(o.basis_curve_continuity() as i32);
    type_name(&o.basis_curve());
    flag(o.has_eval_representation());
    summary(o, u);
    dump(o, -1);
}

pub fn offset_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c) = built(&mut r) else { return };
    let u = r.reals();
    let o: Handle<Geom2dOffsetCurve> = c.cast();
    let d = r.real();
    let no_check = r.flag();
    let m = trsf(&mut r);
    let basis = curve(&mut r).unwrap();
    offset_state(&o, &u);
    offset_state(&o.copy().unwrap().cast::<Geom2dOffsetCurve>(), &u);
    o.set_offset_value(d);
    offset_state(&o, &u);
    attempt(|| o.reverse());
    offset_state(&o, &u);
    attempt(|| o.transform(m));
    offset_state(&o, &u);
    attempt(|| o.set_basis_curve(&basis, no_check));
    offset_state(&o, &u);
    attempt(|| o.set_basis_curve(&o.cast(), no_check));
    offset_state(&o, &u);
    real(o.transformed_parameter(0.5, m));
    real(o.parametric_transformation(m));
    end();
}

pub fn offset_utils_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let p = r.pnt();
    let (d1, d2, d3, d4) = (r.vec(), r.vec(), r.vec(), r.vec());
    let offset = r.real();
    let change = r.flag();
    let mut v = p;
    attempt(|| {
        flag(calculate_d0(&mut v, d1, offset)?);
        pnt(v);
        Ok(())
    });
    let (mut a1, mut v) = (d1, p);
    flag(calculate_d1(&mut v, &mut a1, d2, offset));
    pnt(v);
    vec(a1);
    let (mut a1, mut a2, mut v) = (d1, d2, p);
    flag(calculate_d2(&mut v, &mut a1, &mut a2, d3, change, offset));
    pnt(v);
    vec(a1);
    vec(a2);
    let (mut a1, mut a2, mut a3, mut v) = (d1, d2, d3, p);
    flag(calculate_d3(
        &mut v,
        [&mut a1, &mut a2, &mut a3],
        d4,
        change,
        offset,
    ));
    pnt(v);
    [a1, a2, a3].into_iter().for_each(vec);
    end();
}
