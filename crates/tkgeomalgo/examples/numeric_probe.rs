//! Exact TKGeomAlgo numeric oracle counterpart of tools/oracle/tkgeomalgo_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod ga_probe;
use ga_probe::{intf, intf_poly, intf_tool, toptrans};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkgeomalgo_probe.cxx (`probe_cases`); each
/// package adds its operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // Intf (ga_probe/intf*.rs)
    ("intf_triangle", intf::triangle_case),
    ("intf_spoint", intf::spoint_case),
    ("intf_sline", intf::sline_case),
    ("intf_tzone", intf::tzone_case),
    ("intf_interference", intf::interference_case),
    ("intf_tool2d", intf_tool::tool2d_case),
    ("intf_tool3d", intf_tool::tool3d_case),
    ("intf_ipoly2d", intf_poly::ipoly2d_case),
    ("intf_ipp", intf_poly::ipp_case),
    // TopTrans (ga_probe/toptrans.rs)
    ("tt_curve", toptrans::curve_case),
    ("tt_surface", toptrans::surface_case),
    ("tt_states", toptrans::states_case),
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
