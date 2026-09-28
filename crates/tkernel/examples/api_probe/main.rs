//! TKernel API differential probe: the Rust twin of tools/oracle/tkernel_api_probe.cxx.
//! Reads one request per stdin line and prints one response line.
mod ascii_string_api;
mod common;
mod exceptions;
mod extended_string_api;
mod standard_runtime;
mod standard_services;
mod standard_values;

use common::Case;
use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};

/// Merges the per-area tables, rejecting a duplicated operation name.
fn collect_cases() -> BTreeMap<&'static str, Case> {
    let tables: [&[(&str, Case)]; 6] = [
        exceptions::CASES,
        ascii_string_api::CASES,
        extended_string_api::CASES,
        standard_values::CASES,
        standard_runtime::CASES,
        standard_services::CASES,
    ];
    let mut cases = BTreeMap::new();
    for &(name, case) in tables.iter().copied().flatten() {
        assert!(
            cases.insert(name, case).is_none(),
            "duplicate probe operation {name:?}"
        );
    }
    cases
}

fn main() {
    let cases = collect_cases();
    let mut output = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let line = line.expect("request line");
        let mut input = line.split_whitespace();
        let operation = input.next().unwrap_or_default();
        let case = cases
            .get(operation)
            .unwrap_or_else(|| panic!("unknown probe operation {operation:?}"));
        writeln!(output, "{}", case(&mut input)).expect("stdout");
    }
}
