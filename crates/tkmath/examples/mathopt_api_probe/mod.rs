//! Differential twin of tools/oracle/tkmath_mathopt_api_probe.cxx: MathOpt::Utils,
//! PolishCoordinateWise and the modern Uzawa solver.
mod objective;
use objective::{Input, Objective, matrix_out, vector_out};
use rustycad_tkmath::math::{Matrix, MultipleVarFunction, Vector};
use rustycad_tkmath::math_opt::utils::*;
use rustycad_tkmath::math_opt::{
    UzawaConfig, UzawaResult, polish_coordinate_wise, uzawa, uzawa_equality,
};
use rustycad_tkmath::math_utils::{Config, LineSearchResult, Status};

fn flag(value: bool) -> i32 {
    i32::from(value)
}

fn status_from(code: usize) -> Status {
    const ALL: [Status; 11] = [
        Status::Ok,
        Status::NotConverged,
        Status::MaxIterations,
        Status::NumericalError,
        Status::InvalidInput,
        Status::InfiniteSolutions,
        Status::NoSolution,
        Status::NotPositiveDefinite,
        Status::Singular,
        Status::NonDescentDirection,
        Status::CallbackError,
    ];
    ALL[code]
}

pub fn mathopt_checks_case(values: &[f64]) {
    let mut input: Input<'_> = Input::new(values);
    let config: Config = Config {
        max_iterations: input.count() as u32,
        tolerance: input.real(),
        x_tolerance: input.real(),
        f_tolerance: input.real(),
        relative_tolerance: input.real(),
        step_min: input.real(),
    };
    let n: usize = input.count();
    let a: Vector<'static> = input.vector(n, 3);
    let b: Vector<'static> = input.vector(n, -2);
    let mut m: Matrix<'static> = Matrix::with_bounds(2, 3, -1, 0).unwrap();
    for row in 0..2 {
        for col in 0..2 {
            *m.change_at(row, col).unwrap() = input.real();
        }
    }
    let longer: Vector<'static> = Vector::filled(0, n as i32, 1.0).unwrap();
    println!(
        "{} {} {} {} {} {} {}",
        flag(is_valid_config(&config)),
        flag(is_finite_vector(&a)),
        flag(is_finite_matrix(&m)),
        flag(is_valid_bounds(&a, &b)),
        flag(is_valid_bounds(&a, &longer)),
        norm(&a).to_bits(),
        norm(&b).to_bits()
    );
}

pub fn mathopt_line_case(values: &[f64]) {
    let mut input: Input<'_> = Input::new(values);
    let first: LineSearchResult = LineSearchResult {
        status: status_from(input.count()),
        ..Default::default()
    };
    let second: LineSearchResult = LineSearchResult {
        status: status_from(input.count()),
        ..Default::default()
    };
    println!(
        "{} {} {} {}",
        flag(is_line_search_evaluation_error(&first)),
        line_search_failure_status(&first) as i32,
        line_search_fallback_failure_status(&first, &second) as i32,
        FRPR_WOLFE_CURVATURE.to_bits()
    );
}

fn evaluation_statuses(f: &mut Objective, x: &Vector<'_>, n: usize, value: &mut f64) {
    let status: Status = scalar_value_status(f, *x.at(0).unwrap(), value);
    print!("{} {} ", status as i32, value.to_bits());
    let status: Status = value_status(f, x, value);
    print!("{} {} ", status as i32, value.to_bits());
    let ok: bool = checked_value(f, x, value);
    print!("{} {} ", flag(ok), value.to_bits());
    let mut g: Vector<'static> = Vector::filled(2, 1 + n as i32, 91.0).unwrap();
    print!("{} ", gradient_status(f, x, &mut g) as i32);
    vector_out(&g);
    print!("{} ", flag(checked_gradient(f, x, &mut g)));
    let mut h: Matrix<'static> = Matrix::filled(1, n as i32, 1, n as i32, 91.0).unwrap();
    print!("{} ", hessian_status(f, x, &mut h) as i32);
    matrix_out(&h);
    print!("{} ", flag(checked_hessian(f, x, &mut h)));
}

fn checked_record(f: &mut Objective, x: &Vector<'_>, n: usize, value: &mut f64) {
    let bad: Vector<'static> = Vector::filled(1, n as i32, f64::NAN).unwrap();
    let mut checked = CheckedFunction::new(f);
    let first: bool = checked.value(x, value);
    let second: bool = checked.value(&bad, value);
    let third: bool = checked.value(x, value);
    print!(
        "{} {} {} {} {} {} ",
        flag(first),
        flag(second),
        flag(third),
        flag(checked.had_failure),
        flag(checked.has_valid_evaluation),
        checked.failure_status as i32
    );
    print!(
        "{} ",
        flag(MultipleVarFunction::value(checked.original(), x, value))
    );
}

pub fn mathopt_eval_case(values: &[f64]) {
    let mut input: Input<'_> = Input::new(values);
    let mut f: Objective = Objective::read(&mut input);
    let n: usize = input.count();
    let x: Vector<'static> = input.vector(n, 1);
    let lower: Vector<'static> = input.vector(n, 0);
    let upper: Vector<'static> = input.vector(n, 5);
    // One shared output slot, as the native case reuses a single `value` across calls.
    let mut value: f64 = 91.0;
    evaluation_statuses(&mut f, &x, n, &mut value);
    checked_record(&mut f, &x, n, &mut value);
    let mut bounded = BoundedFunction::new(&mut f, &lower, &upper);
    let ok: bool = bounded.value(&x, &mut value);
    print!("{} {} ", flag(ok), value.to_bits());
    f.flush();
    println!();
}

pub fn mathopt_stationary_case(values: &[f64]) {
    let mut input: Input<'_> = Input::new(values);
    let mut f: Objective = Objective::read(&mut input);
    let n: usize = input.count();
    let x: Vector<'static> = input.vector(n, 2);
    let fx: f64 = input.real();
    let config: Config = Config {
        x_tolerance: input.real(),
        f_tolerance: input.real(),
        ..Default::default()
    };
    let mut status: Status = Status::MaxIterations;
    let stationary: bool = is_stationary(&mut f, &x, fx, &config, &mut status);
    print!("{} {} ", flag(stationary), status as i32);
    f.flush();
    println!();
}

pub fn mathopt_backtrack_case(values: &[f64]) {
    let mut input: Input<'_> = Input::new(values);
    let mut f: Objective = Objective::read(&mut input);
    let n: usize = input.count();
    let x: Vector<'static> = input.vector(n, 1);
    let d: Vector<'static> = input.vector(n, 1);
    let g: Vector<'static> = input.vector(n, 1);
    let (fx, step, step_min): (f64, f64, f64) = (input.real(), input.real(), input.real());
    let r: LineSearchResult = backtrack(&mut f, &x, &d, &g, fx, step, step_min);
    print!(
        "{} {} {} {} {} ",
        r.status as i32,
        flag(r.is_valid),
        r.alpha.to_bits(),
        r.f_new.to_bits(),
        r.evaluations
    );
    f.flush();
    println!();
}

pub fn mathopt_regularize_case(values: &[f64]) {
    let mut input: Input<'_> = Input::new(values);
    let n: usize = input.count();
    let mut h: Matrix<'static> = Matrix::with_bounds(1, n as i32, 0, n as i32 - 1).unwrap();
    for row in 0..n {
        for col in 0..n {
            *h.change_at(row, col).unwrap() = input.real();
        }
    }
    let minimum: f64 = input.real();
    print!("{} ", flag(regularize_hessian(&mut h, minimum)));
    matrix_out(&h);
    println!();
}

pub fn mathopt_polish_case(values: &[f64]) {
    let mut input: Input<'_> = Input::new(values);
    let mut f: Objective = Objective::read(&mut input);
    let n: usize = input.count();
    let mut x: Vector<'static> = input.vector(n, 1);
    let lower: Vector<'static> = input.vector(n, 1);
    let upper: Vector<'static> = input.vector(n, 1);
    let mut value: f64 = input.real();
    let tolerance: f64 = input.real();
    let budget: usize = input.count();
    let mut evaluations: usize = 91;
    polish_coordinate_wise(
        &mut f,
        &mut x,
        &mut value,
        &lower,
        &upper,
        tolerance,
        budget,
        &mut evaluations,
    );
    vector_out(&x);
    print!("{} {} ", value.to_bits(), evaluations);
    f.flush();
    println!();
}

fn optional_out(vector: &Option<Vector<'static>>) {
    print!("{} ", flag(vector.is_some()));
    if let Some(vector) = vector {
        print!("{} ", vector.length());
        vector_out(vector);
    }
}

fn uzawa_out(r: &UzawaResult) {
    print!(
        "{} {} {} {} ",
        r.status as i32,
        flag(r.is_done()),
        flag(bool::from(r)),
        r.iterations
    );
    optional_out(&r.solution);
    optional_out(&r.dual);
    optional_out(&r.error);
    optional_out(&r.initial_error);
    print!("{} ", flag(r.inverse_ctc.is_some()));
    if let Some(inverse) = &r.inverse_ctc {
        matrix_out(inverse);
    }
    println!();
}

pub fn mathopt_uzawa_case(values: &[f64]) {
    let mut input: Input<'_> = Input::new(values);
    let (rows, cols, nce, nci): (usize, usize, usize, usize) =
        (input.count(), input.count(), input.count(), input.count());
    let config: UzawaConfig = UzawaConfig {
        eps_lix: input.real(),
        eps_lic: input.real(),
        max_iterations: input.count() as u32,
    };
    let equality: bool = input.count() != 0;
    let mut c: Matrix<'static> =
        Matrix::with_bounds(3, 2 + rows as i32, -1, cols as i32 - 2).unwrap();
    for row in 0..rows {
        for col in 0..cols {
            *c.change_at(row, col).unwrap() = input.real();
        }
    }
    let s: Vector<'static> = input.vector(rows, 2);
    let x0: Vector<'static> = input.vector(cols, -1);
    let result: UzawaResult = if equality {
        uzawa_equality(&c, &s, &x0, &config)
    } else {
        uzawa(&c, &s, &x0, nce, nci, &config)
    };
    uzawa_out(&result);
}
