//! math_IntegerVector, dump and extended math_Matrix oracle counterpart.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Xy, Xyz};
use rustycad_tkmath::math::{IntegerVector, Matrix, Vector, VectorBase, VectorItem};
use std::fmt::Display;
pub(super) fn values<T: VectorItem + Display>(vector: &VectorBase<'_, T>) {
    print!("{} {} {} ", vector.lower(), vector.upper(), vector.length());
    for index in vector.lower()..=vector.upper() {
        print!("{} ", vector[index]);
    }
}
pub(super) fn reals(vector: &Vector<'_>) {
    print!("{} {} {} ", vector.lower(), vector.upper(), vector.length());
    for index in vector.lower()..=vector.upper() {
        print!("{} ", vector[index].to_bits());
    }
}
pub(super) fn cells(matrix: &Matrix<'_>) {
    let bounds = [
        matrix.lower_row(),
        matrix.upper_row(),
        matrix.lower_col(),
        matrix.upper_col(),
    ];
    print!("{} {} {} {} ", bounds[0], bounds[1], bounds[2], bounds[3]);
    for row in bounds[0]..=bounds[1] {
        for col in bounds[2]..=bounds[3] {
            print!("{} ", matrix[(row, col)].to_bits());
        }
    }
}
pub(super) fn dumped(value: &impl Display) {
    print!("{} ", value.to_string().replace('\n', "|"));
}
pub(super) fn guarded(body: impl FnOnce() -> Result<(), Failure>) {
    if let Err(error) = body() {
        print!("{} ", error.exception_type());
    }
}
fn read_matrix(v: &[f64], rows: i32, cols: i32, bounds: [i32; 2], start: usize) -> Matrix<'static> {
    let upper = [bounds[0] + rows - 1, bounds[1] + cols - 1];
    let mut matrix = Matrix::with_bounds(bounds[0], upper[0], bounds[1], upper[1]).unwrap();
    for row in 0..rows as usize {
        for col in 0..cols as usize {
            *matrix.change_at(row, col).unwrap() = v[start + row * cols as usize + col];
        }
    }
    matrix
}
pub(super) fn int_vector_case(v: &[f64]) {
    let (n, lower) = (v[0] as i32, v[1] as i32);
    let mut vector = IntegerVector::with_bounds(lower, lower + n - 1).unwrap();
    for offset in 0..n as usize {
        *vector.change_at(offset).unwrap() = v[2 + offset] as i32;
    }
    let (scalar, divisor) = (v[2 + n as usize] as i32, v[3 + n as usize] as i32);
    let matrix = read_matrix(v, n, 2, [1, 1], 4 + n as usize);
    int_scalars(&vector, scalar, divisor);
    int_pairs(&vector, scalar);
    int_products(&vector, &matrix);
    values(&vector.slice(vector.upper(), vector.lower()).unwrap());
    dumped(&vector);
    let array = vector.array1();
    print!("{} {} {} ", array.lower(), array.upper(), vector.size());
    values(&IntegerVector::with_size_filled(n as usize, scalar));
    values(&IntegerVector::from_xy(Xy::new(
        v[2 + n as usize],
        v[3 + n as usize],
    )));
    println!();
}
fn int_scalars(vector: &IntegerVector<'_>, scalar: i32, divisor: i32) {
    print!(
        "{} {} {} {} ",
        vector.norm().unwrap().to_bits(),
        vector.norm2().unwrap().to_bits(),
        vector.max_index(),
        vector.min_index()
    );
    guarded(|| vector.normalized().map(|result| values(&result)));
    guarded(|| vector.inverse().map(|result| values(&result)));
    values(&vector.multiplied(scalar));
    values(&vector.t_multiplied(scalar));
    values(&(scalar * vector));
    guarded(|| vector.divided(divisor).map(|result| values(&result)));
    guarded(|| {
        let mut copy = vector.clone();
        copy.divide(divisor)?;
        values(&copy);
        Ok(())
    });
}
fn int_pairs(vector: &IntegerVector<'_>, scalar: i32) {
    let doubled = vector.multiplied(2);
    values(&vector.opposite());
    values(&vector.added(&doubled).unwrap());
    values(&vector.subtracted(&doubled).unwrap());
    print!("{} ", vector.dot(&doubled).unwrap());
    let mut output = IntegerVector::with_bounds(3, 3 + vector.length() - 1).unwrap();
    guarded(|| {
        output.set_scaled(scalar, vector)?;
        values(&output);
        Ok(())
    });
    output.set_sum(vector, &doubled).unwrap();
    values(&output);
    output.set_difference(vector, &vector.opposite()).unwrap();
    values(&output);
}
fn int_products(vector: &IntegerVector<'_>, matrix: &Matrix<'_>) {
    let mut output = IntegerVector::with_bounds(4, 5).unwrap();
    output.set_vector_matrix(vector, matrix).unwrap();
    values(&output);
    output.set_transpose_matrix_vector(matrix, vector).unwrap();
    values(&output);
    values(&vector.multiplied_matrix(matrix).unwrap());
    let transposed = matrix.transposed();
    output.set_matrix_vector(&transposed, vector).unwrap();
    values(&output);
    output
        .set_vector_transpose_matrix(vector, &transposed)
        .unwrap();
    values(&output);
}
pub(super) fn real_vector_extra_case(v: &[f64]) {
    let (n, lower) = (v[0] as i32, v[1] as i32);
    let mut vector = Vector::with_bounds(lower, lower + n - 1).unwrap();
    for offset in 0..n as usize {
        *vector.change_at(offset).unwrap() = v[2 + offset];
    }
    let scalar = v[2 + n as usize];
    reals(&vector.t_multiplied(scalar));
    reals(&(scalar * &vector));
    dumped(&vector);
    let array = vector.array1();
    print!("{} {} {} ", array.lower(), array.upper(), vector.size());
    reals(&Vector::with_size_filled(n as usize, scalar));
    reals(&Vector::from_xyz(Xyz::new(scalar, -scalar, 2.0 * scalar)));
    println!();
}
pub(super) fn matrix_extra_case(v: &[f64]) {
    let (rows, cols) = (v[0] as i32, v[1] as i32);
    let matrix = read_matrix(v, rows, cols, [v[2] as i32, v[3] as i32], 4);
    let tail = 4 + (rows * cols) as usize;
    let (scalar, block_row, block_col) = (v[tail], v[tail + 1] as i32, v[tail + 2] as i32);
    matrix_scalars(&matrix, scalar);
    matrix_sums(&matrix, scalar);
    matrix_products(&matrix);
    matrix_edits(&matrix, scalar, block_row, block_col);
    dumped(&matrix);
    println!();
}
fn matrix_scalars(matrix: &Matrix<'_>, scalar: f64) {
    cells(&matrix.opposite());
    cells(&-matrix);
    cells(&matrix.t_multiplied(scalar));
    cells(&(scalar * matrix));
    cells(&(matrix * scalar));
}
fn matrix_sums(matrix: &Matrix<'_>, scalar: f64) {
    let mut copy = matrix.clone();
    copy.add(&matrix.opposite()).unwrap();
    cells(&copy);
    copy.subtract(matrix).unwrap();
    cells(&copy);
    guarded(|| {
        copy.add(&matrix.transposed())?;
        cells(&copy);
        Ok(())
    });
    let (rows, cols) = (matrix.row_number(), matrix.col_number());
    let mut sum = Matrix::with_bounds(-1, -2 + rows, 5, 4 + cols).unwrap();
    sum.set_sum(matrix, &matrix.multiplied(scalar)).unwrap();
    cells(&sum);
    sum.set_difference(matrix, &matrix.multiplied(scalar))
        .unwrap();
    cells(&sum);
    guarded(|| sum.set_sum(matrix, &matrix.transposed()));
}
fn matrix_products(matrix: &Matrix<'_>) {
    let (rows, cols) = (matrix.row_number(), matrix.col_number());
    cells(&matrix.t_multiply(matrix).unwrap());
    guarded(|| matrix.t_multiply(&matrix.transposed()).map(|m| cells(&m)));
    let mut product = Matrix::with_bounds(2, 1 + rows, -3, -4 + rows).unwrap();
    product.set_product(matrix, &matrix.transposed()).unwrap();
    cells(&product);
    guarded(|| {
        product.set_product(matrix, matrix)?;
        cells(&product);
        Ok(())
    });
    let mut t_product = Matrix::with_bounds(0, cols - 1, 7, 6 + cols).unwrap();
    t_product.set_transpose_product(matrix, matrix).unwrap();
    cells(&t_product);
    guarded(|| t_product.set_transpose_product(&matrix.transposed(), matrix));
}
fn matrix_edits(matrix: &Matrix<'_>, scalar: f64, block_row: i32, block_col: i32) {
    let block = Matrix::filled(1, 1, 1, 1, scalar).unwrap();
    let mut target = matrix.clone();
    guarded(|| target.set(block_row, block_row, block_col, block_col, &block));
    cells(&target);
    guarded(|| target.set(block_row, block_row + 1, block_col, block_col, &block));
    let mut swapped = matrix.clone();
    guarded(|| swapped.transpose());
    cells(&swapped);
    let (rows, cols) = (matrix.row_number() as usize, matrix.col_number() as usize);
    cells(&Matrix::with_size_filled(rows, cols, scalar));
    print!("{} {} ", matrix.row_size(), matrix.col_size());
    let mut rescaled = matrix.clone();
    guarded(|| rescaled.divide(scalar));
    cells(&rescaled);
}
