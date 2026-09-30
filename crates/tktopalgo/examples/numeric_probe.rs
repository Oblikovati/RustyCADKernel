//! Exact TKTopAlgo numeric oracle counterpart of tools/oracle/tktopalgo_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod tp_probe;
use tp_probe::{bisector, bisector_algo, brep_builder_api, mat, mat_parts};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tktopalgo_probe.cxx (`probe_cases`); each
/// package adds its operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // BRepBuilderAPI (tp_probe/brep_builder_api.rs)
    ("tp_make_vertex", brep_builder_api::make_vertex_case),
    // Bisector (tp_probe/bisector.rs)
    ("tp_bisec_curves", bisector::bisec_curves_case),
    ("tp_bisec_curve_point", bisector::bisec_curve_point_case),
    ("tp_bisec_points", bisector::bisec_points_case),
    ("tp_bisec_ana", bisector::bisec_ana_case),
    ("tp_bisec_cc", bisector_algo::bisec_cc_case),
    ("tp_bisec_pc", bisector_algo::bisec_pc_case),
    ("tp_bisec_inter", bisector_algo::bisec_inter_case),
    ("tp_bisec_function_h", bisector_algo::bisec_function_h_case),
    (
        "tp_bisec_function_inter",
        bisector_algo::bisec_function_inter_case,
    ),
    ("tp_bisec_poly", bisector_algo::bisec_poly_case),
    ("tp_bisec_is_convex", bisector_algo::bisec_is_convex_case),
    // MAT and MAT2d (tp_probe/mat.rs, tp_probe/mat_parts.rs)
    ("tp_mat2d_locus", mat::mat2d_locus_case),
    ("tp_mat_list", mat_parts::mat_list_case),
    ("tp_mat_bisector", mat_parts::mat_bisector_case),
    ("tp_mat_arc", mat_parts::mat_arc_case),
    ("tp_mat2d_connexion", mat_parts::mat2d_connexion_case),
    ("tp_mat2d_cut", mat_parts::mat2d_cut_case),
    ("tp_mat2d_minipath", mat_parts::mat2d_minipath_case),
    ("tp_mat2d_tool", mat_parts::mat2d_tool_case),
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
