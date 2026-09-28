const THE_TOLERANCE: f64 = 1e-10;
fn near(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}");
}
fn CreateIdentity(n: i32) -> Matrix<'static> {
    let mut a: Matrix<'static> = Matrix::filled(1, n, 1, n, 0.0).unwrap();
    for i in 1..=n {
        a[(i, i)] = 1.0;
    }
    a
}
fn CreateSPD(n: i32) -> Matrix<'static> {
    let mut a: Matrix<'static> = Matrix::filled(1, n, 1, n, 0.0).unwrap();
    for i in 1..=n {
        for j in 1..=n {
            a[(i, j)] = 1.0 / (i + j - 1) as f64;
        }
        a[(i, i)] += n as f64;
    }
    a
}
fn vectornorm(v: &Vector<'_>) -> f64 {
    let mut sum: f64 = 0.0;
    for i in 0..v.length() as usize {
        sum += v.at(i).unwrap() * v.at(i).unwrap();
    }
    sum.sqrt()
}
fn matmul(a: &Matrix<'_>, b: &Matrix<'_>) -> Matrix<'static> {
    let mut c: Matrix<'static> = Matrix::filled(1, a.row_number(), 1, b.col_number(), 0.0).unwrap();
    for i in 0..a.row_number() as usize {
        for j in 0..b.col_number() as usize {
            *c.change_at(i, j).unwrap() = product_entry(a, b, i, j);
        }
    }
    c
}
fn product_entry(a: &Matrix<'_>, b: &Matrix<'_>, i: usize, j: usize) -> f64 {
    let mut sum: f64 = 0.0;
    for k in 0..a.col_number() as usize {
        sum += a.at(i, k).unwrap() * b.at(k, j).unwrap();
    }
    sum
}
fn transpose(a: &Matrix<'_>) -> Matrix<'static> {
    let mut out: Matrix<'static> =
        Matrix::with_bounds(1, a.col_number(), 1, a.row_number()).unwrap();
    for i in 0..a.row_number() as usize {
        for j in 0..a.col_number() as usize {
            *out.change_at(j, i).unwrap() = *a.at(i, j).unwrap();
        }
    }
    out
}
fn swap_entries(a: &mut Matrix<'_>, i: usize, k: usize, j: usize) {
    let saved: f64 = *a.at(i, j).unwrap();
    *a.change_at(i, j).unwrap() = *a.at(k, j).unwrap();
    *a.change_at(k, j).unwrap() = saved;
}
// glibc rand() sequence from the pinned Linux oracle, seed 42; no runtime C dependency.
fn CreateRandom(m: i32, n: i32) -> Matrix<'static> {
    assert_eq!((m, n), (5, 3));
    let draws: [i32; 15] = [
        71876166, 708592740, 1483128881, 907283241, 442951012, 537146758, 1366999021, 1854614940,
        647800535, 53523743, 783815874, 1643643143, 682599717, 291474504, 229233696,
    ];
    let mut a: Matrix<'static> = Matrix::with_bounds(1, m, 1, n).unwrap();
    for i in 0..m as usize {
        for j in 0..n as usize {
            *a.change_at(i, j).unwrap() =
                draws[i * n as usize + j] as f64 / 2147483647.0 * 2.0 - 1.0;
        }
    }
    a
}
