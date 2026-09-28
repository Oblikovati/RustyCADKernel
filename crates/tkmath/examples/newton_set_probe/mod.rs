//! Newton system callbacks, LU mutation, clipping, and state capture.
use rustycad_tkmath::math::{
    FunctionSet, FunctionSetRoot, FunctionSetWithDerivatives, Matrix, NewtonFunctionSetRoot, Vector,
};
struct PolynomialSystem {
    linear: Matrix<'static>,
    quadratic: Vector<'static>,
    rhs: Vector<'static>,
    calls: i32,
    fail: i32,
}
impl FunctionSet for PolynomialSystem {
    fn variables(&self) -> i32 {
        self.linear.col_number()
    }
    fn equations(&self) -> i32 {
        self.rhs.length()
    }
    fn value(&mut self, point: &Vector<'_>, residual: &mut Vector<'_>) -> bool {
        print!("value ");
        print_vector(point);
        for row in 1..=self.equations() {
            let mut sum: f64 = 0.0;
            for col in 1..=self.variables() {
                sum += self.linear[(row, col)] * point[col];
            }
            residual[row] = sum
                + self.quadratic[row]
                    * point[(row - 1) % self.variables() + 1]
                    * point[(row - 1) % self.variables() + 1]
                - self.rhs[row];
        }
        true
    }
    fn state_number(&mut self) -> i32 {
        print!("state {} ", self.calls);
        self.calls
    }
}
impl FunctionSetWithDerivatives for PolynomialSystem {
    fn derivatives(&mut self, point: &Vector<'_>, jacobian: &mut Matrix<'_>) -> bool {
        jacobian.initialized(&self.linear).unwrap();
        for index in 1..=self.equations() {
            let col: i32 = (index - 1) % self.variables() + 1;
            jacobian[(index, col)] += 2.0 * self.quadratic[index] * point[col];
        }
        true
    }
    fn values(
        &mut self,
        point: &Vector<'_>,
        residual: &mut Vector<'_>,
        jacobian: &mut Matrix<'_>,
    ) -> bool {
        self.calls += 1;
        print!("call {} ", self.calls);
        print_vector(point);
        if self.calls > 1 {
            print_matrix(jacobian);
        }
        self.value(point, residual);
        self.derivatives(point, jacobian);
        self.calls != self.fail
    }
}
pub(super) fn newton_set_case(values: &[f64]) {
    let n: i32 = values[0] as i32;
    let mut function: PolynomialSystem = read_function(values, n);
    let tolerance: Vector<'static> = Vector::filled(1, n, values[1]).unwrap();
    let mut solver: NewtonFunctionSetRoot =
        NewtonFunctionSetRoot::new(&function, &tolerance, values[2], values[3] as i32).unwrap();
    let mut start: Vector<'static> = Vector::with_bounds(-3, n - 4).unwrap();
    for index in 0..n {
        start[index - 3] = values[6 + (n * n + 2 * n + index) as usize];
    }
    let lower: Vector<'static> = Vector::filled(1, n, -values[4]).unwrap();
    let upper: Vector<'static> = Vector::filled(1, n, values[4]).unwrap();
    solver
        .perform_bounded(&mut function, &start, &lower, &upper)
        .unwrap();
    print!("done {} ", i32::from(solver.is_done()));
    if solver.is_done() {
        print_results(&solver, n);
    }
    println!();
}
fn read_function(values: &[f64], n: i32) -> PolynomialSystem {
    let mut linear: Matrix<'static> = Matrix::with_bounds(1, n, 1, n).unwrap();
    for row in 0..n {
        for col in 0..n {
            linear[(row + 1, col + 1)] = values[6 + (row * n + col) as usize];
        }
    }
    let mut quadratic: Vector<'static> = Vector::with_bounds(1, n).unwrap();
    let mut rhs: Vector<'static> = Vector::with_bounds(1, n).unwrap();
    for index in 0..n {
        quadratic[index + 1] = values[6 + (n * n + index) as usize];
        rhs[index + 1] = values[6 + (n * n + n + index) as usize];
    }
    PolynomialSystem {
        linear,
        quadratic,
        rhs,
        calls: 0,
        fail: values[5] as i32,
    }
}
fn print_results(solver: &NewtonFunctionSetRoot, n: i32) {
    print!("{} ", solver.iterations().unwrap());
    print_vector(solver.root().unwrap());
    print_vector(solver.function_set_errors().unwrap());
    print_matrix(solver.derivative().unwrap());
    let mut vector: Vector<'static> = Vector::with_bounds(5, n + 4).unwrap();
    let mut matrix: Matrix<'static> = Matrix::with_bounds(2, n + 1, 4, n + 3).unwrap();
    solver.copy_root(&mut vector).unwrap();
    print_vector(&vector);
    solver.copy_function_set_errors(&mut vector).unwrap();
    print_vector(&vector);
    solver.copy_derivative(&mut matrix).unwrap();
    print_matrix(&matrix);
}
fn print_vector(vector: &Vector<'_>) {
    for index in vector.lower()..=vector.upper() {
        print!("{} ", vector[index].to_bits());
    }
}
fn print_matrix(matrix: &Matrix<'_>) {
    for row in matrix.lower_row()..=matrix.upper_row() {
        for col in matrix.lower_col()..=matrix.upper_col() {
            print!("{} ", matrix[(row, col)].to_bits());
        }
    }
}

pub(super) fn function_set_case(values: &[f64]) {
    let (n, m): (i32, i32) = (values[0] as i32, values[1] as i32);
    let mut function: PolynomialSystem = read_rectangular(values, n, m);
    let tolerance: Vector<'static> = Vector::filled(1, n, values[2]).unwrap();
    let mut solver: FunctionSetRoot =
        FunctionSetRoot::new(&function, &tolerance, values[3] as i32).unwrap();
    let mut start: Vector<'static> = Vector::with_bounds(-3, n - 4).unwrap();
    for index in 0..n {
        start[index - 3] = values[7 + (m * n + 2 * m + index) as usize];
    }
    let lower: Vector<'static> = Vector::filled(1, n, -values[4]).unwrap();
    let upper: Vector<'static> = Vector::filled(1, n, values[4]).unwrap();
    solver
        .perform_bounded(&mut function, &start, &lower, &upper, values[6] != 0.0)
        .unwrap();
    print!(
        "done {} {} ",
        i32::from(solver.is_done()),
        i32::from(solver.is_divergent())
    );
    if solver.is_done() {
        print!(
            "{} {} ",
            solver.iterations().unwrap(),
            solver.state_number().unwrap()
        );
        print_vector(solver.root().unwrap());
        if function.calls > 1 {
            print_vector(solver.function_set_errors().unwrap());
        }
        print_matrix(solver.derivative().unwrap());
    }
    println!();
}
fn read_rectangular(values: &[f64], n: i32, m: i32) -> PolynomialSystem {
    let mut linear: Matrix<'static> = Matrix::with_bounds(1, m, 1, n).unwrap();
    for row in 0..m {
        for col in 0..n {
            linear[(row + 1, col + 1)] = values[7 + (row * n + col) as usize];
        }
    }
    let mut quadratic: Vector<'static> = Vector::with_bounds(1, m).unwrap();
    let mut rhs: Vector<'static> = Vector::with_bounds(1, m).unwrap();
    for index in 0..m {
        quadratic[index + 1] = values[7 + (m * n + index) as usize];
        rhs[index + 1] = values[7 + (m * n + m + index) as usize];
    }
    PolynomialSystem {
        linear,
        quadratic,
        rhs,
        calls: 0,
        fail: values[5] as i32,
    }
}
