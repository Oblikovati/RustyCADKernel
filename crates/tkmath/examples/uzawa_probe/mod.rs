use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{Matrix, Uzawa, Vector};
fn vector_result(result: Result<&Vector<'_>, Failure>, count: i32) {
    match result {
        Ok(value) => {
            print!("{} {} ", value.lower(), value.upper());
            for i in 1..=count {
                print!("{} ", value[i].to_bits());
            }
        }
        Err(e) => print!("{} ", e.exception_type()),
    }
}
fn results(solver: &Uzawa, rows: i32, columns: i32, inequalities: i32) {
    print!("{} ", i32::from(solver.is_done()));
    match solver.iterations() {
        Ok(n) => print!("{n} "),
        Err(e) => print!("{} ", e.exception_type()),
    }
    vector_result(solver.value(), columns);
    vector_result(solver.error(), columns);
    vector_result(solver.initial_error(), rows);
    match solver.inverse_constraints() {
        Ok(inverse) => {
            print!("{} {} ", inverse.row_number(), inverse.col_number());
            if inequalities == 0 {
                for i in 1..=rows {
                    for j in 1..=i {
                        print!("{} ", inverse[(i, j)].to_bits());
                    }
                }
            }
        }
        Err(e) => print!("{} ", e.exception_type()),
    }
    let mut dual: Vector<'static> = Vector::filled(-3, rows - 4, 71.0).unwrap();
    solver.copy_dual(&mut dual).unwrap();
    for i in dual.lower()..=dual.upper() {
        print!("{} ", dual[i].to_bits());
    }
}
pub(super) fn uzawa_case(values: &[f64]) {
    if let Err(error) = run(values) {
        print!("EXCEPTION {} ", error.exception_type());
    }
    println!();
}
fn run(values: &[f64]) -> Result<(), Failure> {
    let (rows, columns): (i32, i32) = (values[0] as i32, values[1] as i32);
    let counts: [i32; 2] = if values[7] == 0.0 {
        [rows, 0]
    } else {
        [values[2] as i32, values[3] as i32]
    };
    let tolerances: [f64; 2] = [values[4], values[5]];
    let limit: i32 = values[6] as i32;
    let mut matrix: Matrix<'static> = Matrix::with_bounds(1, rows, 1, columns).unwrap();
    let mut offset: usize = 8;
    for i in 1..=rows {
        for j in 1..=columns {
            matrix[(i, j)] = values[offset];
            offset += 1;
        }
    }
    let mut rhs: Vector<'static> = Vector::with_bounds(1, rows)?;
    let mut start: Vector<'static> = Vector::with_bounds(1, columns)?;
    for i in 1..=rows {
        rhs[i] = values[offset];
        offset += 1;
    }
    for i in 1..=columns {
        start[i] = values[offset];
        offset += 1;
    }
    let mut solver: Uzawa = if values[7] == 0.0 {
        Uzawa::with_options(&matrix, &rhs, &start, tolerances[0], tolerances[1], limit)?
    } else {
        Uzawa::with_constraints(
            &matrix, &rhs, &start, counts[0], counts[1], tolerances, limit,
        )?
    };
    results(&solver, rows, columns, counts[1]);
    start.multiply(-1.0);
    solver.perform(&matrix, &rhs, &start, counts, tolerances, limit)?;
    results(&solver, rows, columns, counts[1]);
    Ok(())
}
