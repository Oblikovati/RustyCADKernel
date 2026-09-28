//! Generated quadrature nodes and weights checked bit for bit.
use rustycad_tkmath::math::{KronrodPointsAndWeights, Vector};
pub(super) fn kronrod_case(values: &[f64]) {
    let solver: KronrodPointsAndWeights = KronrodPointsAndWeights::new(values[0] as i32).unwrap();
    print!("{} ", i32::from(solver.is_done()));
    if solver.is_done() {
        let points: Vector<'static> = solver.points();
        let weights: Vector<'static> = solver.weights();
        for index in 1..=points.length() {
            print!("{} {} ", points[index].to_bits(), weights[index].to_bits());
        }
    }
    println!();
}
