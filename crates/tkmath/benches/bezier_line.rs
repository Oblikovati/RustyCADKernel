// SPDX-License-Identifier: GPL-2.0-only
//! Replay accepted exact curve/line inputs to measure a repeatable local baseline.
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, bezier_line::*};
use std::time::Instant;
struct ContactCase {
    controls: Vec<[f64; 2]>,
    weights: Option<Vec<f64>>,
    line: [[f64; 2]; 2],
    expected: usize,
}
fn values(text: &str) -> Vec<f64> {
    text.split(',')
        .map(|bits| f64::from_bits(u64::from_str_radix(bits, 16).unwrap()))
        .collect()
}
fn corpus() -> Vec<ContactCase> {
    include_str!("../tests/exact_roots/bezier_line/oracle.tsv")
        .lines()
        .map(|row| {
            let fields: Vec<&str> = row.split('\t').collect();
            let endpoints = values(fields[3]);
            ContactCase {
                controls: fields[1]
                    .split(';')
                    .map(|p| values(p).try_into().unwrap())
                    .collect(),
                weights: (fields[2] != "-").then(|| values(fields[2])),
                line: [[endpoints[0], endpoints[1]], [endpoints[2], endpoints[3]]],
                expected: fields[4].split('|').filter(|root| !root.is_empty()).count(),
            }
        })
        .collect()
}
fn query(case: &ContactCase) -> usize {
    let request = BezierLineQuery {
        controls: &case.controls,
        weights: case.weights.as_deref(),
        line: case.line,
        interval: [0., 1.],
        options: RootIsolationOptions::new(0.0001),
    };
    let result = intersect_bezier_line(request, || false).unwrap();
    let BezierLineContact::Isolated(roots) = result else {
        panic!("expected isolated contacts");
    };
    assert_eq!(roots.roots().len(), case.expected);
    std::hint::black_box(roots).roots().len()
}
fn run(cases: &[ContactCase], repetitions: usize) -> usize {
    let mut count = 0;
    for _ in 0..repetitions {
        for case in cases {
            count += query(case);
        }
    }
    count
}
fn main() {
    let cases = corpus();
    run(&cases, 1);
    let start = Instant::now();
    let contacts = run(&cases, 1000);
    println!(
        "{{\"requests\":{},\"contacts\":{contacts},\"elapsed_ns\":{}}}",
        cases.len() * 1000,
        start.elapsed().as_nanos()
    );
}
