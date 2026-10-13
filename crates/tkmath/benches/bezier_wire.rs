// SPDX-License-Identifier: GPL-2.0-only
//! Measure complete accepted wire queries, including occurrence-view allocation.
#[path = "../tests/fixtures/bezier_wire.rs"]
mod fixture;
use rustycad_tkmath::predicates::roots::bezier_line::classify_bezier_wire;
use std::time::Instant;
fn run(cases: &[fixture::WireFixture], repeats: usize) -> usize {
    let mut completed = 0;
    for _ in 0..repeats {
        for case in cases {
            complete(case);
            completed += 1;
        }
    }
    completed
}
fn complete(case: &fixture::WireFixture) {
    let result = case.with_query(|query| classify_bezier_wire(query, || false).unwrap());
    assert_eq!(result.location(), &case.expected, "{}", case.name);
    std::hint::black_box(result);
}
fn main() {
    let cases = fixture::corpus();
    run(&cases, 1);
    let start = Instant::now();
    let completed = run(&cases, 1000);
    println!(
        "{{\"requests\":{completed},\"elapsed_ns\":{}}}",
        start.elapsed().as_nanos()
    );
}
