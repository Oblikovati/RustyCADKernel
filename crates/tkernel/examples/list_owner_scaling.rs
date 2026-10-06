//! Diagnostic timings only: no wall-clock threshold is a correctness assertion.
use rustycad_tkernel::ncollection::List;
use std::time::Instant;
fn run_operation(list: &mut List<usize>, operation: &str, size: usize) {
    match operation {
        "clear" => list.clear(),
        "remove_first" => {
            while !list.is_empty() {
                list.remove_first().unwrap();
            }
        }
        "retain_all" => list.retain(|_| true),
        "search_miss" => assert!(!list.remove_value(&size)),
        _ => unreachable!(),
    }
}
fn measure(size: usize, operation: &str) -> u128 {
    let mut list: List<usize> = (0..size).collect();
    let start = Instant::now();
    run_operation(&mut list, operation, size);
    let elapsed = start.elapsed().as_nanos();
    assert_eq!(
        list.len(),
        if ["clear", "remove_first"].contains(&operation) {
            0
        } else {
            size
        }
    );
    elapsed
}
fn sample(size: usize, operation: &str) {
    let mut samples: Vec<_> = (0..3).map(|_| measure(size, operation)).collect();
    samples.sort_unstable();
    println!(
        "{{\"operation\":\"{operation}\",\"size\":{size},\"median_ns\":{},\"samples_ns\":{samples:?}}}",
        samples[1]
    );
}
fn main() {
    for size in [4_000, 8_000, 16_000, 32_000] {
        for operation in ["clear", "remove_first", "retain_all", "search_miss"] {
            sample(size, operation);
        }
    }
}
