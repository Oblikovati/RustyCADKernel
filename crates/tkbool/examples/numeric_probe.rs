//! Exact TKBool numeric oracle counterpart of tools/oracle/tkbool_probe.cxx: reads
//! `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod bl_probe;
use bl_probe::{brep_fill_generator, brep_fill_package};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkbool_probe.cxx (`probe_cases`); each package adds
/// its operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // BRepFill (bl_probe/brep_fill_generator.rs, bl_probe/brep_fill_package.rs)
    ("bf_generator", brep_fill_generator::generator_case),
    ("bf_face", brep_fill_package::face_case),
    ("bf_shell", brep_fill_package::shell_case),
    ("bf_axe", brep_fill_package::axe_case),
    ("bf_search_origin", brep_fill_package::search_origin_case),
    ("bf_compute_acr", brep_fill_package::compute_acr_case),
    ("bf_insert_acr", brep_fill_package::insert_acr_case),
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
