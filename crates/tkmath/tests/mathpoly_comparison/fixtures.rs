const THE_TOLERANCE: f64 = 1e-9;
fn near(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}");
}
fn old_roots(solver: &DirectPolynomialRoots) -> Vec<f64> {
    let mut roots: Vec<f64> = Vec::new();
    if solver.is_done() {
        for i in 1..=solver.solutions().unwrap() {
            roots.push(solver.value(i).unwrap());
        }
    }
    roots.sort_by(|a, b| a.partial_cmp(b).unwrap());
    roots
}
fn new_roots(result: &PolyResult) -> Vec<f64> {
    let mut roots: Vec<f64> = Vec::new();
    if result.is_done() {
        for i in 0..result.nb_roots {
            roots.extend(std::iter::repeat_n(
                result.roots[i],
                result.multiplicities[i],
            ));
        }
    }
    roots.sort_by(|a, b| a.partial_cmp(b).unwrap());
    roots
}
fn compare_roots(old: Vec<f64>, new: Vec<f64>, tol: f64) {
    assert_eq!(old.len(), new.len());
    for i in 0..old.len() {
        near(old[i], new[i], tol);
    }
}
