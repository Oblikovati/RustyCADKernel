// Copyright (c) 2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! `Utils::ConsolidateRealRoots` / `ConsolidateComplexRoots`: merge root clusters closer
//! than `tolerance^(1/degree)` into one root with verified algebraic multiplicity.
use super::multiplicity::{has_complex_multiplicity, has_multiplicity};
use crate::math_poly::extended::{Extended as E, scale_double};
use crate::math_poly::{Complex, PolyResult};
/// Scaled-polynomial context: coefficients (`c.len() - 1` is the degree) and variable exponent.
pub(super) struct ScaledPolynomial<'a> {
    pub(super) coefficients: &'a [f64],
    pub(super) exponent: i32,
}
impl ScaledPolynomial<'_> {
    fn resolution(&self, tolerance: f64) -> f64 {
        tolerance.powf(1.0 / (self.coefficients.len() - 1) as f64)
    }
}
/// End of the cluster starting at `source`, where `near(k)` compares roots `k-1` and `k`.
fn cluster_end(source: usize, count: usize, near: impl Fn(usize) -> bool) -> usize {
    let mut end: usize = source + 1;
    while end < count && near(end) {
        end += 1;
    }
    end
}
/// Mean of a cluster, accumulated in long double as native code does.
fn real_mean(roots: &[f64]) -> f64 {
    let sum: E = roots.iter().fold(E::ZERO, |s, &x| s + E::new(x));
    (sum / E::from_usize(roots.len())).narrow()
}
/// Largest verified multiplicity of a real cluster, with its representative root.
fn real_cluster(p: &ScaledPolynomial, cluster: &[f64]) -> (f64, usize) {
    let mut multiplicity: usize = cluster.len();
    while multiplicity > 1 {
        let mut scaled: f64 = scale_double(real_mean(&cluster[..multiplicity]), -p.exponent);
        if has_multiplicity(p.coefficients, &mut scaled, multiplicity) {
            return (scale_double(scaled, p.exponent), multiplicity);
        }
        multiplicity -= 1;
    }
    (cluster[0], 1)
}
/// Sorts and merges `result.roots[..nb_roots]` in place.
pub(super) fn consolidate_real_roots(
    result: &mut PolyResult,
    tolerance: f64,
    p: &ScaledPolynomial,
) {
    let count: usize = result.nb_roots;
    // std::sort on <= 16 values is a stable insertion sort; roots here are finite.
    result.roots[..count].sort_by(|a, b| a.partial_cmp(b).unwrap());
    let merge: f64 = p.resolution(tolerance);
    let (mut source, mut written): (usize, usize) = (0, 0);
    while source < count {
        let roots: [f64; crate::math_poly::MAX_POLY_DEGREE] = result.roots;
        let end: usize = cluster_end(source, count, |k| {
            (scale_double(roots[k], -p.exponent) - scale_double(roots[k - 1], -p.exponent)).abs()
                <= merge
        });
        let (root, multiplicity) = real_cluster(p, &roots[source..end]);
        result.roots[written] = root;
        result.multiplicities[written] = multiplicity;
        written += 1;
        source += multiplicity;
    }
    result.nb_roots = written;
}
fn complex_mean(roots: &[Complex]) -> Complex {
    let n: E = E::from_usize(roots.len());
    let re: E = roots.iter().fold(E::ZERO, |s, z| s + E::new(z.real()));
    let im: E = roots.iter().fold(E::ZERO, |s, z| s + E::new(z.imag()));
    Complex::new((re / n).narrow(), (im / n).narrow())
}
fn complex_cluster(p: &ScaledPolynomial, cluster: &[Complex]) -> (Complex, usize) {
    let mut multiplicity: usize = cluster.len();
    while multiplicity > 1 {
        let mut scaled: Complex =
            complex_mean(&cluster[..multiplicity]) * scale_double(1.0, -p.exponent);
        if has_complex_multiplicity(p.coefficients, &mut scaled, multiplicity) {
            return (scaled * scale_double(1.0, p.exponent), multiplicity);
        }
        multiplicity -= 1;
    }
    (cluster[0], 1)
}
/// Upper-half-plane roots in (real, imaginary) lexicographic order.
fn positive_roots(result: &PolyResult) -> Vec<Complex> {
    let mut roots: Vec<Complex> = result.complex_roots[..result.nb_complex_roots]
        .iter()
        .copied()
        .filter(|z| z.imag() > 0.0)
        .collect();
    roots.sort_by(|l, r| {
        l.real()
            .partial_cmp(&r.real())
            .unwrap()
            .then(l.imag().partial_cmp(&r.imag()).unwrap())
    });
    roots
}
fn complex_near(roots: &[Complex], k: usize, resolution: f64) -> bool {
    let bound: f64 = [roots[k].abs(), roots[k - 1].abs()]
        .into_iter()
        .fold(1.0_f64, |m, x| if m < x { x } else { m });
    (roots[k] - roots[k - 1]).abs() <= resolution * bound
}
/// Rebuilds the conjugate-pair list with merged clusters.
pub(super) fn consolidate_complex_roots(
    result: &mut PolyResult,
    tolerance: f64,
    p: &ScaledPolynomial,
) {
    let positive: Vec<Complex> = positive_roots(result);
    let resolution: f64 = p.resolution(tolerance);
    let (mut source, mut written): (usize, usize) = (0, 0);
    while source < positive.len() {
        let end: usize = cluster_end(source, positive.len(), |k| {
            complex_near(&positive, k, resolution)
        });
        let (root, multiplicity) = complex_cluster(p, &positive[source..end]);
        for value in [root, root.conj()] {
            result.complex_roots[written] = value;
            result.complex_multiplicities[written] = multiplicity;
            written += 1;
        }
        source += multiplicity;
    }
    result.nb_complex_roots = written;
}
