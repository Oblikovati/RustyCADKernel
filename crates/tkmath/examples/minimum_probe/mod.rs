//! Minimum bracket outputs and callback traces.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{BracketMinimum, BrentMinimum, Function};
struct Bowl {
    center: f64,
    scale: f64,
    offset: f64,
    fail: i32,
    trace: Vec<(f64, f64)>,
}
impl Bowl {
    fn evaluate(&self, x: f64) -> f64 {
        self.scale * (x - self.center) * (x - self.center) + self.offset
    }
}
impl Function for Bowl {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        *result = self.evaluate(x);
        self.trace.push((x, *result));
        self.trace.len() as i32 != self.fail
    }
}
pub(super) fn minimum_case(values: &[f64]) {
    let mut function: Bowl = Bowl {
        center: values[2],
        scale: values[3],
        offset: values[4],
        fail: values[8] as i32,
        trace: Vec::new(),
    };
    let first: f64 = function.evaluate(values[0]);
    let second: f64 = function.evaluate(values[1]);
    let solver: BracketMinimum = match values[5] as i32 {
        0 => BracketMinimum::compute(&mut function, values[0], values[1]),
        1 => BracketMinimum::with_first_value(&mut function, values[0], values[1], first),
        2 => BracketMinimum::with_values(&mut function, values[0], values[1], first, second),
        _ => limited(&mut function, values),
    };
    print!("{} ", i32::from(solver.is_done()));
    triplet(solver.values());
    triplet(solver.function_values());
    print!("{} ", function.trace.len());
    for (x, value) in function.trace {
        print!("{} {} ", x.to_bits(), value.to_bits());
    }
    brent_values(values);
    println!();
}
fn limited(function: &mut Bowl, values: &[f64]) -> BracketMinimum {
    let mut solver: BracketMinimum = BracketMinimum::new(values[0], values[1]);
    solver.set_limits(values[6], values[7]);
    solver.perform(function);
    solver
}
fn triplet(result: Result<[f64; 3], Failure>) {
    match result {
        Ok(values) => {
            for value in values {
                print!("{} ", value.to_bits());
            }
        }
        Err(error) => print!("{} ", error.exception_type()),
    }
}

fn brent_values(values: &[f64]) {
    let mut function: Bowl = Bowl {
        center: values[2],
        scale: values[3],
        offset: values[4],
        fail: values[8] as i32,
        trace: Vec::new(),
    };
    let middle: f64 = 0.5 * (values[0] + values[1]);
    let limit: i32 = if values[5] == 1.0 { 2 } else { 100 };
    let mut solver: BrentMinimum = if values[5] == 2.0 {
        BrentMinimum::with_value(1e-10, function.evaluate(middle), limit, 1e-12)
    } else {
        BrentMinimum::with_options(1e-10, limit, 1e-12)
    };
    solver.perform(&mut function, values[0], middle, values[1]);
    print_brent(&solver, &function);
    function.trace.clear();
    function.fail = 1;
    solver.perform(&mut function, values[0], values[0], values[1]);
    print_brent(&solver, &function);
}
fn print_brent(solver: &BrentMinimum, function: &Bowl) {
    print!("{} ", i32::from(solver.is_done()));
    scalar(solver.location());
    scalar(solver.minimum());
    match solver.iterations() {
        Ok(count) => print!("{count} "),
        Err(error) => print!("{} ", error.exception_type()),
    }
    print!("{} ", function.trace.len());
    for (x, value) in &function.trace {
        print!("{} {} ", x.to_bits(), value.to_bits());
    }
}
fn scalar(result: Result<f64, Failure>) {
    match result {
        Ok(value) => print!("{} ", value.to_bits()),
        Err(error) => print!("{} ", error.exception_type()),
    }
}
