//! Exact TKFillet numeric oracle counterpart of tools/oracle/tkfillet_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod fl_probe;
use fl_probe::{blend_func, brep_blend, chamfer, chfi_ds, fillet, rst_rst, surf_rst};

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
    ("fl_const_rad", fillet::const_rad_case),
    ("fl_const_rad_inv", blend_func::const_rad_inv_case),
    ("fl_evol_rad", fillet::evol_rad_case),
    ("fl_evol_rad_inv", blend_func::evol_rad_inv_case),
    ("fl_chamfer", chamfer::chamfer_case),
    ("fl_chamfer_inv", chamfer::chamfer_inv_case),
    ("fl_rad_inv", brep_blend::rad_inv_case),
    ("fl_blend_tool", brep_blend::blend_tool_case),
    ("fl_surf_rst", surf_rst::surf_rst_case),
    ("fl_rst_rst", rst_rst::rst_rst_case),
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
