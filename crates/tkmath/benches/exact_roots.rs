// SPDX-License-Identifier: GPL-2.0-only
//! Replay the certified root corpus to measure a repeatable local baseline.
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, isolate_polynomial_roots};
use std::time::Instant;
struct RootCase {
    coefficients: Vec<f64>,
    expected: usize,
}
fn corpus() -> Vec<RootCase> {
    include_str!("../tests/exact_roots/oracle.tsv")
        .lines()
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            RootCase {
                coefficients: fields[1]
                    .split(',')
                    .map(|bits| f64::from_bits(u64::from_str_radix(bits, 16).unwrap()))
                    .collect(),
                expected: fields[2].split('|').filter(|root| !root.is_empty()).count(),
            }
        })
        .collect()
}
fn run(cases: &[RootCase], repetitions: usize) -> usize {
    let options = RootIsolationOptions::new(0.0001);
    let mut roots = 0;
    for _ in 0..repetitions {
        for case in cases {
            let result =
                isolate_polynomial_roots(&case.coefficients, [-16., 16.], options, || false)
                    .unwrap();
            assert_eq!(result.roots().len(), case.expected);
            roots += result.roots().len();
            std::hint::black_box(result);
        }
    }
    roots
}
fn main() {
    let cases = corpus();
    run(&cases, 1);
    let start = Instant::now();
    let roots = run(&cases, 1000);
    println!(
        "{{\"requests\":{},\"roots\":{roots},\"elapsed_ns\":{}}}",
        cases.len() * 1000,
        start.elapsed().as_nanos()
    );
}
