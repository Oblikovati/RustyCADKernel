//! The AdvApprox package; the counterpart of tools/oracle/tkg3d_probe_advapprox.cxx,
//! with the same probe functions and evaluators.
use super::common::{Reader, end, flag, int, pnt, real, show};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adv_approx::{
    AdvApproxApproxAFunction, AdvApproxApproximationInput, AdvApproxApproximationOutput,
    AdvApproxCutting, AdvApproxDichoCutting, AdvApproxEvaluation, AdvApproxEvaluatorFunction,
    AdvApproxPrefAndRec, AdvApproxSimpleApprox, approximation,
};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::plib::{self, JacobiPolynomial};

fn polynomial(s: f64, c: f64, t: f64, d: i32) -> f64 {
    match d {
        0 => ((s * t - 2.0) * t + 0.5) * t + c,
        1 => (3.0 * s * t - 4.0) * t + 0.5,
        _ => 6.0 * s * t - 4.0,
    }
}

fn trigonometric(s: f64, c: f64, t: f64, d: i32) -> f64 {
    match d {
        0 => (s * t).sin() + c * (0.5 * t).cos(),
        1 => s * (s * t).cos() - 0.5 * c * (0.5 * t).sin(),
        _ => -s * s * (s * t).sin() - 0.25 * c * (0.5 * t).cos(),
    }
}

fn rational(s: f64, t: f64, d: i32) -> f64 {
    let q: f64 = 1.0 + s * t * t;
    match d {
        0 => 1.0 / q,
        1 => -2.0 * s * t / (q * q),
        _ => (6.0 * s * s * t * t - 2.0 * s) / (q * q * q),
    }
}

fn helix(c: i32, t: f64, d: i32) -> f64 {
    let r: f64 = 2.0 + f64::from(c / 3);
    match (c % 3, d) {
        (0, 0) => r * t.cos(),
        (0, 1) => -r * t.sin(),
        (0, _) => -r * t.cos(),
        (1, 0) => r * t.sin(),
        (1, 1) => r * t.cos(),
        (1, _) => -r * t.sin(),
        (_, 0) => 0.3 * t,
        (_, 1) => 0.3,
        _ => 0.0,
    }
}

fn kink(s: f64, t: f64, d: i32) -> f64 {
    let x: f64 = t - 0.3;
    let a: f64 = x.abs();
    match d {
        0 => s * a.powf(2.5),
        1 => s * 2.5 * a.powf(1.5) * if x < 0.0 { -1.0 } else { 1.0 },
        _ => s * 3.75 * a.powf(0.5),
    }
}

fn exponential(s: f64, t: f64, d: i32) -> f64 {
    match d {
        0 => (0.5 * s * t).exp(),
        1 => 0.5 * s * (0.5 * s * t).exp(),
        _ => 0.25 * s * s * (0.5 * s * t).exp(),
    }
}

/// Coordinate `c` of function `kind`, derivative `d` (g3_aa_value).
fn value(kind: i32, c: i32, t: f64, d: i32) -> f64 {
    let s: f64 = f64::from(c) + 1.0;
    match kind {
        0 => polynomial(s, f64::from(c), t, d),
        1 => trigonometric(s, f64::from(c), t, d),
        2 => rational(s, t, d),
        3 => helix(c, t, d),
        4 => kink(s, t, d),
        _ => exponential(s, t, d),
    }
}

/// The probe evaluator (G3AdvEvaluator).
struct Evaluator {
    kind: i32,
    mode: i32,
    fail_at: i32,
    calls: i32,
}

impl AdvApproxEvaluatorFunction for Evaluator {
    fn evaluate(&mut self, dimension: i32, _: [f64; 2], t: f64, d: i32, result: &mut [f64]) -> i32 {
        self.evaluate_values(dimension, t, d, result)
    }

    fn evaluate_native(&mut self, call: &mut AdvApproxEvaluation<'_>) {
        if self.mode == 3 && self.calls == 0 {
            call.start_end[1] = 2.0;
            *call.parameter = 0.25;
        }
        if self.mode == 3 && self.calls == 4 {
            *call.derivative_request = 1;
        }
        let dimension = *call.dimension;
        *call.error_code = self.evaluate_values(
            dimension,
            *call.parameter,
            *call.derivative_request,
            call.result,
        );
        if self.mode == 3 && self.calls == 5 {
            *call.dimension = 1;
        }
    }
}

impl Evaluator {
    fn evaluate_values(&mut self, dimension: i32, t: f64, d: i32, result: &mut [f64]) -> i32 {
        self.calls += 1;
        if (self.mode == 1 && d == 2) || (self.mode == 2 && self.calls == self.fail_at) {
            return 1;
        }
        for c in 0..dimension {
            result[c as usize] = value(self.kind, c, t, d);
        }
        0
    }
}

fn evaluator(r: &mut Reader) -> Evaluator {
    let (kind, mode) = (r.integer(), r.integer());
    let fail_at = r.integer();
    Evaluator {
        kind,
        mode,
        fail_at,
        calls: 0,
    }
}

fn table(values: &[f64]) -> Array1<'static, f64> {
    let mut array = Array1::with_bounds(1, values.len() as i32).unwrap();
    array.as_mut_slice().copy_from_slice(values);
    array
}

/// A cutting spec (g3_aa_cutting): None for kind 0.
fn cutting(r: &mut Reader) -> Result<Option<Box<dyn AdvApproxCutting>>, Failure> {
    match r.integer() {
        1 => Ok(Some(Box::new(AdvApproxDichoCutting::new()))),
        2 => {
            let (rec, pref) = (r.reals(), r.reals());
            let w = r.real();
            let tool = AdvApproxPrefAndRec::new(&table(&rec), &table(&pref), w)?;
            Ok(Some(Box::new(tool)))
        }
        _ => Ok(None),
    }
}

fn shape(code: i32) -> Shape {
    match code {
        i32::MIN..=0 => Shape::C0,
        1 => Shape::C1,
        2 => Shape::C2,
        3 => Shape::C3,
        _ => Shape::G1,
    }
}

fn tolerances(r: &mut Reader, n: i32) -> Option<Array1<'static, f64>> {
    let t = r.real();
    (n != 0).then(|| table(&(1..=n).map(|i| t * f64::from(i)).collect::<Vec<f64>>()))
}

fn poles(a: &AdvApproxApproxAFunction) {
    let n = a.nb_poles().unwrap();
    if let Some(p) = a.poles_1d() {
        p.as_slice().iter().copied().for_each(real);
        for i in 1..=a.num_sub_spaces(1).unwrap() {
            let mut column: Array1<'static, f64> = Array1::with_bounds(1, n).unwrap();
            a.poles_1d_of(i, &mut column).unwrap();
            column.as_slice().iter().copied().for_each(real);
        }
    }
    let pnt2 = |v: &Point2| {
        real(v.x());
        real(v.y());
    };
    if let Some(p) = a.poles_2d() {
        p.as_slice().iter().for_each(pnt2);
        for i in 1..=a.num_sub_spaces(2).unwrap() {
            let mut column: Array1<'static, Point2> = Array1::with_bounds(1, n).unwrap();
            a.poles_2d_of(i, &mut column).unwrap();
            column.as_slice().iter().for_each(pnt2);
        }
    }
    if let Some(p) = a.poles() {
        p.as_slice().iter().copied().for_each(pnt);
        for i in 1..=a.num_sub_spaces(3).unwrap() {
            let mut column: Array1<'static, Point3> = Array1::with_bounds(1, n).unwrap();
            a.poles_of(i, &mut column).unwrap();
            column.as_slice().iter().copied().for_each(pnt);
        }
    }
}

fn errors(a: &AdvApproxApproxAFunction) {
    for d in 1..=3 {
        if a.num_sub_spaces(d).unwrap() == 0 {
            flag(a.max_errors(d).unwrap().is_none());
            flag(a.average_errors(d).unwrap().is_none());
            continue;
        }
        for i in 1..=a.num_sub_spaces(d).unwrap() {
            real(a.max_error(d, i).unwrap());
            real(a.average_error(d, i).unwrap());
        }
        a.max_errors(d)
            .unwrap()
            .unwrap()
            .as_slice()
            .iter()
            .copied()
            .for_each(real);
        a.average_errors(d)
            .unwrap()
            .unwrap()
            .as_slice()
            .iter()
            .copied()
            .for_each(real);
    }
    if let Err(error) = a.max_errors(0) {
        print!("{} ", error.exception_type());
    }
    if let Err(error) = a.average_errors(4) {
        print!("{} ", error.exception_type());
    }
}

fn dump_text(write: impl FnOnce(&mut Vec<u8>) -> std::io::Result<()>) {
    let mut text: Vec<u8> = Vec::new();
    write(&mut text).unwrap();
    print!("[{}] ", String::from_utf8(text).unwrap().replace('\n', "|"));
}

fn result(a: &AdvApproxApproxAFunction) {
    flag(a.is_done());
    flag(a.has_result());
    for d in 0..=4 {
        show(a.num_sub_spaces(d), int);
    }
    int(a.nb_poles().unwrap());
    if !a.has_result() {
        print!("noresult ");
        return;
    }
    int(a.degree());
    int(a.nb_knots().unwrap());
    a.knots().unwrap().as_slice().iter().copied().for_each(real);
    a.multiplicities()
        .unwrap()
        .as_slice()
        .iter()
        .copied()
        .for_each(int);
    poles(a);
    errors(a);
    dump_text(|text| a.dump(text));
}

pub fn advapprox_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut f = evaluator(&mut r);
    let (n1, n2, n3) = (r.integer(), r.integer(), r.integer());
    let (t1, t2, t3) = (
        tolerances(&mut r, n1),
        tolerances(&mut r, n2),
        tolerances(&mut r, n3),
    );
    let (first, last) = (r.real(), r.real());
    let cont = shape(r.integer());
    let (deg, seg) = (r.integer(), r.integer());
    let built = cutting(&mut r).and_then(|cut| match cut {
        Some(cut) => AdvApproxApproxAFunction::with_cutting(
            n1,
            n2,
            n3,
            t1.as_ref(),
            t2.as_ref(),
            t3.as_ref(),
            first,
            last,
            cont,
            deg,
            seg,
            &mut f,
            &*cut,
        ),
        None => AdvApproxApproxAFunction::new(
            n1,
            n2,
            n3,
            t1.as_ref(),
            t2.as_ref(),
            t3.as_ref(),
            first,
            last,
            cont,
            deg,
            seg,
            &mut f,
        ),
    });
    show(built, |a| result(&a));
    int(f.calls);
    end();
}

pub fn advapprox_static_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut f = evaluator(&mut r);
    let nss = r.integer();
    let dims_values: Vec<i32> = (0..nss).map(|_| r.integer()).collect();
    let mut dims: Array1<'static, i32> = Array1::with_bounds(1, nss.max(1)).unwrap();
    dims.as_mut_slice()[..dims_values.len()].copy_from_slice(&dims_values);
    let given = r.integer();
    let (first, last) = (r.real(), r.real());
    let (order, coeffs, seg) = (r.integer(), r.integer(), r.integer());
    let (tol, code) = (r.real(), r.integer());
    let tol_values: Vec<f64> = (1..=nss.max(1)).map(|i| tol * f64::from(i)).collect();
    let tols = table(&tol_values);
    let size = (seg * coeffs * given).max(1);
    let mut ncoeff: Array1<'static, i32> = Array1::with_bounds(1, seg.max(1)).unwrap();
    ncoeff.init(-7);
    let mut coef: Array1<'static, f64> = Array1::with_bounds(1, size).unwrap();
    let mut ints: Array1<'static, f64> = Array1::with_bounds(1, seg + 1).unwrap();
    let mut emax: Array1<'static, f64> = Array1::with_bounds(1, (seg * nss).max(1)).unwrap();
    let mut eavg: Array1<'static, f64> = Array1::with_bounds(1, (seg * nss).max(1)).unwrap();
    let mut run = || -> Result<(i32, i32), Failure> {
        let cut = cutting(&mut r)?;
        let dicho = AdvApproxDichoCutting::new();
        let tool: &dyn AdvApproxCutting = match &cut {
            Some(cut) => &**cut,
            None => &dicho,
        };
        let input = AdvApproxApproximationInput {
            total_dimension: given,
            total_num_ss: nss,
            local_dimension: &dims,
            first,
            last,
            continuity_order: order,
            num_max_coeffs: coeffs,
            max_segments: seg,
            tolerances: &tols,
            code_precis: code,
        };
        let mut output = AdvApproxApproximationOutput {
            num_curves: 0,
            num_coeff_per_curve: &mut ncoeff,
            coefficients: &mut coef,
            intervals: &mut ints,
            error_max: &mut emax,
            average_error: &mut eavg,
            error_code: 0,
        };
        approximation(&input, &mut f, tool, &mut output)?;
        Ok((output.num_curves, output.error_code))
    };
    let outcome = run();
    show(outcome, |(num, err)| {
        int(num);
        int(err);
        (1..=num).for_each(|i| int(ncoeff[i]));
        if num > 0 {
            (1..=num + 1).for_each(|i| real(ints[i]));
        }
        coef.as_slice().iter().copied().for_each(real);
        for i in 1..=num * nss {
            real(emax[i]);
            real(eavg[i]);
        }
    });
    int(f.calls);
    end();
}

pub fn advapprox_simple_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut f = evaluator(&mut r);
    let nss = r.integer();
    let dims_values: Vec<i32> = (0..nss).map(|_| r.integer()).collect();
    let td: i32 = dims_values.iter().sum();
    let mut dims: Array1<'static, i32> = Array1::with_bounds(1, nss).unwrap();
    dims.as_mut_slice().copy_from_slice(&dims_values);
    let cont = shape(r.integer());
    let (code, maxcoef) = (r.integer(), r.integer());
    let (first, last) = (r.real(), r.real());
    let (maxdeg, tol) = (r.integer(), r.real());
    let tols = table(&(1..=nss).map(|i| tol * f64::from(i)).collect::<Vec<f64>>());
    let run = |f: &mut Evaluator| -> Result<(), Failure> {
        let (mut ng, mut wd) = (0, 0);
        plib::jacobi_parameters(cont, maxcoef, code, &mut ng, &mut wd)?;
        int(ng);
        int(wd);
        let jac = JacobiPolynomial::new(wd, cont)?;
        let mut s = AdvApproxSimpleApprox::new(td, nss, cont, wd, ng, &jac, f)?;
        s.perform(&dims, &tols, first, last, maxdeg)?;
        flag(s.is_done());
        if !s.is_done() {
            return Ok(());
        }
        s.first_constr().as_slice().iter().copied().for_each(real);
        int(s.degree());
        s.coefficients().as_slice().iter().copied().for_each(real);
        s.last_constr().as_slice().iter().copied().for_each(real);
        for i in td..=s.som_tab().upper() {
            real(s.som_tab()[i]);
            real(s.dif_tab()[i]);
        }
        if ng % 2 == 1 {
            for i in 0..td {
                real(s.som_tab()[i]);
                real(s.dif_tab()[i]);
            }
        }
        for i in 1..=nss {
            real(s.max_error(i)?);
            real(s.average_error(i)?);
        }
        dump_text(|text| s.dump(text));
        s.perform(&dims, &tols, first, 0.5 * (first + last), maxdeg)?;
        flag(s.is_done());
        int(s.degree());
        real(s.max_error(1)?);
        Ok(())
    };
    if let Err(error) = run(&mut f) {
        print!("{} ", error.exception_type());
    }
    int(f.calls);
    end();
}

pub fn advapprox_cut_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (a, b) = (r.real(), r.real());
    let (accepted, c) = AdvApproxDichoCutting::new().value(a, b);
    flag(accepted);
    real(c);
    match cutting(&mut r) {
        Ok(Some(tool)) => {
            let (accepted, c) = tool.value(a, b);
            flag(accepted);
            real(c);
        }
        Ok(None) => {}
        Err(error) => print!("{} ", error.exception_type()),
    }
    end();
}
