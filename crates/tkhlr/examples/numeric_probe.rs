//! Exact TKHLR numeric oracle counterpart of tools/oracle/tkhlr_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod hl_probe;
use hl_probe::frame;
use hl_probe::intrv;
use hl_probe::top_cnx;
use hl_probe::{hlr_algo, hlr_algo_internal, hlr_algo_poly, hlr_algo_projector};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkhlr_probe.cxx (`probe_cases`); each
/// package adds its operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    ("ct_walking", hl_probe::contap_walking::walking_case),
    (
        "ct_boundary",
        hl_probe::contap_search_boundary::boundary_case,
    ),
    ("ct_inside", hl_probe::contap_search_inside::inside_case),
    (
        "ct_search_records",
        hl_probe::contap_search_records::records_case,
    ),
    ("ct_iwline", hl_probe::contap_iwline::iwline_case),
    (
        "ct_arc_function",
        hl_probe::contap_arc_function::arc_function_case,
    ),
    ("ct_function", hl_probe::contap_function::function_case),
    ("ct_sampling", hl_probe::contap_hcont::sampling_case),
    ("ct_hcont_arc", hl_probe::contap_hcont::arc_case),
    ("ct_hcurve", hl_probe::contap_hcurve::hc2d_case),
    ("ct_surface", hl_probe::contap_surface::properties_case),
    ("ct_analytic", hl_probe::contana::analytic_case),
    ("ct_point", hl_probe::contap::point_case),
    ("ct_line", hl_probe::contap::line_case),
    // HLRAlgo_Projector viewing frame (hl_probe/frame.rs)
    ("hl_frame", frame::frame_case),
    // Intrv (hl_probe/intrv.rs)
    ("iv_interval", intrv::interval_case),
    ("iv_edit", intrv::edit_case),
    ("iv_relate", intrv::relate_case),
    ("iv_set", intrv::set_case),
    ("iv_value", intrv::value_case),
    // TopBas and TopCnx (hl_probe/top_cnx.rs)
    ("tb_interference", top_cnx::interference_case),
    ("tc_transition", top_cnx::transition_case),
    // HLRAlgo (hl_probe/hlr_algo.rs, hlr_algo_projector.rs, hlr_algo_poly.rs,
    // hlr_algo_internal.rs)
    ("ha_minmax", hlr_algo::minmax_case),
    ("ha_encode", hlr_algo::encode_case),
    ("ha_bipoint", hlr_algo::bipoint_case),
    ("ha_edges", hlr_algo::edges_case),
    ("ha_wires", hlr_algo::wires_case),
    ("ha_status", hlr_algo::status_case),
    ("ha_records", hlr_algo::records_case),
    ("ha_projector", hlr_algo_projector::projector_case),
    ("ha_polydata", hlr_algo_poly::polydata_case),
    ("ha_polyalgo", hlr_algo_poly::polyalgo_case),
    ("ha_internal", hlr_algo_internal::internal_case),
    ("ha_grow", hlr_algo_internal::grow_case),
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
