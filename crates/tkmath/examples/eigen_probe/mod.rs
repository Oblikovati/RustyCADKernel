//! Native tridiagonal QL ordering and bounded inputs.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::math::{EigenValuesSearcher, Vector};
pub(super) fn eigen_case(values: &[f64]) {
    let n: i32 = values[0] as i32;
    let lower: i32 = values[1] as i32;
    let mut diagonal: Array1<'static, f64> = Array1::with_bounds(lower, lower + n - 1).unwrap();
    let mut subdiagonal: Array1<'static, f64> = Array1::with_bounds(-lower, n - lower - 1).unwrap();
    for index in 0..n {
        diagonal[lower + index] = values[2 + index as usize];
        subdiagonal[index - lower] = values[2 + (n + index) as usize];
    }
    let solver: EigenValuesSearcher = EigenValuesSearcher::new(&diagonal, &subdiagonal).unwrap();
    print!("{} {} ", i32::from(solver.is_done()), solver.dimension());
    if solver.is_done() {
        print_results(&solver);
    }
    println!();
}
fn print_results(solver: &EigenValuesSearcher) {
    for index in 1..=solver.dimension() {
        print!("{} ", solver.eigen_value(index).unwrap().to_bits());
        let vector: Vector<'static> = solver.eigen_vector(index).unwrap();
        for row in 1..=solver.dimension() {
            print!("{} ", vector[row].to_bits());
        }
    }
}
