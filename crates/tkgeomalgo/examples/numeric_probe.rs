//! Exact TKGeomAlgo numeric oracle counterpart of tools/oracle/tkgeomalgo_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod ga_probe;
use ga_probe::{int_curve, int_imp_par_gen, int_res2d, intf};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkgeomalgo_probe.cxx (`probe_cases`); each
/// package adds its operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // Intf (ga_probe/intf.rs)
    ("intf_triangle", intf::triangle_case),
    // IntRes2d (ga_probe/int_res2d.rs)
    ("intres2d_domain", int_res2d::domain_case),
    ("intres2d_insert", int_res2d::insert_case),
    ("intres2d_append", int_res2d::append_case),
    // IntImpParGen (ga_probe/int_imp_par_gen.rs)
    ("imppar_transition", int_imp_par_gen::transition_case),
    ("imppar_domain", int_imp_par_gen::domain_case),
    // IntCurve (ga_probe/int_curve.rs)
    ("intcurve_conics", int_curve::conics_case),
    ("intcurve_imppar", int_curve::imppar_case),
    ("intcurve_itool", int_curve::itool_case),
    ("intcurve_ptool", int_curve::ptool_case),
    ("intcurve_function", int_curve::function_case),
    ("intcurve_interval", int_curve::interval_case),
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
