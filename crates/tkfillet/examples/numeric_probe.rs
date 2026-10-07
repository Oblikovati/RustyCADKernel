//! Exact TKFillet numeric oracle counterpart of tools/oracle/tkfillet_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod fl_probe;
use fl_probe::{blend_func, chfi_ds};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkfillet_probe.cxx (`probe_cases`); each
/// package adds its operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // ChFiDS (fl_probe/chfi_ds.rs)
    ("fl_spine", chfi_ds::spine_case),
    ("fl_arc_spine", chfi_ds::arc_spine_case),
    ("fl_fil_law", chfi_ds::fil_law_case),
    // BlendFunc (fl_probe/blend_func.rs)
    ("fl_const_rad", blend_func::const_rad_case),
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
