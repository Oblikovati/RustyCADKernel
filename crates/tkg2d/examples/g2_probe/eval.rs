//! The Geom2dEval curves and the Geom2dHash hashers; the counterpart of
//! tools/oracle/tkg2d_probe_eval.cxx.
use super::common::{Reader, ax, built, dump, end, flag, int, pnt, real, show, summary, vec};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBoundedCurve, Geom2dCurve};
use rustycad_tkg2d::geom2d_eval::{
    Geom2dEvalAHTBezierCurve, Geom2dEvalArchimedeanSpiralCurve, Geom2dEvalCircleInvoluteCurve,
    Geom2dEvalLogarithmicSpiralCurve, Geom2dEvalSineWaveCurve, Geom2dEvalTBezierCurve,
};
use rustycad_tkg2d::geom2d_hash::{
    Geom2dHashAxisPlacement, Geom2dHashCurveHasher, Geom2dHashDirectionHasher,
    Geom2dHashPointHasher,
};
use rustycad_tkmath::gp::{Axis22d, Point2};

fn bezier(kind: i32, r: &mut Reader) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let rational = r.flag();
    let lower = r.integer();
    let (mut degree, mut alpha, mut beta) = (0, 0.0, 0.0);
    if kind == 16 {
        (degree, alpha, beta) = (r.integer(), r.real(), r.real());
    }
    let np = r.integer();
    let mut p = Array1::<Point2>::with_bounds(lower, lower + np - 1).unwrap();
    for i in lower..lower + np {
        p[i] = r.pnt();
    }
    let nw = r.integer();
    let mut w = Array1::<f64>::with_bounds(lower, lower + nw - 1).unwrap();
    for i in lower..lower + nw {
        w[i] = r.real();
    }
    if kind == 15 {
        let a = r.real();
        let curve = if rational {
            Geom2dEvalTBezierCurve::with_weights(&p, &w, a)?
        } else {
            Geom2dEvalTBezierCurve::new(&p, a)?
        };
        return Ok(Handle::new(curve).cast());
    }
    let curve = if rational {
        Geom2dEvalAHTBezierCurve::with_weights(&p, &w, degree, alpha, beta)?
    } else {
        Geom2dEvalAHTBezierCurve::new(&p, degree, alpha, beta)?
    };
    Ok(Handle::new(curve).cast())
}

/// Curve kinds 11..16, as g2_eval_curve reads them.
pub fn curve(kind: i32, r: &mut Reader) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    if kind >= 15 {
        return bezier(kind, r);
    }
    let a = r.ax();
    let x = r.real();
    if kind == 12 {
        return Ok(Handle::new(Geom2dEvalCircleInvoluteCurve::new(a, x)?).cast());
    }
    let y = r.real();
    match kind {
        11 => Ok(Handle::new(Geom2dEvalArchimedeanSpiralCurve::new(a, x, y)?).cast()),
        13 => Ok(Handle::new(Geom2dEvalLogarithmicSpiralCurve::new(a, x, y)?).cast()),
        _ => Ok(Handle::new(Geom2dEvalSineWaveCurve::new(a, x, y, r.real())?).cast()),
    }
}

fn poles(p: &Array1<'_, Point2>) {
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

fn tbezier_state(t: &Geom2dEvalTBezierCurve) {
    int(t.nb_poles());
    int(t.order());
    real(t.alpha());
    flag(t.is_rational());
    poles(&t.poles());
    weights(&t.weights(), true);
    show(t.start_point(), pnt);
    show(t.end_point(), pnt);
}
fn aht_state(t: &Geom2dEvalAHTBezierCurve) {
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

fn analytic_state(c: &Handle<dyn Geom2dCurve>) {
    if let Some(s) = c.cast::<Geom2dEvalArchimedeanSpiralCurve>().get() {
        ax(s.position());
        [s.initial_radius(), s.growth_rate()]
            .into_iter()
            .for_each(real);
    }
    if let Some(s) = c.cast::<Geom2dEvalCircleInvoluteCurve>().get() {
        ax(s.position());
        real(s.radius());
    }
    if let Some(s) = c.cast::<Geom2dEvalLogarithmicSpiralCurve>().get() {
        ax(s.position());
        [s.scale(), s.growth_exponent()].into_iter().for_each(real);
    }
    if let Some(s) = c.cast::<Geom2dEvalSineWaveCurve>().get() {
        ax(s.position());
        [s.amplitude(), s.omega(), s.phase()]
            .into_iter()
            .for_each(real);
    }
}

fn state(c: &Handle<dyn Geom2dCurve>) {
    if let Some(t) = c.cast::<Geom2dEvalTBezierCurve>().get() {
        tbezier_state(t);
    }
    if let Some(t) = c.cast::<Geom2dEvalAHTBezierCurve>().get() {
        aht_state(t);
    }
    analytic_state(c);
}

fn type_name(c: &Handle<dyn Geom2dCurve>) {
    print!("{} ", c.dynamic_type().unwrap().name());
}

pub fn eval_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c) = built(&mut r) else { return };
    let u = r.reals();
    type_name(&c);
    state(&c);
    let copy: Handle<dyn Geom2dCurve> = c.copy().unwrap().cast();
    type_name(&copy);
    state(&copy);
    summary(&*copy, &u);
    for &v in &u {
        for n in 6..=9 {
            show(c.eval_dn(v, n), vec);
        }
    }
    for depth in [0, 1, 2] {
        dump(&*c, depth);
    }
    end();
}

fn hash_value(value: usize) {
    print!("h{value} ");
}

pub fn hash_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (comp, tol) = (r.real(), r.real());
    let n = r.integer();
    let mut curves: Vec<Handle<dyn Geom2dCurve>> = Vec::new();
    for _ in 0..n {
        let c = match r.integer() {
            0 => {
                let Some(c) = built(&mut r) else { return };
                c
            }
            2 => curves[r.integer() as usize].clone(),
            _ => Handle::default(),
        };
        curves.push(c);
    }
    let h = Geom2dHashCurveHasher::new(comp, tol);
    curves.iter().for_each(|c| hash_value(h.hash(c)));
    for a in &curves {
        curves.iter().for_each(|b| flag(h.equal(a, b)));
    }
    end();
}

pub fn hash_frame_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (comp, tol) = (r.real(), r.real());
    let (p1, p2, d1, d2) = (r.pnt(), r.pnt(), r.dir(), r.dir());
    let points = Geom2dHashPointHasher::new(comp, tol);
    let directions = Geom2dHashDirectionHasher::new(comp, tol);
    hash_value(points.hash(p1));
    hash_value(points.hash(p2));
    flag(points.equal(p1, p2));
    hash_value(directions.hash(d1));
    hash_value(directions.hash(d2));
    flag(directions.equal(d1, d2));
    let mut frames: Vec<Axis22d> = Vec::new();
    for _ in 0..2 {
        let (p, x, y) = (r.pnt(), r.dir(), r.dir());
        match Axis22d::with_directions(p, x, y) {
            Ok(frame) => frames.push(frame),
            Err(error) => {
                println!("{}", error.exception_type());
                return;
            }
        }
    }
    let axes = Geom2dHashAxisPlacement::new(comp, tol);
    hash_value(axes.hash(frames[0]));
    hash_value(axes.hash(frames[1]));
    flag(axes.equal(frames[0], frames[1]));
    end();
}
