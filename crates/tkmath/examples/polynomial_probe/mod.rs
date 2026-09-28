use rustycad_tkmath::math::DirectPolynomialRoots;
fn results(solver: &DirectPolynomialRoots, initialized_count: bool) {
    print!(
        "{} {} ",
        i32::from(solver.is_done()),
        i32::from(solver.infinite_roots())
    );
    if !initialized_count && !solver.is_done() && !solver.infinite_roots() {
        return;
    }
    let mut count: i32 = 0;
    match solver.solutions() {
        Ok(n) => {
            count = n;
            print!("{n} ");
        }
        Err(e) => print!("{} ", e.exception_type()),
    }
    for index in 1..=count {
        print!("{} ", solver.value(index).unwrap().to_bits());
    }
    for index in [0, count + 1] {
        match solver.value(index) {
            Ok(value) => print!("{} ", value.to_bits()),
            Err(e) => print!("{} ", e.exception_type()),
        }
    }
}
pub(super) fn polynomial_case(values: &[f64]) {
    let mut solver: DirectPolynomialRoots = match values[0] as i32 {
        1 => DirectPolynomialRoots::linear(values[1], values[2]),
        2 => DirectPolynomialRoots::quadratic(values[1], values[2], values[3]),
        3 => DirectPolynomialRoots::cubic(values[1], values[2], values[3], values[4]),
        _ => DirectPolynomialRoots::quartic(values[1], values[2], values[3], values[4], values[5]),
    };
    results(&solver, false);
    if solver.is_done() && !solver.infinite_roots() {
        solver.solve_cubic(1.0, 1e60, 1.0, 1.0);
        results(&solver, true);
    }
    solver.solve_linear(0.0, 0.0);
    results(&solver, false);
    solver.solve_linear(1.0, -2.0);
    results(&solver, false);
    solver.solve_quadratic(1.0, -5.0, 6.0);
    results(&solver, false);
    solver.solve_cubic(1.0, -6.0, 11.0, -6.0);
    results(&solver, false);
    solver.solve_quartic(1.0, 0.0, -5.0, 0.0, 4.0);
    results(&solver, false);
    println!();
}
