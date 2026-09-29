//! FairCurve probe operations, the Rust twin of tools/oracle/tkgeomalgo_probe_faircurve.cxx.
use super::common::{Reader, end, flag, int, pnt2d, real};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeomalgo::fair_curve::*;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::math::{FunctionSet, Matrix, MultipleVarFunctionWithHessian, Vector};

/// `h slope sliding h2 slope2 sliding2 n t*n`: the law before and after its setters.
pub fn law_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (h, s, l) = (r.real(), r.real(), r.real());
    let (h2, s2, l2) = (r.real(), r.real(), r.real());
    let ts = r.reals();
    let mut law = FairCurveBattenLaw::new(h, s, l);
    let print = |law: &FairCurveBattenLaw| {
        for &t in &ts {
            let (positive, value) = law.height(t);
            flag(positive);
            real(value);
        }
    };
    print(&law);
    law.set_heigth(h2);
    law.set_slope(s2);
    law.set_sliding(l2);
    print(&law);
    end();
}

/// The spline of a request: flat knots, shared poles and order.
struct Spline {
    flat: Handle<HArray1<f64>>,
    poles: Handle<HArray1<Point2>>,
    order: i32,
}

fn spline(r: &mut Reader) -> Spline {
    let nk = r.integer();
    let mut knots: Array1<'static, f64> = Array1::with_bounds(1, nk).expect("probe knots");
    let mut mults: Array1<'static, i32> = Array1::with_bounds(1, nk).expect("probe mults");
    for i in 1..=nk {
        knots[i] = r.real();
    }
    let mut total = 0;
    for i in 1..=nk {
        mults[i] = r.integer();
        total += mults[i];
    }
    let np = r.integer();
    let mut poles: Array1<'static, Point2> = Array1::with_bounds(1, np).expect("probe poles");
    for i in 1..=np {
        poles[i] = r.pnt2d();
    }
    let order = total - np;
    let mut flat: Array1<'static, f64> = Array1::with_bounds(1, total).expect("probe flat");
    bsplclib::knot_sequence_with_degree(&knots, &mults, order - 1, false, &mut flat)
        .expect("probe flat knots");
    Spline {
        flat: Handle::new(HArray1::from_owned(flat)),
        poles: Handle::new(HArray1::from_owned(poles)),
        order,
    }
}

fn print_poles(poles: &Handle<HArray1<Point2>>) {
    let poles = poles.get().expect("probe poles").array1();
    for i in poles.lower()..=poles.upper() {
        pnt2d(poles[i]);
    }
}

fn print_vector(v: &Vector<'_>) {
    for i in v.lower()..=v.upper() {
        real(v[i]);
    }
}

/// A density as the native `FairCurve_DistributionOfEnergy*`.
enum Density {
    Tension(FairCurveDistributionOfTension),
    Sagging(FairCurveDistributionOfSagging),
    Jerk(FairCurveDistributionOfJerk),
}

impl Density {
    fn base(&mut self) -> &mut FairCurveDistributionOfEnergy {
        match self {
            Density::Tension(f) => f,
            Density::Sagging(f) => f,
            Density::Jerk(f) => f,
        }
    }
    fn value(&mut self, t: &Vector<'_>, v: &mut Vector<'_>) -> bool {
        match self {
            Density::Tension(f) => f.value(t, v),
            Density::Sagging(f) => f.value(t, v),
            Density::Jerk(f) => f.value(t, v),
        }
    }
    fn evaluate(&mut self, t: f64) {
        let t = Vector::filled(1, 1, t).expect("probe parameter");
        let mut v = Vector::filled(1, self.base().nb_equations(), 0.0).expect("probe values");
        flag(self.value(&t, &mut v));
        print_vector(&v);
    }
}

/// `kind spline derivative aux h slope sliding lsliding uniform d2 n t*n`: a density.
pub fn density_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let s = spline(&mut r);
    let (d, aux) = (r.integer(), r.integer());
    let (h, slope, sl, ls) = (r.real(), r.real(), r.real(), r.real());
    let uniform = r.flag();
    let d2 = r.integer();
    let ts = r.reals();
    let law = FairCurveBattenLaw::new(h, slope, sl);
    let (order, flat, poles) = (s.order, &s.flat, &s.poles);
    let mut f = match kind {
        0 => Density::Tension(FairCurveDistributionOfTension::new(
            order, flat, poles, d, ls, &law, aux, uniform,
        )),
        1 => Density::Sagging(FairCurveDistributionOfSagging::new(
            order, flat, poles, d, &law, aux,
        )),
        _ => Density::Jerk(FairCurveDistributionOfJerk::new(
            order, flat, poles, d, &law, aux,
        )),
    };
    int(f.base().nb_variables());
    int(f.base().nb_equations());
    for &t in &ts {
        f.evaluate(t);
    }
    f.base().set_derivative_order(d2);
    int(f.base().nb_equations());
    if let Some(&t) = ts.first() {
        f.evaluate(t);
    }
    end();
}

fn evaluate<E: FairCurveEnergyModel>(e: &mut E, x: &Vector<'_>) {
    let n = e.nb_variables();
    let mut v = 0.0;
    flag(FairCurveEnergyModel::value(e, x, &mut v));
    real(v);
    let mut g = Vector::filled(1, n, 0.0).expect("probe gradient");
    flag(FairCurveEnergyModel::values(e, x, &mut v, &mut g));
    real(v);
    print_vector(&g);
    let mut hm = Matrix::filled(1, n, 1, n, 0.0).expect("probe hessian");
    let mut g2 = Vector::filled(1, n, 0.0).expect("probe gradient");
    flag(e.values_hessian(x, &mut v, &mut g2, &mut hm));
    real(v);
    print_vector(&g2);
    for i in 1..=n {
        for j in 1..=n {
            real(hm[(i, j)]);
        }
    }
}

/// The accessors the native report reads from both energies.
trait EnergyState: FairCurveEnergyModel + MultipleVarFunctionWithHessian {
    fn state(&self) -> (FairCurveAnalysisCode, f64);
}
impl EnergyState for FairCurveEnergyOfBatten {
    fn state(&self) -> (FairCurveAnalysisCode, f64) {
        (self.status(), self.length_sliding())
    }
}
impl EnergyState for FairCurveEnergyOfMvc {
    fn state(&self) -> (FairCurveAnalysisCode, f64) {
        (self.status(), self.length_sliding())
    }
}

fn print_state<E: EnergyState>(e: &E, s: &Spline) {
    let (status, length) = e.state();
    int(status as i32);
    real(length);
    print_poles(&s.poles);
}

fn energy_report<E: EnergyState>(
    e: &mut E,
    s: &Spline,
    pert: f64,
    newton: Option<[f64; 4]>,
) -> Result<(), Failure> {
    let n = FairCurveEnergyModel::nb_variables(e);
    int(n);
    let mut x = Vector::filled(1, n, 0.0)?;
    flag(FairCurveEnergyModel::variable(e, &mut x));
    print_vector(&x);
    evaluate(e, &x);
    let (status, length) = e.state();
    int(status as i32);
    real(length);
    let mut x2 = Vector::filled(1, n, 0.0)?;
    for k in 1..=n {
        x2[k] = x[k] * (1.0 + pert * f64::from(k));
    }
    evaluate(e, &x2);
    print_state(e, s);
    let Some(nt) = newton else {
        return Ok(());
    };
    let mut solver = FairCurveNewton::new(e, nt[0], nt[1], nt[2] as i32, nt[3], true)?;
    solver.perform(e, &x)?;
    flag(solver.is_done());
    flag(solver.is_converged());
    if solver.is_done() {
        print_vector(solver.location()?);
        real(solver.minimum()?);
    }
    print_state(e, s);
    Ok(())
}

/// `kind spline o1 o2 free a1 a2 c1 c2 h slope sliding lsliding ratio pert newton
/// sptol tol nbiter convexity`: an energy of batten (kind 0) or of MVC (kind 1).
pub fn energy_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let s = spline(&mut r);
    let orders = [r.integer(), r.integer()];
    let free = r.flag();
    let angles = [r.real(), r.real()];
    let curvatures = [r.real(), r.real()];
    let (h, slope, sl, ls) = (r.real(), r.real(), r.real(), r.real());
    let (ratio, pert) = (r.real(), r.real());
    let newton = r.flag();
    let nt = [r.real(), r.real(), r.real(), r.real()];
    let law = FairCurveBattenLaw::new(h, slope, sl);
    let newton = newton.then_some(nt);
    let run = || -> Result<(), Failure> {
        if kind == 0 {
            let mut e = FairCurveEnergyOfBatten::new(
                s.order, &s.flat, &s.poles, orders, &law, ls, free, angles,
            );
            energy_report(&mut e, &s, pert, newton)
        } else {
            let mut e = FairCurveEnergyOfMvc::new(
                s.order, &s.flat, &s.poles, orders, &law, ratio, ls, free, angles, curvatures,
            )?;
            energy_report(&mut e, &s, pert, newton)
        }
    };
    if let Err(error) = run() {
        print!("{} ", error.exception_type());
    }
    end();
}

/// A curve as the native `FairCurve_Batten*`.
enum Curve {
    Batten(FairCurveBatten),
    Mvc(FairCurveMinimalVariation),
}

impl Curve {
    fn batten(&mut self) -> &mut FairCurveBatten {
        match self {
            Curve::Batten(b) => b,
            Curve::Mvc(m) => m,
        }
    }
    fn compute(
        &mut self,
        code: &mut FairCurveAnalysisCode,
        nb: i32,
        tol: f64,
    ) -> Result<bool, Failure> {
        match self {
            Curve::Batten(b) => b.compute(code, nb, tol),
            Curve::Mvc(m) => m.compute(code, nb, tol),
        }
    }
    fn dump(&self, o: &mut Vec<u8>) -> std::io::Result<()> {
        match self {
            Curve::Batten(b) => b.dump(o),
            Curve::Mvc(m) => m.dump(o),
        }
    }
    fn report(&mut self) {
        let b: &FairCurveBatten = self.batten();
        pnt2d(b.get_p1());
        pnt2d(b.get_p2());
        for v in [
            b.get_angle1(),
            b.get_angle2(),
            b.get_height(),
            b.get_slope(),
        ] {
            real(v);
        }
        real(b.get_sliding_factor());
        flag(b.get_free_sliding());
        int(b.get_constraint_order1());
        int(b.get_constraint_order2());
        if let Curve::Mvc(m) = self {
            real(m.get_curvature1());
            real(m.get_curvature2());
            real(m.get_physical_ratio());
        }
        let b: &FairCurveBatten = self.batten();
        real(b.sliding_of_reference());
        if let Err(error) = print_curve(b) {
            print!("{} ", error.exception_type());
        }
        let mut text: Vec<u8> = Vec::new();
        self.dump(&mut text).expect("probe dump");
        let s: String = String::from_utf8_lossy(&text)
            .replace('\n', "|")
            .replace(' ', "_");
        print!("\"{s}\" ");
    }
}

fn print_curve(b: &FairCurveBatten) -> Result<(), Failure> {
    let c = b.curve()?;
    let c = c.get().expect("probe curve");
    int(c.degree());
    let poles = c.poles();
    int(poles.len() as i32);
    for i in poles.lower()..=poles.upper() {
        pnt2d(poles[i]);
    }
    int(c.nb_knots());
    for i in 1..=c.nb_knots() {
        real(c.knot(i)?);
        int(c.multiplicity(i)?);
    }
    Ok(())
}

/// Sets the requested constraints the native probe sets, in its order.
fn configure(curve: &mut Curve, r: &mut Reader, ratio: f64) -> (i32, f64, i32) {
    let free = r.flag();
    let (o1, o2) = (r.integer(), r.integer());
    let (a1, a2, factor, c1, c2) = (r.real(), r.real(), r.real(), r.real(), r.real());
    let h2 = r.real();
    let (nbiter, tol, moves) = (r.integer(), r.real(), r.integer());
    let b = curve.batten();
    b.set_free_sliding(free);
    b.set_constraint_order1(o1);
    b.set_constraint_order2(o2);
    b.set_angle1(a1);
    b.set_angle2(a2);
    b.set_sliding_factor(factor);
    if let Curve::Mvc(m) = curve {
        m.set_curvature1(c1);
        m.set_curvature2(c2);
        if let Err(error) = m.set_physical_ratio(ratio) {
            print!("{} ", error.exception_type());
        }
    }
    if let Err(error) = curve.batten().set_height(h2) {
        print!("{} ", error.exception_type());
    }
    (nbiter, tol, moves)
}

/// `kind p1 p2 height slope ratio free o1 o2 a1 a2 factor c1 c2 height2 nbiter tol
/// moves (p1 p2)*moves`: a batten or minimal-variation curve and its moves.
pub fn batten_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let (p1, p2) = (r.pnt2d(), r.pnt2d());
    let (h, slope, ratio) = (r.real(), r.real(), r.real());
    let built = if kind == 0 {
        FairCurveBatten::new(p1, p2, h, slope).map(Curve::Batten)
    } else {
        FairCurveMinimalVariation::new(p1, p2, h, slope, ratio).map(Curve::Mvc)
    };
    let mut curve = match built {
        Ok(curve) => curve,
        Err(error) => {
            print!("{} ", error.exception_type());
            end();
            return;
        }
    };
    let (nbiter, tol, moves) = configure(&mut curve, &mut r, ratio);
    for m in 0..=moves {
        if m > 0 {
            let (q1, q2) = (r.pnt2d(), r.pnt2d());
            for result in [curve.batten().set_p1(q1), curve.batten().set_p2(q2)] {
                if let Err(error) = result {
                    print!("{} ", error.exception_type());
                }
            }
        }
        let mut code = FairCurveAnalysisCode::Ok;
        match curve.compute(&mut code, nbiter, tol) {
            Ok(ok) => {
                flag(ok);
                int(code as i32);
            }
            Err(error) => print!("{} ", error.exception_type()),
        }
        curve.report();
    }
    end();
}
