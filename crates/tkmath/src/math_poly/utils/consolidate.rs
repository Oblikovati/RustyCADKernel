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

//! `ConsolidateRealRoots` / `ConsolidateComplexRoots`: merge root clusters closer than
//! `tolerance^(1/degree)` into one root with verified algebraic multiplicity.
use super::{has_complex_multiplicity, has_multiplicity};
use crate::math_poly::extended::{LongDouble, scale_double};
use crate::math_poly::{Complex, MAX_POLY_DEGREE, PolyResult};
use rustycad_tkernel::stl;
/// The undeflated scaled polynomial and its variable exponent.
struct Original<'a> {
    coefficients: &'a [f64],
    degree: i32,
    exponent: i32,
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
    let sum: LongDouble = roots
        .iter()
        .fold(LongDouble::ZERO, |s, &x| s + LongDouble::new(x));
    (sum / LongDouble::from_usize(roots.len())).narrow()
}
/// Largest verified multiplicity of a real cluster, with its representative root.
fn real_cluster(p: &Original, cluster: &[f64]) -> (f64, usize) {
    let mut multiplicity: usize = cluster.len();
    while multiplicity > 1 {
        let mut scaled: f64 = scale_double(real_mean(&cluster[..multiplicity]), -p.exponent);
        if has_multiplicity(p.coefficients, p.degree, &mut scaled, multiplicity) {
            return (scale_double(scaled, p.exponent), multiplicity);
        }
        multiplicity -= 1;
    }
    (cluster[0], 1)
}
/// `ConsolidateRealRoots`: sorts `result.roots[..nb_roots]` with libstdc++'s `std::sort`
/// and merges clusters in place; `original` is the undeflated scaled polynomial;
/// `utils::consolidate_real_roots(&mut result, 1e-12, &c, 3, exponent)`.
pub fn consolidate_real_roots(
    result: &mut PolyResult,
    tolerance: f64,
    original: &[f64],
    degree: i32,
    variable_exponent: i32,
) {
    let p: Original = Original {
        coefficients: original,
        degree,
        exponent: variable_exponent,
    };
    let count: usize = result.nb_roots;
    stl::sort_by(&mut result.roots[..count], |a, b| a < b);
    let merge: f64 = tolerance.powf(1.0 / degree as f64);
    let (mut source, mut written): (usize, usize) = (0, 0);
    while source < count {
        let roots: [f64; MAX_POLY_DEGREE] = result.roots;
        let end: usize = cluster_end(source, count, |k| {
            (scale_double(roots[k], -p.exponent) - scale_double(roots[k - 1], -p.exponent)).abs()
                <= merge
        });
        let (root, multiplicity) = real_cluster(&p, &roots[source..end]);
        result.roots[written] = root;
        result.multiplicities[written] = multiplicity;
        written += 1;
        source += multiplicity;
    }
    result.nb_roots = written;
}
fn complex_mean(roots: &[Complex]) -> Complex {
    let n: LongDouble = LongDouble::from_usize(roots.len());
    let re: LongDouble = roots
        .iter()
        .fold(LongDouble::ZERO, |s, z| s + LongDouble::new(z.real()));
    let im: LongDouble = roots
        .iter()
        .fold(LongDouble::ZERO, |s, z| s + LongDouble::new(z.imag()));
    Complex::new((re / n).narrow(), (im / n).narrow())
}
fn complex_cluster(p: &Original, cluster: &[Complex]) -> (Complex, usize) {
    let mut multiplicity: usize = cluster.len();
    while multiplicity > 1 {
        let mut scaled: Complex =
            complex_mean(&cluster[..multiplicity]) * scale_double(1.0, -p.exponent);
        if has_complex_multiplicity(p.coefficients, p.degree, &mut scaled, multiplicity) {
            return (scaled * scale_double(1.0, p.exponent), multiplicity);
        }
        multiplicity -= 1;
    }
    (cluster[0], 1)
}
/// Upper-half-plane roots in (real, imaginary) lexicographic order, sorted by libstdc++'s
/// `std::sort` into a `THE_MAX_POLY_DEGREE / 2` buffer as native code does.
fn positive_roots(result: &PolyResult) -> ([Complex; MAX_POLY_DEGREE / 2], usize) {
    let mut roots: [Complex; MAX_POLY_DEGREE / 2] = [Complex::default(); MAX_POLY_DEGREE / 2];
    let mut count: usize = 0;
    for &root in &result.complex_roots[..result.nb_complex_roots] {
        if root.imag() > 0.0 {
            roots[count] = root;
            count += 1;
        }
    }
    stl::sort_by(&mut roots[..count], |l, r| {
        l.real() < r.real() || (l.real() == r.real() && l.imag() < r.imag())
    });
    (roots, count)
}
fn complex_near(roots: &[Complex], k: usize, resolution: f64) -> bool {
    let bound: f64 = [roots[k].abs(), roots[k - 1].abs()]
        .into_iter()
        .fold(1.0_f64, |m, x| if m < x { x } else { m });
    (roots[k] - roots[k - 1]).abs() <= resolution * bound
}
/// `ConsolidateComplexRoots`: rebuilds the conjugate-pair list from the upper-half-plane
/// roots with merged clusters; `utils::consolidate_complex_roots(&mut result, 1e-12, &c, 4, 0)`.
pub fn consolidate_complex_roots(
    result: &mut PolyResult,
    tolerance: f64,
    original: &[f64],
    degree: i32,
    variable_exponent: i32,
) {
    let p: Original = Original {
        coefficients: original,
        degree,
        exponent: variable_exponent,
    };
    let (roots, count) = positive_roots(result);
    let positive: &[Complex] = &roots[..count];
    let resolution: f64 = tolerance.powf(1.0 / degree as f64);
    let (mut source, mut written): (usize, usize) = (0, 0);
    while source < positive.len() {
        let end: usize = cluster_end(source, positive.len(), |k| {
            complex_near(positive, k, resolution)
        });
        let (root, multiplicity) = complex_cluster(&p, &positive[source..end]);
        for value in [root, root.conj()] {
            result.complex_roots[written] = value;
            result.complex_multiplicities[written] = multiplicity;
            written += 1;
        }
        source += multiplicity;
    }
    result.nb_complex_roots = written;
}
