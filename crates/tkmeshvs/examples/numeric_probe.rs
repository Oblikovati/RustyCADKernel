//! Exact TKMeshVS numeric oracle counterpart of tools/oracle/tkmeshvs_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod mv_probe;

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkmeshvs_probe.cxx (`probe_cases`).
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // Data sources (mv_probe/source.rs)
    ("mv_ds", mv_probe::source::ds_case),
    ("mv_topology", mv_probe::source::topology_case),
    ("mv_deformed", mv_probe::source::deformed_case),
    // Presentation builders and the mesh (mv_probe/build.rs)
    ("mv_build", mv_probe::build::build_case),
    ("mv_compute", mv_probe::build::compute_case),
    ("mv_mesh_api", mv_probe::build::mesh_api_case),
    // Selection, owners and sensitive entities (mv_probe/select.rs)
    ("mv_select", mv_probe::select::select_case),
    ("mv_highlight", mv_probe::select::highlight_case),
    ("mv_sensitive", mv_probe::select::sensitive_case),
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
