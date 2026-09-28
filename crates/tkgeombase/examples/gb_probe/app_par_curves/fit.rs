// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! The Rust twin of tools/oracle/tkgeombase_probe_appparcurves_fit.cxx.
use super::super::common::{self as p, Reader, attempt, end};
use super::super::fem_tool::{read_vector, vector};
use super::line::ProbeLine;
use super::{curve, ivector, matrix};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::app_par_curves::{
    BSpFunction, BSpGradient, BSpGradientInput, Constraint, ConstraintCouple, Constraints,
    Function, Gradient, LeastSquare, LineTool, MultiCurveEval, ResolConstraint,
};
use rustycad_tkmath::math::Vector;

fn constraint(code: i32) -> Constraint {
    Constraint::from_code(code).expect("probe constraint")
}

fn read_constraints(r: &mut Reader<'_>) -> Constraints {
    let n = r.integer();
    let mut couples = Array1::with_bounds(1, n).unwrap();
    for i in 1..=n {
        let index = r.integer();
        couples[i] = ConstraintCouple::new(index, constraint(r.integer()));
    }
    Handle::new(HArray1::from_owned(couples))
}

fn read_knots(r: &mut Reader<'_>) -> Array1<'static, f64> {
    let n = r.integer();
    let mut k = Array1::with_bounds(1, n).unwrap();
    (1..=n).for_each(|i| k[i] = r.real());
    k
}

fn read_mults(r: &mut Reader<'_>) -> Array1<'static, i32> {
    let n = r.integer();
    let mut m = Array1::with_bounds(1, n).unwrap();
    (1..=n).for_each(|i| m[i] = r.integer());
    m
}

fn ls_results(ls: &mut LeastSquare, bspline: bool, lambdas: i32) {
    p::flag(ls.is_done());
    attempt(|| ls.function_matrix().map(matrix));
    attempt(|| ls.derivative_function_matrix().map(matrix));
    ivector(ls.k_index());
    matrix(ls.points());
    if !ls.is_done() {
        return;
    }
    matrix(ls.poles());
    attempt(|| {
        let (f, e3, e2) = ls.error()?;
        [f, e3, e2].into_iter().for_each(p::real);
        Ok(())
    });
    matrix(ls.distance());
    attempt(|| {
        let mut g = Vector::filled(ls.points().lower_row(), ls.points().upper_row(), -7.)?;
        let (f, e3, e2) = ls.error_gradient(&mut g)?;
        vector(&g);
        [f, e3, e2].into_iter().for_each(p::real);
        Ok(())
    });
    matrix(ls.distance());
    if lambdas & 1 != 0 {
        p::real(ls.first_lambda());
    }
    if lambdas & 2 != 0 {
        p::real(ls.last_lambda());
    }
    attempt(|| curve(&ls.bezier_value()?));
    attempt(|| {
        let c = ls.bspline_value()?;
        curve(c)?;
        if bspline {
            p::int(MultiCurveEval::degree(c)?);
        }
        Ok(())
    });
}

struct LsRequest {
    line: ProbeLine,
    points: [i32; 2],
    cons: [Constraint; 2],
    nbpol: i32,
    mode: i32,
    knots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
    u: Vector<'static>,
    variant: i32,
    vectors: Vec<Vector<'static>>,
    lambdas: [f64; 2],
}

fn read_ls(r: &mut Reader<'_>) -> LsRequest {
    let line = ProbeLine::read(r);
    let points = [r.integer(), r.integer()];
    let cons = [constraint(r.integer()), constraint(r.integer())];
    let (nbpol, mode) = (r.integer(), r.integer());
    let (knots, mults) = if mode >= 2 {
        (read_knots(r), read_mults(r))
    } else {
        (
            Array1::with_bounds(1, 1).unwrap(),
            Array1::with_bounds(1, 1).unwrap(),
        )
    };
    let u = read_vector(r);
    let variant = r.integer();
    let count = match variant {
        2 => 2,
        3 => 4,
        _ => 0,
    };
    let vectors = (0..count).map(|_| read_vector(r)).collect();
    let lambdas = if variant > 0 {
        [r.real(), r.real()]
    } else {
        [0., 0.]
    };
    LsRequest {
        line,
        points,
        cons,
        nbpol,
        mode,
        knots,
        mults,
        u,
        variant,
        vectors,
        lambdas,
    }
}

fn run_ls(q: &LsRequest, lambdas: i32) -> Result<(), Failure> {
    let [fp, lp] = q.points;
    let [fc, lc] = q.cons;
    let mut ls = match q.mode {
        0 => LeastSquare::new(&q.line, fp, lp, fc, lc, &q.u, q.nbpol)?,
        1 => LeastSquare::without_parameters(&q.line, fp, lp, fc, lc, q.nbpol)?,
        2 => LeastSquare::with_knots(&q.line, &q.knots, &q.mults, fp, lp, fc, lc, &q.u, q.nbpol)?,
        _ => LeastSquare::with_knots_without_parameters(
            &q.line, &q.knots, &q.mults, fp, lp, fc, lc, q.nbpol,
        )?,
    };
    if q.mode == 1 || q.mode == 3 {
        let [l1, l2] = q.lambdas;
        match q.variant {
            0 => ls.perform(&q.u)?,
            1 => ls.perform_lambdas(&q.u, l1, l2)?,
            2 => ls.perform_tangents(&q.u, &q.vectors[0], &q.vectors[1], l1, l2)?,
            _ => {
                let v = &q.vectors;
                ls.perform_curvatures(&q.u, [&v[0], &v[1], &v[2], &v[3]], l1, l2)?
            }
        }
    }
    ls_results(&mut ls, q.mode >= 2, lambdas);
    Ok(())
}

pub fn ls_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let q = read_ls(&mut r);
    let lambdas = r.integer();
    attempt(|| run_ls(&q, lambdas));
    end();
}

pub fn resol_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let line = ProbeLine::read(&mut r);
    let (fp, lp) = (r.integer(), r.integer());
    let (fc, lc) = (constraint(r.integer()), constraint(r.integer()));
    let cons = read_constraints(&mut r);
    let nbpol = r.integer();
    let u = read_vector(&mut r);
    let tol = r.real();
    attempt(|| {
        let mut ls = LeastSquare::new(&line, fp, lp, fc, lc, &u, nbpol)?;
        let mut c = ls.bezier_value()?;
        let (a, da) = (
            ls.function_matrix()?.clone(),
            ls.derivative_function_matrix()?.clone(),
        );
        let mut resol = ResolConstraint::new(&line, &mut c, fp, lp, &cons, &a, &da, tol)?;
        p::flag(resol.is_done());
        matrix(resol.constraint_matrix());
        if resol.is_done() {
            matrix(resol.inverse_matrix());
            vector(resol.duale());
        }
        curve(&c)?;
        curve(&ls.bezier_value()?)?;
        attempt(|| {
            matrix(resol.constraint_derivative(&line, &u, nbpol - 1, &da)?);
            matrix(resol.constraint_matrix());
            Ok(())
        });
        Ok(())
    });
    end();
}

/// The calls shared by Function and BSpFunction.
trait ProbeFunction {
    fn value(&mut self, x: &Vector<'_>, f: &mut f64) -> Result<bool, Failure>;
    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> Result<bool, Failure>;
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> Result<bool, Failure>;
    fn print_curve(&mut self) -> Result<(), Failure>;
    fn error(&mut self, i: i32, k: i32) -> Result<f64, Failure>;
    fn errors(&self) -> [f64; 2];
    fn parameters(&self) -> &Vector<'static>;
    fn extra(&mut self, code: i32) -> Result<(), Failure>;
}

impl ProbeFunction for Function<ProbeLine> {
    fn value(&mut self, x: &Vector<'_>, f: &mut f64) -> Result<bool, Failure> {
        self.evaluate(x, f)
    }
    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> Result<bool, Failure> {
        self.evaluate_gradient(x, g)
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> Result<bool, Failure> {
        self.evaluate_values(x, f, g)
    }
    fn print_curve(&mut self) -> Result<(), Failure> {
        curve(self.curve_value()?)
    }
    fn error(&mut self, i: i32, k: i32) -> Result<f64, Failure> {
        Function::error(self, i, k)
    }
    fn errors(&self) -> [f64; 2] {
        [self.max_error_3d(), self.max_error_2d()]
    }
    fn parameters(&self) -> &Vector<'static> {
        self.new_parameters()
    }
    fn extra(&mut self, _: i32) -> Result<(), Failure> {
        Ok(())
    }
}

impl ProbeFunction for BSpFunction<ProbeLine> {
    fn value(&mut self, x: &Vector<'_>, f: &mut f64) -> Result<bool, Failure> {
        self.evaluate(x, f)
    }
    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> Result<bool, Failure> {
        self.evaluate_gradient(x, g)
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> Result<bool, Failure> {
        self.evaluate_values(x, f, g)
    }
    fn print_curve(&mut self) -> Result<(), Failure> {
        let c = self.curve_value()?;
        curve(&c)?;
        p::int(MultiCurveEval::degree(&c)?);
        Ok(())
    }
    fn error(&mut self, i: i32, k: i32) -> Result<f64, Failure> {
        BSpFunction::error(self, i, k)
    }
    fn errors(&self) -> [f64; 2] {
        [self.max_error_3d(), self.max_error_2d()]
    }
    fn parameters(&self) -> &Vector<'static> {
        self.new_parameters()
    }
    fn extra(&mut self, code: i32) -> Result<(), Failure> {
        match code {
            8 => matrix(self.function_matrix()?),
            9 => matrix(self.derivative_function_matrix()?),
            _ => ivector(self.index()),
        }
        Ok(())
    }
}

fn function_call(
    r: &mut Reader<'_>,
    f: &mut dyn ProbeFunction,
    [first, last, curves]: [i32; 3],
) -> Result<(), Failure> {
    let code = r.integer();
    match code {
        1..=3 => {
            let x = read_vector(r);
            let (mut value, mut g) = (-7., Vector::filled(first, last, -7.)?);
            match code {
                1 => {
                    p::flag(f.value(&x, &mut value)?);
                    p::real(value);
                }
                2 => {
                    p::flag(f.gradient(&x, &mut g)?);
                    vector(&g);
                }
                _ => {
                    p::flag(f.values(&x, &mut value, &mut g)?);
                    p::real(value);
                    vector(&g);
                }
            }
        }
        4 => f.print_curve()?,
        5 => {
            for i in first..=last {
                for k in 1..=curves {
                    p::real(f.error(i, k)?);
                }
            }
        }
        6 => f.errors().into_iter().for_each(p::real),
        7 => vector(f.parameters()),
        _ => f.extra(code)?,
    }
    Ok(())
}

fn function_calls(r: &mut Reader<'_>, f: &mut dyn ProbeFunction, bounds: [i32; 3]) {
    for _ in 0..r.integer() {
        attempt(|| function_call(r, f, bounds));
    }
}

pub fn function_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let line = ProbeLine::read(&mut r);
    let (fp, lp) = (r.integer(), r.integer());
    let cons = read_constraints(&mut r);
    let u = read_vector(&mut r);
    let deg = r.integer();
    let curves = line.nb_p3d() + line.nb_p2d();
    attempt(|| {
        let mut f = Function::new(&line, fp, lp, &cons, &u, deg)?;
        p::int(f.nb_variables());
        function_calls(&mut r, &mut f, [fp, lp, curves]);
        Ok(())
    });
    end();
}

pub fn bspfunction_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let line = ProbeLine::read(&mut r);
    let (fp, lp) = (r.integer(), r.integer());
    let cons = read_constraints(&mut r);
    let u = read_vector(&mut r);
    let (knots, mults) = (read_knots(&mut r), read_mults(&mut r));
    let nbpol = r.integer();
    let (l1, l2) = (r.real(), r.real());
    let curves = line.nb_p3d() + line.nb_p2d();
    attempt(|| {
        let mut f = BSpFunction::new(&line, fp, lp, &cons, &u, &knots, &mults, nbpol)?;
        p::int(f.nb_variables());
        f.set_first_lambda(l1);
        f.set_last_lambda(l2);
        function_calls(&mut r, &mut f, [fp, lp, curves]);
        Ok(())
    });
    end();
}

pub fn gradient_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let line = ProbeLine::read(&mut r);
    let (fp, lp) = (r.integer(), r.integer());
    let cons = read_constraints(&mut r);
    let mut u = read_vector(&mut r);
    let (deg, t3, t2, iterations) = (r.integer(), r.real(), r.real(), r.integer());
    attempt(|| {
        let g = Gradient::new(&line, fp, lp, &cons, &mut u, deg, t3, t2, iterations)?;
        p::flag(g.is_done());
        curve(&g.value())?;
        let errors = (fp..=lp)
            .map(|j| g.error(j))
            .collect::<Result<Vec<_>, _>>()?;
        gradient_results_tail(
            errors,
            [g.average_error(), g.max_error_3d(), g.max_error_2d()],
            &u,
        );
        Ok(())
    });
    end();
}

pub fn bspgradient_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let line = ProbeLine::read(&mut r);
    let (fp, lp) = (r.integer(), r.integer());
    let cons = read_constraints(&mut r);
    let mut u = read_vector(&mut r);
    let (knots, mults) = (read_knots(&mut r), read_mults(&mut r));
    let (deg, t3, t2, iterations, with) = (r.integer(), r.real(), r.real(), r.integer(), r.flag());
    let (l1, l2) = (r.real(), r.real());
    attempt(|| {
        let input = BSpGradientInput {
            ssp: &line,
            points: [fp, lp],
            constraints: &cons,
            knots: &knots,
            mults: &mults,
            deg,
            tolerances: [t3, t2],
            nb_iterations: iterations,
        };
        let g = if with {
            BSpGradient::with_lambdas(&input, &mut u, l1, l2)?
        } else {
            BSpGradient::new(&input, &mut u)?
        };
        p::flag(g.is_done());
        curve(&g.value())?;
        let errors = (fp..=lp)
            .map(|j| g.error(j))
            .collect::<Result<Vec<_>, _>>()?;
        gradient_results_tail(
            errors,
            [g.average_error(), g.max_error_3d(), g.max_error_2d()],
            &u,
        );
        p::int(MultiCurveEval::degree(&g.value())?);
        Ok(())
    });
    end();
}

fn gradient_results_tail(errors: Vec<f64>, stats: [f64; 3], u: &Vector<'_>) {
    errors.into_iter().for_each(p::real);
    stats.into_iter().for_each(p::real);
    vector(u);
}
