//! Exact TKHelix numeric oracle counterpart of tools/oracle/tkhelix_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod hx_probe;
use hx_probe::{helix_brep, helix_geom, helix_geom_builders, placement};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkhelix_probe.cxx (`probe_cases`); each
/// package adds its operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // HelixGeom_BuilderHelix placement (hx_probe/placement.rs)
    ("hx_placement", placement::placement_case),
    // HelixGeom (hx_probe/helix_geom.rs, hx_probe/helix_geom_builders.rs)
    ("hx_curve", helix_geom::curve_case),
    ("hx_appr_curve3d", helix_geom::appr_curve3d_case),
    ("hx_appr_helix", helix_geom::appr_helix_case),
    ("hx_coil", helix_geom_builders::coil_case),
    ("hx_helix", helix_geom_builders::helix_case),
    // HelixBRep (hx_probe/helix_brep.rs)
    ("hxb_build", helix_brep::build_case),
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
