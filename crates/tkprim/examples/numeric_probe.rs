//! Exact TKPrim numeric oracle counterpart of tools/oracle/tkprim_probe.cxx: reads
//! `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod pr_probe;
use pr_probe::{brep_prim, brep_prim_one_axis, shape, sweep};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkprim_probe.cxx (`probe_cases`); each package adds
/// its operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // The shared shape digest (pr_probe/shape.rs)
    ("pr_digest_vertex", shape::digest_vertex_case),
    // Sweep (pr_probe/sweep.rs)
    ("sw_numshape", sweep::numshape_case),
    ("sw_reinit", sweep::reinit_case),
    ("sw_init", sweep::init_case),
    // BRepPrim (pr_probe/brep_prim.rs, pr_probe/brep_prim_one_axis.rs)
    ("bp_wedge", brep_prim::wedge_case),
    ("bp_builder", brep_prim::builder_case),
    ("bp_oneaxis", brep_prim_one_axis::oneaxis_case),
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
