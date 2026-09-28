//! Dynamic matrix and Gaussian elimination oracle counterpart.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{Crout, Gauss, Jacobi, Matrix, Vector};
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
fn checked_matrix(result: Result<Matrix<'_>, Failure>) {
    match result {
        Ok(matrix) => matrix_values(&matrix),
        Err(error) => print!("{} ", error.exception_type()),
    }
}
fn vector_values(vector: &Vector<'_>) {
    print!("{} {} {} ", vector.lower(), vector.upper(), vector.length());
    for index in vector.lower()..=vector.upper() {
        print!("{} ", vector[index].to_bits());
    }
}
pub(super) fn numeric_matrix_case(v: &[f64]) {
    let (n, rl, cl) = (v[0] as i32, v[1] as i32, v[2] as i32);
    let mut matrix = Matrix::with_bounds(rl, rl + n - 1, cl, cl + n - 1).unwrap();
    for row in 0..n {
        for col in 0..n {
            matrix[(row + rl, col + cl)] = v[5 + (row * n + col) as usize];
        }
    }

    let mut rhs = Vector::with_bounds(-3, -3 + n - 1).unwrap();
    for index in 0..n {
        *rhs.change_at(index as usize).unwrap() = v[5 + (n * n + index) as usize];
    }
    jacobi_values(&matrix);
    crout_values(&matrix, &rhs, v[3]);
    arithmetic(&matrix, v[4]);
    solve(&matrix, &mut rhs, v[3]);
    print!("{} ", matrix.determinant().unwrap().to_bits());
    checked_matrix(matrix.inverse());
    println!();
}
fn arithmetic(matrix: &Matrix<'_>, scalar: f64) {
    let transpose = matrix.transposed();
    matrix_values(&transpose);
    checked_matrix(matrix.added(&transpose));
    checked_matrix(matrix.subtracted(&transpose));
    matrix_values(&matrix.multiplied(scalar));
    checked_matrix(matrix.divided(scalar));
    let mut divided = matrix.clone();
    match divided.divide(scalar) {
        Ok(()) => matrix_values(&divided),
        Err(error) => print!("{} ", error.exception_type()),
    }
    checked_matrix(matrix.multiplied_matrix(&transpose));
    let mut square = matrix.clone();
    square.square().unwrap();
    matrix_values(&square);
    let mut inplace = matrix.clone();
    if let Err(error) = inplace.multiply_matrix(&transpose) {
        print!("{} ", error.exception_type());
    }
    matrix_values(&inplace);
}
fn solve(matrix: &Matrix<'_>, rhs: &mut Vector<'_>, minimum: f64) {
    let solver = Gauss::with_min_pivot(matrix, minimum).unwrap();
    print!("{} ", i32::from(solver.is_done()));
    match solver.determinant() {
        Ok(value) => print!("{} ", value.to_bits()),
        Err(error) => print!("{} ", error.exception_type()),
    }
    let n = rhs.length();
    let mut solution = Vector::filled(5, 5 + n - 1, 91.0).unwrap();
    match solver.solve_into(rhs, &mut solution) {
        Ok(()) => vector_values(&solution),
        Err(error) => print!("{} ", error.exception_type()),
    }
    match solver.solve(rhs) {
        Ok(()) => vector_values(rhs),
        Err(error) => print!("{} ", error.exception_type()),
    }
    let mut inverse = Matrix::with_bounds(2, 2 + n - 1, 4, 4 + n - 1).unwrap();
    match solver.invert(&mut inverse) {
        Ok(()) => matrix_values(&inverse),
        Err(error) => print!("{} ", error.exception_type()),
    }
}

fn lower_matrix_values(matrix: &Matrix<'_>) {
    for row in 0..matrix.row_number() {
        for col in 0..=row {
            print!(
                "{} ",
                matrix.at(row as usize, col as usize).unwrap().to_bits()
            );
        }
    }
}
fn crout_values(matrix: &Matrix<'_>, rhs: &Vector<'_>, minimum: f64) {
    let solver = Crout::with_min_pivot(matrix, minimum).unwrap();
    print!("{} ", i32::from(solver.is_done()));
    match solver.determinant() {
        Ok(value) => print!("{} ", value.to_bits()),
        Err(error) => print!("{} ", error.exception_type()),
    }
    match solver.inverse() {
        Ok(inverse) => lower_matrix_values(inverse),
        Err(error) => print!("{} ", error.exception_type()),
    }
    let n = rhs.length();
    let mut solution = Vector::filled(5, 5 + n - 1, 91.0).unwrap();
    match solver.solve_into(rhs, &mut solution) {
        Ok(()) => vector_values(&solution),
        Err(error) => print!("{} ", error.exception_type()),
    }
    let mut aliased = rhs.clone();
    match solver.solve_in_place(&mut aliased) {
        Ok(()) => vector_values(&aliased),
        Err(error) => print!("{} ", error.exception_type()),
    }
    let mut inverse = Matrix::with_bounds(2, 2 + n - 1, 4, 4 + n - 1).unwrap();
    match solver.invert(&mut inverse) {
        Ok(()) => lower_matrix_values(&inverse),
        Err(error) => print!("{} ", error.exception_type()),
    }
}

fn jacobi_values(matrix: &Matrix<'_>) {
    let solver: Jacobi = Jacobi::new(matrix).unwrap();
    print!("{} ", i32::from(solver.is_done()));
    if !solver.is_done() {
        return;
    }
    vector_values(solver.values().unwrap());
    matrix_values(solver.vectors().unwrap());
    let mut vector: Vector<'static> = Vector::with_bounds(5, 4 + matrix.row_number()).unwrap();
    for index in 1..=matrix.row_number() {
        print!("{} ", solver.value(index).unwrap().to_bits());
        solver.vector(index, &mut vector).unwrap();
        vector_values(&vector);
    }
}
