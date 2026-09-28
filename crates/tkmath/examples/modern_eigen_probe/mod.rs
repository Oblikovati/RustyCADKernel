//! Differential coverage for modern tridiagonal and Jacobi eigenanalysis.
use rustycad_tkmath::math::{Matrix, Vector};
use rustycad_tkmath::math_lin::*;
pub(super) fn fields(v: &Option<Vector<'_>>, m: &Option<Matrix<'_>>) {
    print!("{} ", i32::from(v.is_some()));
    if let Some(v) = v {
        for i in 0..v.length() as usize {
            print!("{} ", v.at(i).unwrap().to_bits());
        }
    }
    print!("{} ", i32::from(m.is_some()));
    if let Some(m) = m {
        for i in 0..m.row_number() as usize {
            for j in 0..m.col_number() as usize {
                print!("{} ", m.at(i, j).unwrap().to_bits());
            }
        }
    }
}
pub(super) fn modern_eigen_case(v: &[f64]) {
    let n: usize = v[0] as usize;
    let lower: i32 = v[1] as i32;
    let mode: i32 = v[3] as i32;
    let mut d: Vector<'_> = Vector::with_bounds(lower, lower + n as i32 - 1).unwrap();
    let mut e: Vector<'_> = Vector::with_bounds(lower, lower + n as i32 - 1).unwrap();
    for i in 0..n {
        *d.change_at(i).unwrap() = v[4 + i];
        *e.change_at(i).unwrap() = v[4 + n + i];
    }
    if mode == 1 {
        *d.change_at(0).unwrap() = f64::INFINITY;
    }
    let mut a: Matrix<'_> = Matrix::filled(
        lower,
        lower + n as i32 - 1,
        lower + 2,
        lower + n as i32 + 1,
        0.0,
    )
    .unwrap();
    for i in 0..n {
        *a.change_at(i, i).unwrap() = *d.at(i).unwrap();
        if i + 1 < n {
            *a.change_at(i, i + 1).unwrap() = *e.at(i + 1).unwrap();
            *a.change_at(i + 1, i).unwrap() = *e.at(i + 1).unwrap();
        }
    }
    if mode == 2 && n > 1 {
        *a.change_at(1, 0).unwrap() += 1.0;
    }
    let r = eigen_tridiagonal(&d, &e, v[2] as u32);
    print!("{} {} ", r.status as i32, r.dimension);
    fields(&r.eigen_values, &r.eigen_vectors);
    for sort in [false, true] {
        let j = jacobi(&a, sort);
        print!("{} {} ", j.status as i32, j.iterations);
        fields(&j.eigen_values, &j.eigen_vectors);
    }
    for power in [0.5, -0.5, 2.0, -3.0] {
        fields(&None, &matrix_power(&a, power));
    }
    println!();
}

pub(super) fn optional(value: Option<f64>) {
    print!(
        "{} {} ",
        i32::from(value.is_some()),
        value.unwrap_or(91.0).to_bits()
    );
}
pub(super) fn modern_lu_case(v: &[f64]) {
    let n: usize = v[0] as usize;
    let lower: i32 = v[1] as i32;
    let mode: i32 = v[2] as i32;
    let tol: f64 = v[3];
    let mut a: Matrix<'_> =
        Matrix::with_bounds(lower, lower + n as i32 - 1, lower + 2, lower + n as i32 + 1).unwrap();
    let mut b: Vector<'_> = Vector::with_bounds(lower, lower + n as i32 - 1).unwrap();
    for i in 0..n {
        for j in 0..n {
            *a.change_at(i, j).unwrap() = v[4 + i * n + j];
        }
        *b.change_at(i).unwrap() = v[4 + n * n + i];
    }
    if mode == 1 {
        *a.change_at(0, 0).unwrap() = f64::INFINITY;
    }
    if mode == 2 {
        *b.change_at(0).unwrap() = f64::NAN;
    }
    let r = lu(&a, tol);
    print!(
        "{} {} {} {} {} {} {} ",
        r.status as i32,
        r.sign,
        r.row_lower,
        r.col_lower,
        r.dimension,
        r.min_abs_pivot.to_bits(),
        r.max_abs_pivot.to_bits()
    );
    optional(r.determinant);
    optional(r.condition_estimate);
    fields(&None, &r.lu);
    print!("{} ", i32::from(r.pivot.is_some()));
    if let Some(p) = r.pivot {
        for i in p.as_slice() {
            print!("{} ", i);
        }
    }
    let s = solve(&a, &b, tol);
    print!("{} ", s.status as i32);
    optional(s.determinant);
    fields(&s.solution, &None);
    let inv = invert(&a, tol);
    print!("{} ", inv.status as i32);
    optional(inv.determinant);
    fields(&None, &inv.inverse);
    let det = determinant(&a, tol);
    print!("{} ", det.status as i32);
    optional(det.determinant);
    let mut rhs: Matrix<'_> = Matrix::with_bounds(-3, n as i32 - 4, 2, 3).unwrap();
    for i in 0..n {
        *rhs.change_at(i, 0).unwrap() = *b.at(i).unwrap();
        *rhs.change_at(i, 1).unwrap() = i as f64;
    }
    let multiple = solve_multiple(&a, &rhs, tol);
    print!("{} ", multiple.status as i32);
    optional(multiple.determinant);
    fields(&None, &multiple.solutions);
    println!();
}
