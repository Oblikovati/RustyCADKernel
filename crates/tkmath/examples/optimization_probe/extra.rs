use super::{Objective, bits, copied_result, vector_result};
use rustycad_tkmath::math::{Matrix, NewtonMinimum, Powell, Vector};
fn powell_results(solver: &Powell, function: &Objective) {
    print!("{} ", i32::from(solver.is_done()));
    match solver.minimum() {
        Ok(v) => print!("{} ", bits(v)),
        Err(e) => print!("{} ", e.exception_type()),
    }
    match solver.iterations() {
        Ok(v) => print!("{v} "),
        Err(e) => print!("{} ", e.exception_type()),
    }
    vector_result(solver.location());
    let mut copy: Vector<'static> = Vector::filled(-3, function.n - 4, 71.0).unwrap();
    copied_result(solver.copy_location(&mut copy), &copy);
    print!(
        "{} {} {} ",
        function.calls,
        function.states,
        function.trace.len()
    );
    for &v in &function.trace {
        print!("{} ", bits(v));
    }
}
pub(super) fn extra_optimization(
    function: &mut Objective,
    start: &Vector<'_>,
    limit: i32,
    tolerance: f64,
    epsilon: f64,
    width: f64,
    repeat: bool,
) {
    let n: i32 = function.n;
    function.reset((n % 3) as usize);
    function.record_input = false;
    let mut directions: Matrix<'static> = Matrix::with_bounds(1, n, 1, n).unwrap();
    for i in 1..=n {
        for j in 1..=n {
            directions[(i, j)] = if i == j {
                1.0
            } else {
                0.125 * f64::from(i - j)
            };
        }
    }
    let mut powell: Powell = Powell::with_options(function, tolerance, limit, epsilon).unwrap();
    powell.perform(function, start, &directions).unwrap();
    powell_results(&powell, function);
    if repeat {
        function.reset(1);
        powell.perform(function, start, &directions).unwrap();
        powell_results(&powell, function);
    }
    function.reset((n % 4) as usize);
    if n % 2 == 0 {
        function.weights[0] = -function.weights[0];
    }
    if n == 3 {
        function.weights[0] = 0.0;
    }
    let convexity: f64 = if n == 5 { 100.0 } else { 1e-6 };
    let mut solver: NewtonMinimum =
        NewtonMinimum::with_options(function, tolerance, limit, convexity, repeat).unwrap();
    if width >= 0.0 {
        solver
            .set_boundary(
                &Vector::filled(1, n, -width).unwrap(),
                &Vector::filled(1, n, width).unwrap(),
            )
            .unwrap();
    }
    solver.perform(function, start).unwrap();
    print!(
        "{} {} ",
        solver.status() as i32,
        i32::from(solver.is_converged())
    );
    results!(&solver, &*function);
    if repeat {
        function.reset(0);
        solver.perform(function, start).unwrap();
        print!(
            "{} {} ",
            solver.status() as i32,
            i32::from(solver.is_converged())
        );
        results!(&solver, &*function);
    }
}
