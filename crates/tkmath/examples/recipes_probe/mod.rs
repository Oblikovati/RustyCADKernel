//! math_Recipes kernel and linear-solver dump oracle counterpart.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::recipes::{
    dactcl_decompose, dactcl_solve, jacobi, lu_decompose, lu_decompose_with_scales, lu_invert,
    lu_solve, svd_decompose, svd_decompose_with_work, svd_solve,
};
use rustycad_tkmath::math::{Gauss, GaussLeastSquares, IntegerVector, Jacobi, Matrix, Svd, Vector};
fn reals(vector: &Vector<'_>) {
    print!("{} {} {} ", vector.lower(), vector.upper(), vector.length());
    for index in vector.lower()..=vector.upper() {
        print!("{} ", vector[index].to_bits());
    }
}
fn integers(vector: &IntegerVector<'_>) {
    print!("{} {} {} ", vector.lower(), vector.upper(), vector.length());
    for index in vector.lower()..=vector.upper() {
        print!("{} ", vector[index]);
    }
}
fn cells(matrix: &Matrix<'_>) {
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
fn read_matrix(v: &[f64], rows: i32, cols: i32, start: usize) -> Matrix<'static> {
    let mut matrix = Matrix::with_bounds(1, rows, 1, cols).unwrap();
    for row in 0..rows as usize {
        for col in 0..cols as usize {
            *matrix.change_at(row, col).unwrap() = v[start + row * cols as usize + col];
        }
    }
    matrix
}
fn read_vector(v: &[f64], length: i32, start: usize) -> Vector<'static> {
    let mut vector = Vector::with_bounds(1, length).unwrap();
    for offset in 0..length as usize {
        *vector.change_at(offset).unwrap() = v[start + offset];
    }
    vector
}
fn dumped(text: String) {
    print!("{} ", text.replace('\n', "|"));
}
pub(super) fn lu_case(v: &[f64]) {
    let (n, tiny) = (v[0] as i32, v[1]);
    let a = read_matrix(v, n, n, 2);
    let b = read_vector(v, n, 2 + (n * n) as usize);
    let mut lu = a.clone();
    let mut indx = IntegerVector::filled(1, n, 0).unwrap();
    let mut d = 91.0;
    let status = lu_decompose(&mut lu, &mut indx, &mut d, tiny).unwrap();
    print!("{} {} ", status, d.to_bits());
    cells(&lu);
    integers(&indx);
    if status == 0 {
        lu_solutions(&lu, &indx, &b);
    }
    let mut scaled = a.clone();
    let mut scales = Vector::filled(1, n, 91.0).unwrap();
    indx.init(0);
    d = 91.0;
    let status = lu_decompose_with_scales(&mut scaled, &mut indx, &mut d, &mut scales, tiny);
    print!("{} {} ", status.unwrap(), d.to_bits());
    cells(&scaled);
    integers(&indx);
    reals(&scales);
    let mut inverse = a.clone();
    print!("{} ", lu_invert(&mut inverse).unwrap());
    cells(&inverse);
    println!();
}
fn lu_solutions(lu: &Matrix<'_>, indx: &IntegerVector<'_>, b: &Vector<'_>) {
    let mut x = b.clone();
    lu_solve(lu, indx, &mut x).unwrap();
    reals(&x);
    let n = b.length();
    let mut shifted = Vector::with_bounds(-2, -3 + n).unwrap();
    for i in 1..=n {
        shifted[i - 3] = b[i];
    }
    lu_solve(lu, indx, &mut shifted).unwrap();
    reals(&shifted);
}
pub(super) fn svd_case(v: &[f64]) {
    let (m, n) = (v[0] as i32, v[1] as i32);
    let a = read_matrix(v, m, n, 2);
    let b = read_vector(v, m, 2 + (m * n) as usize);
    let mut u = a.clone();
    let mut right = Matrix::filled(1, n, 1, n, 91.0).unwrap();
    let mut w = Vector::filled(1, n, 91.0).unwrap();
    print!("{} ", svd_decompose(&mut u, &mut w, &mut right).unwrap());
    cells(&u);
    reals(&w);
    cells(&right);
    let mut x = Vector::filled(1, n, 91.0).unwrap();
    svd_solve(&u, &w, &right, &b, &mut x).unwrap();
    reals(&x);
    let mut u2 = a.clone();
    let mut v2 = Matrix::with_bounds(1, n, 1, n).unwrap();
    let mut w2 = Vector::with_bounds(1, n).unwrap();
    let mut rv1 = Vector::filled(1, n, 91.0).unwrap();
    let status = svd_decompose_with_work(&mut u2, &mut w2, &mut v2, &mut rv1).unwrap();
    print!("{status} ");
    reals(&rv1);
    println!();
}
pub(super) fn jacobi_case(v: &[f64]) {
    let n = v[0] as i32;
    let mut work = read_matrix(v, n, n, 1);
    let mut vectors = Matrix::with_bounds(1, n, 1, n).unwrap();
    let mut values = Vector::with_bounds(1, n).unwrap();
    let mut rotations = 91;
    let status = jacobi(&mut work, &mut values, &mut vectors, &mut rotations).unwrap();
    print!("{status} {rotations} ");
    reals(&values);
    cells(&vectors);
    cells(&work);
    println!();
}
pub(super) fn dactcl_case(v: &[f64]) {
    let (equations, pivot) = (v[0] as i32, v[1]);
    let mut indx = IntegerVector::with_bounds(1, equations).unwrap();
    let mut total = 0;
    for j in 1..=equations {
        total += v[1 + j as usize] as i32;
        indx[j] = total;
    }
    let start = 2 + equations as usize;
    let mut a = read_vector(v, total, start);
    let mut b = read_vector(v, equations, start + total as usize);
    print!("{} ", dactcl_decompose(&mut a, &indx, pivot).unwrap());
    reals(&a);
    print!("{} ", dactcl_solve(&a, &mut b, &indx, pivot).unwrap());
    reals(&b);
    println!();
}
pub(super) fn linear_dump_case(v: &[f64]) {
    let (rows, cols, pivot) = (v[0] as i32, v[1] as i32, v[2]);
    let a = read_matrix(v, rows, cols, 3);
    let guarded = |result: Result<String, Failure>| match result {
        Ok(text) => dumped(text),
        Err(error) => print!("{} ", error.exception_type()),
    };
    guarded(Gauss::with_min_pivot(&a, pivot).map(|g| g.to_string()));
    dumped(dump_text(|text| {
        GaussLeastSquares::with_min_pivot(&a, pivot)
            .unwrap()
            .dump(text)
    }));
    dumped(dump_text(|text| Svd::new(&a).unwrap().dump(text)));
    guarded(Jacobi::new(&a).map(|j| dump_text(|text| j.dump(text))));
    println!();
}
fn dump_text(write: impl FnOnce(&mut String) -> std::fmt::Result) -> String {
    let mut text = String::new();
    write(&mut text).unwrap();
    text
}
