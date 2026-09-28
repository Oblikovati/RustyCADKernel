//! Solver Dump text, math_Status bookkeeping and convergence-hook oracle counterpart.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{
    DirectPolynomialRoots, GaussPointsAndWeights, KronrodPointsAndWeights, PsoParticle,
    PsoParticlesPool, TrigonometricRoots, ValueAndWeight, Vector,
};
mod minimizers;
mod scalar;
mod sets;
pub(crate) use minimizers::minimizers_case;
pub(crate) use scalar::scalar_case;
pub(crate) use sets::sets_case;
pub(super) fn reals(vector: &Vector<'_>) {
    print!("{} {} {} ", vector.lower(), vector.upper(), vector.length());
    for index in vector.lower()..=vector.upper() {
        print!("{} ", vector[index].to_bits());
    }
}
pub(super) fn flat(text: &str) {
    print!("{} ", text.replace('\n', "|"));
}
pub(super) fn dumped(write: impl FnOnce(&mut String) -> std::fmt::Result) {
    let mut text = String::new();
    write(&mut text).unwrap();
    flat(&text);
}
pub(super) fn guarded(body: impl FnOnce() -> Result<(), Failure>) {
    if let Err(error) = body() {
        print!("{} ", error.exception_type());
    }
}
pub(super) fn vector_from(values: &[f64]) -> Vector<'static> {
    let mut vector = Vector::with_bounds(1, values.len() as i32).unwrap();
    for (offset, value) in values.iter().enumerate() {
        *vector.change_at(offset).unwrap() = *value;
    }
    vector
}
pub(super) fn polynomial_dump_case(v: &[f64]) {
    let [a, b, c, d, e, lo, hi] = [v[0], v[1], v[2], v[3], v[4], v[5], v[6]];
    dumped(|text| DirectPolynomialRoots::quartic(a, b, c, d, e).dump(text));
    dumped(|text| DirectPolynomialRoots::cubic(b, c, d, e).dump(text));
    dumped(|text| DirectPolynomialRoots::quadratic(c, d, e).dump(text));
    dumped(|text| DirectPolynomialRoots::linear(d, e).dump(text));
    flat(&DirectPolynomialRoots::quadratic(c, d, e).to_string());
    dumped(|text| TrigonometricRoots::new(a, b, c, d, e, lo, hi).dump(text));
    dumped(|text| TrigonometricRoots::sine(d, e, lo, hi).dump(text));
    println!();
}
pub(super) fn small_types_case(v: &[f64]) {
    let n = v[0] as usize;
    let mut nodes: Vec<ValueAndWeight> = (0..n)
        .map(|i| ValueAndWeight::new(v[1 + 2 * i], v[2 + 2 * i]))
        .collect();
    print!(
        "{} {} ",
        i32::from(nodes[0].less(&nodes[n - 1])),
        i32::from(nodes[n - 1].less(&nodes[0]))
    );
    ValueAndWeight::sort(&mut nodes);
    for node in &nodes {
        print!("{} {} ", node.value().to_bits(), node.weight().to_bits());
    }
    let zero = ValueAndWeight::default();
    print!("{} {} ", zero.value().to_bits(), zero.weight().to_bits());
    let p = PsoParticle::default();
    let q = PsoParticle {
        distance: -1.0,
        ..Default::default()
    };
    print!(
        "{} {} {} {} ",
        p.distance.to_bits(),
        p.best_distance.to_bits(),
        i32::from(q.less(&p)),
        i32::from(p.less(&q))
    );
    guarded(|| PsoParticlesPool::new(0, 2).map(|_| print!("empty ")));
    guarded(|| PsoParticlesPool::new(2, 0).map(|_| print!("flat ")));
    for order in 0..=3 {
        let gauss = GaussPointsAndWeights::new(order).unwrap();
        print!(
            "{} {} ",
            i32::from(gauss.is_done()),
            gauss.points().length()
        );
        if order == 0 {
            continue;
        }
        let kronrod = KronrodPointsAndWeights::new(order).unwrap();
        print!(
            "{} {} ",
            i32::from(kronrod.is_done()),
            kronrod.points().length()
        );
        if kronrod.is_done() {
            reals(&kronrod.points());
        }
    }
    println!();
}
