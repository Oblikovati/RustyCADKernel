//! Stateful SVD solves and pseudoinverses compared with native OCCT.
use rustycad_tkmath::math::{Matrix, Svd, Vector};
pub(super) fn svd_case(values: &[f64]) {
    let rows: i32 = values[0] as i32;
    let cols: i32 = values[1] as i32;
    let rl: i32 = values[2] as i32;
    let cl: i32 = values[3] as i32;
    let mut matrix: Matrix<'static> =
        Matrix::with_bounds(rl, rl + rows - 1, cl, cl + cols - 1).unwrap();
    for row in 0..rows {
        for col in 0..cols {
            matrix[(rl + row, cl + col)] = values[6 + (row * cols + col) as usize];
        }
    }
    let mut rhs: Vector<'static> = Vector::with_bounds(-2, rows - 3).unwrap();
    for row in 0..rows {
        rhs[row - 2] = values[6 + (rows * cols + row) as usize];
    }
    let mut solver: Svd = Svd::new(&matrix).unwrap();
    print!("{} ", i32::from(solver.is_done()));
    if solver.is_done() {
        outputs(&mut solver, &rhs, rows, cols, values[4]);
        outputs(&mut solver, &rhs, rows, cols, values[5]);
    }
    println!();
}
fn outputs(solver: &mut Svd, rhs: &Vector<'_>, rows: i32, cols: i32, epsilon: f64) {
    let mut output: Vector<'static> = Vector::filled(5, 4 + cols, 91.0).unwrap();
    solver.solve(rhs, &mut output, epsilon).unwrap();
    for index in 5..=4 + cols {
        print!("{} ", output[index].to_bits());
    }
    let mut inverse: Matrix<'static> = Matrix::with_bounds(1, cols, 1, rows).unwrap();
    solver.pseudo_inverse(&mut inverse, epsilon).unwrap();
    for row in 1..=cols {
        for col in 1..=rows {
            print!("{} ", inverse[(row, col)].to_bits());
        }
    }
}
