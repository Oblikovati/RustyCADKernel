//! Exact TKExpress oracle counterpart of tools/oracle/tkexpress_probe.cxx: reads
//! `operation arguments...` lines and prints the same token stream (texts as `x` and
//! hexadecimal bytes, doubles as bit patterns, exceptions as `!` and their type).
// A guarded step ends with `Ok(line.text(...))`: it prints, then succeeds, as each
// native `ex_try` step does.
#![allow(clippy::unit_arg)]
use std::io::{self, BufRead};
mod ex_probe;
use ex_probe::{cases, fold, rel, text};

/// One probe operation over the fields of a request.
type ProbeCase = fn(&[&str]);

/// The dispatch table of tools/oracle/tkexpress_probe.cxx (`probe_cases`).
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    ("ex_text", text::text_case),
    ("ex_fold", fold::fold_case),
    ("ex_fold2", fold::fold2_case),
    ("ex_rel", rel::rel_case),
    ("ex_case", cases::case_case),
];

fn main() {
    for line in io::stdin().lock().lines() {
        let line: String = line.expect("probe input line");
        let mut fields = line.split_whitespace();
        let Some(op) = fields.next() else { continue };
        let arguments: Vec<&str> = fields.collect();
        let Some((_, case)) = PROBE_CASES.iter().find(|(name, _)| *name == op) else {
            eprintln!("Unknown probe operation: {op}");
            std::process::exit(2);
        };
        case(&arguments);
    }
}
