//! Exact TKFeat numeric oracle counterpart of tools/oracle/tkfeat_probe.cxx: reads
//! `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod ft_probe;
use ft_probe::{forms, locope, sweeps};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkfeat_probe.cxx (`probe_cases`).
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // LocOpe (ft_probe/locope.rs)
    ("ft_lo_sweep", locope::sweep_case),
    ("ft_lo_csinter", locope::csinter_case),
    ("ft_lo_find_edges", locope::find_edges_case),
    ("ft_lo_spliter", locope::spliter_case),
    ("ft_lo_gluer", locope::gluer_case),
    // BRepFeat sweeps (ft_probe/sweeps.rs)
    ("ft_prism", sweeps::prism_case),
    ("ft_dprism", sweeps::dprism_case),
    ("ft_revol", sweeps::revol_case),
    ("ft_pipe", sweeps::pipe_case),
    // BRepFeat forms, holes, gluing and splitting (ft_probe/forms.rs)
    ("ft_hole", forms::hole_case),
    ("ft_lform", forms::lform_case),
    ("ft_rform", forms::rform_case),
    ("ft_gluer", forms::gluer_case),
    ("ft_split", forms::split_case),
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
