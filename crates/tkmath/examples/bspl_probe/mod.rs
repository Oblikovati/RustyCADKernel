//! BSplCLib counterparts of tkmath_bspl_probe.cxx and its companion files.
//!
//! Arrays arrive as "lower count values..."; points as "lower count coords...".
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib::{self, CurvePoint};
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::plib::PolePoint;

mod cache;
mod curves;
mod eval;
mod misc;
mod solve;
pub use cache::{bezier_case, cache_case};
pub use curves::{degree_case, insert_case, remove_case, trim_case, unperiodize_case};
pub use eval::{eval_case, flat_eval_case};
pub use misc::{function_case, intervals_case, merge_case, move_case, resolution_case, tang_case};
pub use solve::{basis_case, interpolate_case};

/// Sequential reader over one request's numeric fields.
pub struct In<'v> {
    values: &'v [f64],
    at: usize,
}

impl<'v> In<'v> {
    pub fn new(values: &'v [f64]) -> Self {
        Self { values, at: 0 }
    }
    pub fn r(&mut self) -> f64 {
        self.at += 1;
        self.values[self.at - 1]
    }
    pub fn i(&mut self) -> i32 {
        self.r() as i32
    }
    pub fn b(&mut self) -> bool {
        self.i() != 0
    }
    pub fn reals(&mut self) -> Array1<'static, f64> {
        let (lower, n) = (self.i(), self.i());
        let mut array: Array1<'static, f64> = Array1::with_bounds(lower, lower + n - 1).unwrap();
        for value in array.as_mut_slice() {
            *value = self.r();
        }
        array
    }
    pub fn ints(&mut self) -> Array1<'static, i32> {
        let (lower, n) = (self.i(), self.i());
        let mut array: Array1<'static, i32> = Array1::with_bounds(lower, lower + n - 1).unwrap();
        for value in array.as_mut_slice() {
            *value = self.i();
        }
        array
    }
    pub fn buf(&mut self) -> Vec<f64> {
        let n: i32 = self.i();
        (0..n.max(0)).map(|_| self.r()).collect()
    }
    pub fn points<P: PolePoint>(&mut self) -> Array1<'static, P> {
        let (lower, n) = (self.i(), self.i());
        let mut array: Array1<'static, P> = Array1::with_bounds(lower, lower + n - 1).unwrap();
        for point in array.as_mut_slice() {
            for k in 0..P::DIMENSION {
                point.set_pole_coordinate(k, self.r());
            }
        }
        array
    }
}

pub fn o(value: f64) {
    print!("{} ", value.to_bits());
}
pub fn oi(value: i32) {
    print!("{value} ");
}
pub fn fail(failure: &Failure) {
    if std::env::var_os("BSPL_PROBE_DEBUG").is_some() {
        eprintln!("{}: {}", failure.exception_type(), failure.message());
    }
    print!("{} ", failure.exception_type());
}
pub fn oa(array: &Array1<'_, f64>) {
    array.as_slice().iter().for_each(|value| o(*value));
}
pub fn oia(array: &Array1<'_, i32>) {
    array.as_slice().iter().for_each(|value| oi(*value));
}
pub fn ob(values: &[f64]) {
    values.iter().for_each(|value| o(*value));
}
pub fn op<P: PolePoint>(point: &P) {
    (0..P::DIMENSION).for_each(|k| o(point.pole_coordinate(k)));
}
pub fn ov<P: CurvePoint>(vector: &P::Vector) {
    (0..P::DIMENSION).for_each(|k| o(P::vector_coordinate(vector, k)));
}
pub fn opa<P: PolePoint>(array: &Array1<'_, P>) {
    array.as_slice().iter().for_each(op);
}
/// Prints the value or the failure type.
pub fn report<T>(result: Result<T, Failure>, print: impl FnOnce(T)) {
    match result {
        Ok(value) => print(value),
        Err(failure) => fail(&failure),
    }
}

// knots: deg periodic knots mults xs u1 u2 seq_ok weights pairs last
pub fn knots_case(v: &[f64]) {
    let mut f = In::new(v);
    let (deg, per) = (f.i(), f.b());
    let (k, m, xs) = (f.reals(), f.ints(), f.buf());
    let (u1, u2, seqok) = (f.r(), f.r(), f.b());
    let w = f.reals();
    let npairs: i32 = f.i();
    for x in xs {
        let mut pos: i32 = -7;
        bsplclib::hunt(&k, x, &mut pos).unwrap();
        oi(pos);
    }
    report(bsplclib::first_u_knot_index(deg, &m), oi);
    report(bsplclib::last_u_knot_index(deg, &m), oi);
    for index in m.lower()..=m.upper() {
        oi(bsplclib::flat_index(deg, index, &m, per).unwrap());
    }
    oi(bsplclib::max_knot_mult(&m, m.lower(), m.upper()).unwrap());
    oi(bsplclib::min_knot_mult(&m, m.lower(), m.upper()).unwrap());
    oi(bsplclib::nb_poles(deg, per, &m).unwrap());
    let len: i32 = bsplclib::knot_sequence_length(&m, deg, per).unwrap();
    oi(len);
    if seqok {
        sequences(&k, &m, deg, per, len);
    }
    forms(&k, &m, deg, per);
    for index in m.lower()..=m.upper() {
        oi(bsplclib::pole_index(deg, index, per, &m).unwrap());
    }
    reversals(&k, &m, u1, u2);
    for _ in 0..npairs {
        let (i1, i2) = (f.i(), f.i());
        oi(i32::from(bsplclib::is_rational(&w, i1, i2, 0.0).unwrap()));
    }
    reverse_weights(&w, f.i());
    println!();
}

fn sequences(k: &Array1<'_, f64>, m: &Array1<'_, i32>, deg: i32, per: bool, len: i32) {
    let mut seq: Array1<'static, f64> = Array1::with_bounds(1, len).unwrap();
    bsplclib::knot_sequence_with_degree(k, m, deg, per, &mut seq).unwrap();
    oa(&seq);
    let nk: i32 = bsplclib::knots_length(&seq, per).unwrap();
    oi(nk);
    let mut k2: Array1<'static, f64> = Array1::with_bounds(1, nk).unwrap();
    let mut m2: Array1<'static, i32> = Array1::with_bounds(1, nk).unwrap();
    bsplclib::knots(&seq, &mut k2, &mut m2, per).unwrap();
    oa(&k2);
    oia(&m2);
    if !per {
        let len0: i32 = bsplclib::knot_sequence_length(m, 0, false).unwrap();
        let mut seq0: Array1<'static, f64> = Array1::with_bounds(1, len0).unwrap();
        bsplclib::knot_sequence(k, m, &mut seq0, false).unwrap();
        oa(&seq0);
    }
}

fn forms(k: &Array1<'_, f64>, m: &Array1<'_, i32>, deg: i32, per: bool) {
    oi(bsplclib::knot_form(k, k.lower(), k.upper()).unwrap() as i32);
    oi(bsplclib::knot_form(k, k.lower() + 1, k.upper()).unwrap() as i32);
    oi(bsplclib::mult_form(m, m.lower(), m.upper()).unwrap() as i32);
    oi(bsplclib::mult_form(m, m.upper(), m.lower()).unwrap() as i32);
    report(bsplclib::knot_analysis(deg, per, k, m), |(form, mm)| {
        oi(form as i32);
        oi(mm);
    });
}

fn reversals(k: &Array1<'_, f64>, m: &Array1<'_, i32>, u1: f64, u2: f64) {
    let mut r = k.clone();
    bsplclib::reparametrize(u1, u2, &mut r).unwrap();
    oa(&r);
    let mut v = k.clone();
    bsplclib::reverse_knots(&mut v).unwrap();
    oa(&v);
    let mut mv = m.clone();
    bsplclib::reverse_mults(&mut mv);
    oia(&mv);
}

fn reverse_weights(w: &Array1<'_, f64>, last: i32) {
    let mut wr = w.clone();
    bsplclib::reverse_poles(&mut wr, last).unwrap();
    oa(&wr);
    let mut p3: Array1<'static, Point3> = Array1::with_bounds(w.lower(), w.upper()).unwrap();
    let mut p2: Array1<'static, Point2> = Array1::with_bounds(w.lower(), w.upper()).unwrap();
    for index in w.lower()..=w.upper() {
        let x: f64 = w[index];
        p3[index] = Point3::new(x, -x, 2.0 * x);
        p2[index] = Point2::new(x, f64::from(index));
    }
    bsplclib::reverse_poles(&mut p3, last).unwrap();
    opa(&p3);
    bsplclib::reverse_poles(&mut p2, last).unwrap();
    opa(&p2);
}

// locate: mode deg periodic knots usemults mults u k1 k2 hint
pub fn locate_case(v: &[f64]) {
    let mut f = In::new(v);
    let (mode, deg, per) = (f.i(), f.i(), f.b());
    let k = f.reals();
    let use_mults: bool = f.b();
    let m = f.ints();
    let (u, k1, k2, hint) = (f.r(), f.i(), f.i(), f.i());
    let (mut index, mut new_u): (i32, f64) = (hint, 91.0);
    let result = match mode {
        0 => bsplclib::locate_parameter(deg, &k, &m, u, per, k1, k2, &mut index, &mut new_u),
        1 => bsplclib::locate_parameter_flat_knots(deg, &k, u, per, k1, k2, &mut index, &mut new_u),
        _ => {
            let mults = if use_mults { Some(&m) } else { None };
            bsplclib::locate_parameter_for_eval(deg, &k, mults, u, per, &mut index, &mut new_u)
        }
    };
    report(result, |()| {
        oi(index);
        o(new_u);
    });
    println!();
}

// scheme: op deg dim u depth length tol knots poles
pub fn scheme_case(v: &[f64]) {
    let mut f = In::new(v);
    let (op, deg, dim, u) = (f.i(), f.i(), f.i(), f.r());
    let (depth, length, tol) = (f.i(), f.i(), f.r());
    let (k, mut p) = (f.buf(), f.buf());
    match op {
        0 => bsplclib::eval(u, deg, &k, dim, &mut p).unwrap(),
        1 => bsplclib::boor_scheme(u, deg, &k, dim, &mut p, depth, length).unwrap(),
        2 => oi(i32::from(
            bsplclib::anti_boor_scheme(u, deg, &k, dim, &mut p, depth, length, tol).unwrap(),
        )),
        3 => bsplclib::derivative(deg, &k, dim, length, depth, &mut p).unwrap(),
        _ => bsplclib::bohm(u, deg, depth, &k, dim, &mut p).unwrap(),
    }
    ob(&p);
    println!();
}

// build: BuildKnots, BuildEval (scalar, 3D, 2D), BuildBoor, BoorIndex and GetPole.
pub fn build_case(v: &[f64]) {
    let mut f = In::new(v);
    let (deg, index, per, use_mults) = (f.i(), f.i(), f.b(), f.b());
    let (k, m) = (f.reals(), f.ints());
    let mut lk: Vec<f64> = vec![91.0; (2 * deg).max(1) as usize];
    let mults = if use_mults { Some(&m) } else { None };
    report(
        bsplclib::build_knots(deg, index, per, &k, mults, &mut lk),
        |()| ob(&lk),
    );
    let (eindex, rational) = (f.i(), f.b());
    let (w, p1, p3, p2) = (
        f.reals(),
        f.reals(),
        f.points::<Point3>(),
        f.points::<Point2>(),
    );
    let weights = if rational { Some(&w) } else { None };
    let mut lp: Vec<f64> = vec![91.0; (4 * (deg + 1)) as usize];
    report(
        bsplclib::build_eval_scalar(deg, eindex, &p1, weights, &mut lp),
        |()| ob(&lp),
    );
    lp.fill(91.0);
    report(
        bsplclib::build_eval(deg, eindex, &p3, weights, &mut lp),
        |()| ob(&lp),
    );
    lp.fill(91.0);
    report(
        bsplclib::build_eval(deg, eindex, &p2, weights, &mut lp),
        |()| ob(&lp),
    );
    let (bindex, blength, bdim, depth) = (f.i(), f.i(), f.i(), f.i());
    boor_part(&p1, [bindex, blength, bdim, depth], f.i());
    println!();
}

fn boor_part(p1: &Array1<'_, f64>, [bindex, blength, bdim, depth]: [i32; 4], mut pos: i32) {
    let mut boor: Vec<f64> = vec![91.0; ((2 * blength + 1) * bdim) as usize];
    report(
        bsplclib::build_boor(bindex, blength, bdim, p1, &mut boor),
        |()| ob(&boor),
    );
    for index in -1..=blength + 2 {
        oi(bsplclib::boor_index(index, blength, depth));
    }
    let mut pole: Array1<'static, f64> = Array1::with_bounds(1, bdim * 2).unwrap();
    pole.init(91.0);
    let mut index: i32 = 0;
    while index <= blength && bsplclib::boor_index(index, blength, depth) <= 2 * blength {
        match bsplclib::get_pole(index, blength, depth, bdim, &boor, &mut pos, &mut pole) {
            Ok(()) => oi(pos),
            Err(failure) => {
                fail(&failure);
                break;
            }
        }
        index += 1;
    }
    oa(&pole);
}
