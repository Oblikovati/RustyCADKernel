//! Exact bounded-vector arithmetic and storage oracle counterpart.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{Matrix, Vector};
fn vector_values(vector: &Vector<'_>) {
    print!("{} {} {} ", vector.lower(), vector.upper(), vector.length());
    for index in vector.lower()..=vector.upper() {
        print!("{} ", vector[index].to_bits());
    }
}
fn checked_vector(result: Result<Vector<'_>, Failure>) {
    match result {
        Ok(vector) => vector_values(&vector),
        Err(error) => print!("{} ", error.exception_type()),
    }
}
pub(super) fn numeric_vector_case(v: &[f64]) {
    let (length, lower) = (v[0] as i32, v[1] as i32);
    let mut left = Vector::with_bounds(lower, lower + length - 1).unwrap();
    let mut right = Vector::with_bounds(-3, -3 + length - 1).unwrap();
    for i in 0..length as usize {
        *left.change_at(i).unwrap() = v[2] + i as f64 * v[3];
        *right.change_at(i).unwrap() = v[4] - i as f64 * v[3];
    }
    arithmetic(&left, &right, v[5]);
    matrix_products(&left, v);
    let mut moved = Vector::move_construct(&mut left);
    vector_values(&moved);
    vector_values(&left);
    moved.resize(length + 40).unwrap();
    moved.resize(length).unwrap();
    vector_values(&moved);
    print!(
        "{} ",
        Vector::with_size(0).norm().unwrap_err().exception_type()
    );
    println!();
}
fn arithmetic(left: &Vector<'_>, right: &Vector<'_>, divisor: f64) {
    print!(
        "{} {} {} {} {} ",
        left.norm().unwrap().to_bits(),
        left.norm2().unwrap().to_bits(),
        left.max_index(),
        left.min_index(),
        left.dot(right).unwrap().to_bits()
    );
    checked_vector(left.added(right));
    checked_vector(left.subtracted(right));
    vector_values(&left.multiplied(divisor));
    checked_vector(left.divided(divisor));
    let mut divided = left.clone();
    match divided.divide(divisor) {
        Ok(()) => vector_values(&divided),
        Err(error) => print!("{} ", error.exception_type()),
    }
    checked_vector(left.normalized());
    checked_vector(left.inverse());
    checked_vector(left.slice(left.upper(), left.lower()));
}
fn matrix_products(left: &Vector<'_>, v: &[f64]) {
    let length = left.length();
    let mut matrix = Matrix::with_bounds(2, 4, -2, -2 + length - 1).unwrap();
    for row in 0..3 {
        for col in 0..length {
            matrix[(row + 2, col - 2)] = v[4] + f64::from(row) * v[2] - f64::from(col) * v[3];
        }
    }
    let mut row = Vector::with_bounds(7, 9).unwrap();
    row[7] = v[2];
    row[8] = v[3];
    row[9] = v[4];
    checked_vector(row.multiplied_matrix(&matrix));
    let mut output = Vector::with_bounds(3, 5).unwrap();
    output.set_matrix_vector(&matrix, left).unwrap();
    vector_values(&output);
    output.set_vector_transpose_matrix(left, &matrix).unwrap();
    vector_values(&output);
    let mut transposed = Vector::with_bounds(1, length).unwrap();
    transposed
        .set_transpose_matrix_vector(&matrix, &row)
        .unwrap();
    vector_values(&transposed);
}
