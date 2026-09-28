//! Least-squares bounds, output tails, and aliasing oracle counterpart.
use rustycad_tkmath::math::{GaussLeastSquares, Householder, Matrix, Vector};
fn vector_values(vector: &Vector<'_>) {
    print!("{} {} {} ", vector.lower(), vector.upper(), vector.length());
    for index in vector.lower()..=vector.upper() {
        print!("{} ", vector[index].to_bits());
    }
}
pub(super) fn least_squares_case(v: &[f64]) {
    let (rows, cols, rl, cl) = (v[0] as i32, v[1] as i32, v[2] as i32, v[3] as i32);
    let mut matrix = Matrix::with_bounds(rl, rl + rows - 1, cl, cl + cols - 1).unwrap();
    for row in 0..rows {
        for col in 0..cols {
            matrix[(rl + row, cl + col)] = v[5 + (row * cols + col) as usize];
        }
    }
    let mut rhs = Vector::filled(-3, -3 + rows + 1, 91.0).unwrap();
    for row in 0..rows {
        *rhs.change_at(row as usize).unwrap() = v[5 + (rows * cols + row) as usize];
    }
    householder_cases(&matrix, &rhs, v[4]);
    let solver = GaussLeastSquares::with_min_pivot(&matrix, v[4]).unwrap();
    print!("{} ", i32::from(solver.is_done()));
    solutions(&solver, &rhs, cols);
    println!();
}
fn solutions(solver: &GaussLeastSquares, rhs: &Vector<'_>, cols: i32) {
    let mut solution = Vector::filled(5, 5 + cols - 1, 91.0).unwrap();
    if let Err(error) = solver.solve_into(rhs, &mut solution) {
        print!("{} ", error.exception_type());
    }
    vector_values(&solution);
    let mut oversized = Vector::filled(2, 2 + cols + 1, 91.0).unwrap();
    if let Err(error) = solver.solve_into(rhs, &mut oversized) {
        print!("{} ", error.exception_type());
    }
    vector_values(&oversized);
    let mut aliased = rhs.clone();
    if let Err(error) = solver.solve_in_place(&mut aliased) {
        print!("{} ", error.exception_type());
    }
    vector_values(&aliased);
}

fn householder_values(solver: &Householder, columns: i32) {
    print!("{} ", i32::from(solver.is_done()));
    match solver.all_values() {
        Ok(matrix) => matrix_values(matrix),
        Err(error) => print!("{} ", error.exception_type()),
    }
    let mut solution = Vector::filled(5, 5 + columns - 1, 91.0).unwrap();
    if let Err(error) = solver.value(&mut solution, 1) {
        print!("{} ", error.exception_type());
    }
    vector_values(&solution);
}
fn matrix_values(matrix: &Matrix<'_>) {
    print!(
        "{} {} {} {} ",
        matrix.lower_row(),
        matrix.upper_row(),
        matrix.lower_col(),
        matrix.upper_col()
    );
    for row in matrix.lower_row()..=matrix.upper_row() {
        for col in matrix.lower_col()..=matrix.upper_col() {
            print!("{} ", matrix[(row, col)].to_bits());
        }
    }
}
fn householder_cases(matrix: &Matrix<'_>, rhs: &Vector<'_>, epsilon: f64) {
    let vector_solver = Householder::from_vector(matrix, rhs, epsilon).unwrap();
    householder_values(&vector_solver, matrix.col_number());
    let mut rebased =
        Matrix::with_bounds(-4, -4 + matrix.row_number() - 1, 1, matrix.col_number()).unwrap();
    rebased.initialized(matrix).unwrap();
    let mut multiple = Matrix::with_bounds(-4, -4 + matrix.row_number() - 1, 1, 2).unwrap();
    for row in 0..matrix.row_number() {
        multiple[(row - 4, 1)] = *rhs.at(row as usize).unwrap();
        multiple[(row - 4, 2)] = rhs.at(row as usize).unwrap() * 0.5 + f64::from(row);
    }
    let matrix_solver = Householder::from_matrix(&rebased, &multiple, epsilon).unwrap();
    householder_values(&matrix_solver, matrix.col_number());
    let bounds = [
        matrix.lower_row(),
        matrix.upper_row(),
        matrix.lower_col(),
        matrix.upper_col(),
    ];
    let range_solver = Householder::from_range(matrix, &multiple, bounds, epsilon).unwrap();
    householder_values(&range_solver, matrix.col_number());
    submatrix_case(matrix, &multiple, bounds, epsilon);
}

fn submatrix_case(matrix: &Matrix<'_>, rhs: &Matrix<'_>, bounds: [i32; 4], epsilon: f64) {
    let mut padded = Matrix::filled(
        bounds[0] - 1,
        bounds[1] + 1,
        bounds[2] - 1,
        bounds[3] + 1,
        91.0,
    )
    .unwrap();
    for row in bounds[0]..=bounds[1] {
        for col in bounds[2]..=bounds[3] {
            padded[(row, col)] = matrix[(row, col)];
        }
    }
    let solver = Householder::from_range(&padded, rhs, bounds, epsilon).unwrap();
    householder_values(&solver, matrix.col_number());
}
