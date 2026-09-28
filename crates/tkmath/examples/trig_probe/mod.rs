use rustycad_tkmath::math::{
    Function, FunctionWithDerivative, TrigonometricEquation, TrigonometricRoots,
};
use std::f64::consts::PI;
fn results(solver: &TrigonometricRoots) {
    print!(
        "{} {} ",
        i32::from(solver.is_done()),
        i32::from(solver.infinite_roots())
    );
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
            Ok(v) => print!("{} ", v.to_bits()),
            Err(e) => print!("{} ", e.exception_type()),
        }
    }
}
pub(super) fn trig_case(values: &[f64]) {
    let (mode, mut coefficients, lower, upper): (i32, [f64; 5], f64, f64) = (
        values[0] as i32,
        values[1..6].try_into().unwrap(),
        values[6],
        values[7],
    );
    let [a, b, c, d, e]: [f64; 5] = coefficients;
    let mut solver: TrigonometricRoots = match mode {
        0 => {
            coefficients = [0.0, 0.0, 0.0, d, e];
            TrigonometricRoots::sine(d, e, lower, upper)
        }
        1 => {
            coefficients = [0.0, 0.0, c, d, e];
            TrigonometricRoots::linear(c, d, e, lower, upper)
        }
        _ => TrigonometricRoots::new(a, b, c, d, e, lower, upper),
    };
    results(&solver);
    solver.perform(coefficients, [upper, lower]);
    results(&solver);
    solver.perform([0.0; 5], [0.0, 2.0 * PI]);
    results(&solver);
    solver.perform([0.0, 0.0, 0.0, 1.0, 0.0], [0.0, 2.0 * PI]);
    results(&solver);
    equation_values(coefficients, [0.0, 0.731, lower, upper]);
    println!();
}
fn equation_values([a, b, c, d, e]: [f64; 5], points: [f64; 4]) {
    let mut function: TrigonometricEquation = TrigonometricEquation::new(a, b, c, d, e);
    let (mut value, mut derivative): (f64, f64) = (91.0, 92.0);
    for x in points {
        let good: bool = function.value(x, &mut value);
        print!("{} {} ", i32::from(good), value.to_bits());
        let good: bool = function.derivative(x, &mut derivative);
        print!("{} {} ", i32::from(good), derivative.to_bits());
        let good: bool = function.values(x, &mut value, &mut derivative);
        print!(
            "{} {} {} ",
            i32::from(good),
            value.to_bits(),
            derivative.to_bits()
        );
    }
}
