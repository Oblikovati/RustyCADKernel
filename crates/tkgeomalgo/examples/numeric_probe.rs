//! Exact TKGeomAlgo numeric oracle counterpart of tools/oracle/tkgeomalgo_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod ga_probe;
use ga_probe::{int_surf, intf, law, plate};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkgeomalgo_probe.cxx (`probe_cases`); each
/// package adds its operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // Intf (ga_probe/intf.rs)
    ("intf_triangle", intf::triangle_case),
    // IntSurf (ga_probe/int_surf.rs)
    ("intsurf_quadric_point", int_surf::quadric_point_case),
    ("intsurf_quadric_uv", int_surf::quadric_uv_case),
    ("intsurf_transition", int_surf::transition_case),
    ("intsurf_same", int_surf::same_case),
    // Law (ga_probe/law.rs)
    ("law_bspline", law::bspline_case),
    ("law_edit", law::edit_case),
    ("law_function", law::function_case),
    ("law_mix", law::mix_case),
    ("law_interpolate", law::interpolate_case),
    // Plate (ga_probe/plate.rs)
    ("plate_solve", plate::solve_case),
    ("plate_constraint", plate::constraint_case),
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
