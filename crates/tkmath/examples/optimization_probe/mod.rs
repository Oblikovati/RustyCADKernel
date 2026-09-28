fn bits(value: f64) -> u64 {
    value.to_bits()
}
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{
    Bfgs, Frpr, Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient,
    MultipleVarFunctionWithHessian, Vector,
};
struct Objective {
    record_input: bool,
    n: i32,
    mode: i32,
    offset: f64,
    fail: usize,
    calls: usize,
    states: usize,
    centers: Vec<f64>,
    weights: Vec<f64>,
    trace: Vec<f64>,
}
impl Objective {
    fn evaluate(&self, point: &Vector<'_>) -> f64 {
        let mut result: f64 = self.offset;
        for i in 1..=self.n {
            let d: f64 = point[i] - self.centers[(i - 1) as usize];
            result += self.weights[(i - 1) as usize] * d * d;
            if self.mode == 1 {
                result += 0.1 * d * d * d * d;
            }
        }
        result
    }
    fn begin(&mut self, kind: f64, point: &Vector<'_>, value: f64) {
        self.trace.push(kind);
        for i in 1..=self.n {
            self.trace.push(point[i]);
        }
        if self.record_input {
            self.trace.push(value);
        }
    }
    fn reset(&mut self, fail: usize) {
        self.calls = 0;
        self.states = 0;
        self.trace.clear();
        self.fail = fail;
    }
}
impl MultipleVarFunction for Objective {
    fn variables(&self) -> i32 {
        self.n
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        self.begin(1.0, point, *value);
        *value = self.evaluate(point);
        self.trace.push(*value);
        self.calls += 1;
        self.calls != self.fail
    }
    fn state_number(&mut self) -> i32 {
        self.states += 1;
        self.trace.push(3.0);
        29
    }
}
impl MultipleVarFunctionWithGradient for Objective {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        for i in 1..=self.n {
            let d: f64 = point[i] - self.centers[(i - 1) as usize];
            gradient[i] = 2.0 * self.weights[(i - 1) as usize] * d;
            if self.mode == 1 {
                gradient[i] += 0.4 * d * d * d;
            }
        }
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.begin(2.0, point, *value);
        *value = self.evaluate(point);
        self.gradient(point, gradient);
        self.trace.push(*value);
        for i in 1..=self.n {
            self.trace.push(gradient[i]);
        }
        self.calls += 1;
        self.calls != self.fail
    }
}
fn vector_result(result: Result<&Vector<'_>, Failure>) {
    match result {
        Ok(vector) => {
            for i in vector.lower()..=vector.upper() {
                print!("{} ", bits(vector[i]));
            }
        }
        Err(e) => print!("{} ", e.exception_type()),
    }
}
fn copied_result(result: Result<(), Failure>, vector: &Vector<'_>) {
    match result {
        Ok(()) => vector_result(Ok(vector)),
        Err(e) => print!("{} ", e.exception_type()),
    }
}
macro_rules! results {
    ($solver:expr,$function:expr) => {{
        let solver = $solver;
        let function = $function;
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
        vector_result(solver.gradient());
        let mut copy: Vector<'static> = Vector::filled(-3, function.n - 4, 71.0).unwrap();
        copied_result(solver.copy_location(&mut copy), &copy);
        copy.init(71.0);
        copied_result(solver.copy_gradient(&mut copy), &copy);
        print!(
            "{} {} {} ",
            function.calls,
            function.states,
            function.trace.len()
        );
        for &v in &function.trace {
            print!("{} ", bits(v));
        }
    }};
}
pub fn optimization_case(values: &[f64]) {
    let n: usize = values[0] as usize;
    let mut function: Objective = Objective {
        record_input: true,
        n: n as i32,
        mode: values[1] as i32,
        offset: values[2],
        fail: values[3] as usize,
        calls: 0,
        states: 0,
        centers: values[9..9 + n].to_vec(),
        weights: values[9 + n..9 + 2 * n].to_vec(),
        trace: Vec::new(),
    };
    let (limit, tolerance, epsilon, width, repeat): (i32, f64, f64, f64, bool) = (
        values[4] as i32,
        values[5],
        values[6],
        values[7],
        values[8] != 0.0,
    );
    let mut start: Vector<'static> = Vector::with_bounds(1, n as i32).unwrap();
    for i in 1..=n {
        start[i as i32] = values[8 + 2 * n + i];
    }
    let mut bfgs: Bfgs = Bfgs::with_options(n as i32, tolerance, limit, epsilon).unwrap();
    if width >= 0.0 {
        bfgs.set_boundary(
            &Vector::filled(1, n as i32, -width).unwrap(),
            &Vector::filled(1, n as i32, width).unwrap(),
        )
        .unwrap();
    }
    bfgs.perform(&mut function, &start).unwrap();
    results!(&bfgs, &function);
    if repeat {
        function.reset(1);
        bfgs.perform(&mut function, &start).unwrap();
        results!(&bfgs, &function);
    }
    function.reset(0);
    let mut frpr: Frpr = Frpr::with_options(&function, tolerance, limit, epsilon).unwrap();
    frpr.perform(&mut function, &start).unwrap();
    results!(&frpr, &function);
    if repeat {
        function.reset(1);
        frpr.perform(&mut function, &start).unwrap();
        results!(&frpr, &function);
    }
    extra::extra_optimization(
        &mut function,
        &start,
        limit,
        tolerance,
        epsilon,
        width,
        repeat,
    );
    println!();
}

impl MultipleVarFunctionWithHessian for Objective {
    fn values_hessian(
        &mut self,
        point: &Vector<'_>,
        value: &mut f64,
        gradient: &mut Vector<'_>,
        hessian: &mut Matrix<'_>,
    ) -> bool {
        self.trace.push(4.0);
        for i in 1..=self.n {
            self.trace.push(point[i]);
        }
        *value = self.evaluate(point);
        self.gradient(point, gradient);
        self.trace.push(*value);
        for i in 1..=self.n {
            self.trace.push(gradient[i]);
        }
        hessian.init(0.0);
        for i in 1..=self.n {
            let d: f64 = point[i] - self.centers[(i - 1) as usize];
            hessian[(i, i)] = 2.0 * self.weights[(i - 1) as usize];
            if self.mode == 1 {
                hessian[(i, i)] += 1.2 * d * d;
            }
        }
        for i in 1..=self.n {
            for j in 1..=self.n {
                self.trace.push(hessian[(i, j)]);
            }
        }
        self.calls += 1;
        self.calls != self.fail
    }
}
mod extra;
