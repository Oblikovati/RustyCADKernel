// SPDX-License-Identifier: GPL-2.0-only
//! Replay accepted source/query pairs for a local exact-sign baseline.
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, signs::signs_at_polynomial_roots};
use std::{cmp::Ordering, time::Instant};
struct SignCase {
    source: Vec<f64>,
    query: Vec<f64>,
    expected: Vec<Ordering>,
}
fn coefficients(text: &str) -> Vec<f64> {
    text.split(',')
        .map(|bits| f64::from_bits(u64::from_str_radix(bits, 16).unwrap()))
        .collect()
}
fn corpus() -> Vec<SignCase> {
    include_str!("../tests/exact_roots/signs/oracle.tsv")
        .lines()
        .map(|row| {
            let fields: Vec<&str> = row.split('\t').collect();
            SignCase {
                source: coefficients(fields[1]),
                query: coefficients(fields[2]),
                expected: fields[3]
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.parse::<i8>().unwrap().cmp(&0))
                    .collect(),
            }
        })
        .collect()
}
fn run(cases: &[SignCase], repetitions: usize) -> usize {
    let mut count = 0;
    for _ in 0..repetitions {
        for case in cases {
            let result = signs_at_polynomial_roots(
                &case.source,
                &case.query,
                [-16., 16.],
                RootIsolationOptions::new(0.0001),
                || false,
            )
            .unwrap();
            assert_eq!(result.signs(), case.expected);
            count += result.signs().len();
            std::hint::black_box(result);
        }
    }
    count
}
fn main() {
    let cases = corpus();
    run(&cases, 1);
    let start = Instant::now();
    let signs = run(&cases, 1000);
    println!(
        "{{\"requests\":{},\"signs\":{signs},\"elapsed_ns\":{}}}",
        cases.len() * 1000,
        start.elapsed().as_nanos()
    );
}
