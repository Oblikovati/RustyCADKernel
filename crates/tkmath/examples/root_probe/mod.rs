//! Callback traces and bracketed-root arithmetic compared with native OCCT.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{
    BissecNewton, BracketedRoot, Function, FunctionRoot, FunctionWithDerivative, NewtonFunctionRoot,
};
macro_rules! scalar_results {
    ($solver:expr,$function:expr) => {{
        let solver = $solver;
        let function = $function;
        print!("{} ", i32::from(solver.is_done()));
        print_result(solver.root());
        print_result(solver.value());
        print_result(solver.derivative());
        match solver.iterations() {
            Ok(count) => print!("{count} "),
            Err(error) => print!("{} ", error.exception_type()),
        }
        print!("{} ", function.trace.len());
        for (x, value) in &function.trace {
            print!("{} {} ", x.to_bits(), value.to_bits());
        }
    }};
}
struct Polynomial {
    coefficients: [f64; 4],
    success: bool,
    trace: Vec<(f64, f64)>,
}
impl Function for Polynomial {
    fn state_number(&mut self) -> i32 {
        print!("scalar-state {} ", self.trace.len());
        self.trace.len() as i32
    }

    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        let [a, b, c, d]: [f64; 4] = self.coefficients;
        *result = ((a * x + b) * x + c) * x + d;
        self.trace.push((x, *result));
        self.success
    }
}
impl FunctionWithDerivative for Polynomial {
    fn derivative(&mut self, x: f64, result: &mut f64) -> bool {
        let [a, b, c, _]: [f64; 4] = self.coefficients;
        *result = (3.0 * a * x + 2.0 * b) * x + c;
        self.success
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        self.value(x, value);
        self.derivative(x, derivative)
    }
}
pub(super) fn root_case(values: &[f64]) {
    let mut function: Polynomial = Polynomial {
        coefficients: values[6..10].try_into().unwrap(),
        success: values[5] != 0.0,
        trace: Vec::new(),
    };
    let solver: BracketedRoot = BracketedRoot::with_options(
        &mut function,
        values[0],
        values[1],
        values[2],
        values[3] as i32,
        values[4],
    );
    print!("{} ", i32::from(solver.is_done()));
    print_result(solver.root());
    print_result(solver.value());
    match solver.iterations() {
        Ok(count) => print!("{count} "),
        Err(error) => print!("{} ", error.exception_type()),
    }
    print!("{} ", function.trace.len());
    for (x, value) in function.trace {
        print!("{} {} ", x.to_bits(), value.to_bits());
    }
    bissec_values(values);
    function_root_values(values, false);
    function_root_values(values, true);
    newton_root_values(values);
    println!();
}
fn print_result(result: Result<f64, Failure>) {
    match result {
        Ok(value) => print!("{} ", value.to_bits()),
        Err(error) => print!("{} ", error.exception_type()),
    }
}

fn bissec_values(values: &[f64]) {
    let mut function: Polynomial = Polynomial {
        coefficients: values[6..10].try_into().unwrap(),
        success: values[5] != 0.0,
        trace: Vec::new(),
    };
    let mut solver: BissecNewton = BissecNewton::new(values[2]);
    solver.perform(&mut function, values[0], values[1], values[3] as i32);
    print!("{} ", i32::from(solver.is_done()));
    print_result(solver.root());
    print_result(solver.value());
    print_result(solver.derivative());
    print!("{} ", function.trace.len());
    for (x, value) in function.trace {
        print!("{} {} ", x.to_bits(), value.to_bits());
    }
}

fn function_root_values(values: &[f64], bounded: bool) {
    let mut function: Polynomial = Polynomial {
        coefficients: values[6..10].try_into().unwrap(),
        success: values[5] != 0.0,
        trace: Vec::new(),
    };
    let solver: FunctionRoot = if bounded {
        FunctionRoot::bounded(
            &mut function,
            values[0],
            values[2],
            values[0],
            values[1],
            values[3] as i32,
        )
        .unwrap()
    } else {
        FunctionRoot::with_iterations(&mut function, values[0], values[2], values[3] as i32)
            .unwrap()
    };
    scalar_results!(&solver, &function);
}

fn newton_root_values(values: &[f64]) {
    let mut function: Polynomial = Polynomial {
        coefficients: values[6..10].try_into().unwrap(),
        success: values[5] != 0.0,
        trace: Vec::new(),
    };
    let unbounded: NewtonFunctionRoot = NewtonFunctionRoot::with_iterations(
        &mut function,
        values[0],
        values[2],
        values[4],
        values[3] as i32,
    );
    scalar_results!(&unbounded, &function);
    function.trace.clear();
    let mut bounded: NewtonFunctionRoot = NewtonFunctionRoot::bounded(
        &mut function,
        values[0],
        values[2],
        values[4],
        values[0],
        values[1],
        values[3] as i32,
    );
    scalar_results!(&bounded, &function);
    function.trace.clear();
    bounded.perform(&mut function, values[1]);
    scalar_results!(&bounded, &function);
}
