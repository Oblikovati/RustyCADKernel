//! Exact TKGeomBase numeric oracle counterpart of tools/oracle/tkgeombase_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod gb_probe;
use gb_probe::gce;

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkgeombase_probe.cxx (`probe_cases`); each
/// package adds its operations, in the same order as the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // gce (gb_probe/gce.rs)
    ("gce_circ", gce::circ_case),
    ("gce_circ2d", gce::circ2d_case),
    ("gce_cone", gce::cone_case),
    ("gce_cylinder", gce::cylinder_case),
    ("gce_dir", gce::dir_case),
    ("gce_dir2d", gce::dir2d_case),
    ("gce_elips", gce::elips_case),
    ("gce_elips2d", gce::elips2d_case),
    ("gce_hypr", gce::hypr_case),
    ("gce_hypr2d", gce::hypr2d_case),
    ("gce_lin", gce::lin_case),
    ("gce_lin2d", gce::lin2d_case),
    ("gce_parab", gce::parab_case),
    ("gce_parab2d", gce::parab2d_case),
    ("gce_pln", gce::pln_case),
    ("gce_mirror", gce::mirror_case),
    ("gce_mirror2d", gce::mirror2d_case),
    ("gce_rotation", gce::rotation_case),
    ("gce_rotation2d", gce::rotation2d_case),
    ("gce_scale", gce::scale_case),
    ("gce_scale2d", gce::scale2d_case),
    ("gce_translation", gce::translation_case),
    ("gce_translation2d", gce::translation2d_case),
    ("gce_status", gce::status_case),
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
