//! Exact native counterpart of tools/oracle/tkbo_foundations_probe.cxx.
use std::io::{self, BufRead};
mod bo_probe;
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.expect("expected a readable probe input line");
        let mut fields = line.split_whitespace();
        let Some(operation) = fields.next() else {
            continue;
        };
        let values: Vec<f64> = fields
            .map(|s| s.parse().expect("expected numeric probe field"))
            .collect();
        bo_probe::dispatch(operation, &values);
    }
}
