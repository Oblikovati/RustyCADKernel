//! Exact TKV3d numeric oracle counterpart of tools/oracle/tkv3d_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod v3_probe;
use v3_probe::prs3d;

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkv3d_probe.cxx (`probe_cases`); each package adds its
/// operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // Prs3d tools, arrows, deflection and boxes (v3_probe/prs3d.rs)
    ("v3_quadric", prs3d::quadric_case),
    ("v3_arrow", prs3d::arrow_case),
    ("v3_prs3d", prs3d::prs3d_case),
];

fn main() {
    for line in io::stdin().lock().lines() {
        let line: String = line.expect("probe input line");
        let mut fields = line.split_whitespace();
        let Some(op) = fields.next() else { continue };
        let values: Vec<f64> = fields
            .map(|field| field.parse().expect("numeric probe field"))
            .collect();
        let Some((_, case)) = PROBE_CASES.iter().find(|(name, _)| *name == op) else {
            eprintln!("Unknown probe operation: {op}");
            std::process::exit(2);
        };
        case(&values);
    }
}
