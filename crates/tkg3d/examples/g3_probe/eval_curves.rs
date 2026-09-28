//! The GeomEval curves of the TKG3d probe; the counterpart of
//! tools/oracle/tkg3d_probe_eval_curves.cxx.
use super::common::{
    Reader, ax2, built_curve, curve_summary, dump, end, flag, int, name, pnt, real, show, vec,
};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBoundedCurve, GeomCurve};
use rustycad_tkg3d::geom_eval::{
    GeomEvalAHTBezierCurve, GeomEvalCircularHelixCurve, GeomEvalSineWaveCurve, GeomEvalTBezierCurve,
};
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};

// Kinds 22 and 23: tbezier rational lower np poles nw weights alpha |
// aht rational lower degree alpha beta np poles nw weights.
fn bezier(kind: i32, r: &mut Reader) -> Result<Handle<dyn GeomCurve>, Failure> {
    let rational = r.flag();
    let lower = r.integer();
    let (mut degree, mut alpha, mut beta) = (0, 0.0, 0.0);
    if kind == 23 {
        (degree, alpha, beta) = (r.integer(), r.real(), r.real());
    }
    let np = r.integer();
    let mut p = Array1::<Point3>::with_bounds(lower, lower + np - 1).unwrap();
    for i in lower..lower + np {
        p[i] = r.pnt();
    }
    let nw = r.integer();
    let mut w = Array1::<f64>::with_bounds(lower, lower + nw - 1).unwrap();
    for i in lower..lower + nw {
        w[i] = r.real();
    }
    if kind == 22 {
        let a = r.real();
        let curve = if rational {
            GeomEvalTBezierCurve::with_weights(&p, &w, a)?
        } else {
            GeomEvalTBezierCurve::new(&p, a)?
        };
        return Ok(Handle::new(curve).cast());
    }
    let curve = if rational {
        GeomEvalAHTBezierCurve::with_weights(&p, &w, degree, alpha, beta)?
    } else {
        GeomEvalAHTBezierCurve::new(&p, degree, alpha, beta)?
    };
    Ok(Handle::new(curve).cast())
}

/// One eval curve specification of kind `kind` (20..=23), as g3_eval_curve reads it.
pub fn spec(kind: i32, r: &mut Reader) -> Result<Handle<dyn GeomCurve>, Failure> {
    if kind == 22 || kind == 23 {
        return bezier(kind, r);
    }
    assert!(kind == 20 || kind == 21, "unknown eval curve kind {kind}");
    let (f, x, y) = (r.ax2(), r.real(), r.real());
    if kind == 21 {
        return Ok(Handle::new(GeomEvalCircularHelixCurve::new(f, x, y)?).cast());
    }
    Ok(Handle::new(GeomEvalSineWaveCurve::new(f, x, y, r.real())?).cast())
}

fn poles(p: &Array1<'_, Point3>) {
    int(p.lower());
    int(p.len() as i32);
    p.iter().copied().for_each(pnt);
}
fn weights(w: &Array1<'_, f64>, values: bool) {
    int(w.lower());
    int(w.len() as i32);
    if values {
        w.iter().copied().for_each(real);
    }
}

fn tbezier_state(t: &GeomEvalTBezierCurve) {
    int(t.nb_poles());
    int(t.order());
    real(t.alpha());
    flag(t.is_rational());
    poles(&t.poles());
    weights(&t.weights(), true);
    show(t.start_point(), pnt);
    show(t.end_point(), pnt);
}
fn aht_state(t: &GeomEvalAHTBezierCurve) {
    int(t.nb_poles());
    int(t.alg_degree());
    real(t.alpha());
    real(t.beta());
    flag(t.is_rational());
    poles(&t.poles());
    weights(&t.weights(), t.is_rational());
    show(t.start_point(), pnt);
    show(t.end_point(), pnt);
}

fn state(c: &Handle<dyn GeomCurve>) {
    if let Some(t) = c.cast::<GeomEvalTBezierCurve>().get() {
        tbezier_state(t);
    }
    if let Some(t) = c.cast::<GeomEvalAHTBezierCurve>().get() {
        aht_state(t);
    }
    if let Some(s) = c.cast::<GeomEvalSineWaveCurve>().get() {
        ax2(s.position());
        [s.amplitude(), s.omega(), s.phase()]
            .into_iter()
            .for_each(real);
    }
    if let Some(s) = c.cast::<GeomEvalCircularHelixCurve>().get() {
        ax2(s.position());
        [s.radius(), s.pitch()].into_iter().for_each(real);
    }
}

// Reverse, Reversed, Transform and Transformed, which print only their failures.
fn refused_members(c: &Handle<dyn GeomCurve>) {
    let mut t = Transform3::default();
    t.set_translation(Vector3::new(1., 2., 3.));
    show(c.reverse(), |_| {});
    show(c.reversed(), |_| {});
    show(c.transform(t), |_| {});
    show(c.transformed(t), |_| {});
}

pub fn eval_curve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c) = built_curve(&mut r) else { return };
    let u = r.reals();
    name(&c);
    state(&c);
    let copy: Handle<dyn GeomCurve> = c.copy().unwrap().cast();
    name(&copy);
    state(&copy);
    curve_summary(&*copy, &u);
    for &v in &u {
        for n in 6..=9 {
            show(c.eval_dn(v, n), vec);
        }
    }
    refused_members(&c);
    for depth in [0, 1, 2, -1] {
        dump(&*c, depth);
    }
    end();
}
