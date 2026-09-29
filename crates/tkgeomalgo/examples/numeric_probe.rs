//! Exact TKGeomAlgo numeric oracle counterpart of tools/oracle/tkgeomalgo_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod ga_probe;
use ga_probe::{geom2d_api, geom2d_int, ginter, int_curve, int_imp_par_gen, int_res2d};
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
    // Geom2dInt (ga_probe/geom2d_int.rs)
    ("g2int_tool", geom2d_int::tool_case),
    ("g2int_proj", geom2d_int::proj_case),
    ("g2int_conic_curve", geom2d_int::conic_curve_case),
    ("g2int_functions", geom2d_int::functions_case),
    // Geom2dAPI (ga_probe/geom2d_api.rs)
    ("g2api_extrema", geom2d_api::extrema_case),
    ("g2api_project", geom2d_api::project_case),
    ("g2api_interpolate", geom2d_api::interpolate_case),
    ("g2api_points_to_bspline", geom2d_api::bspline_fit_case),
    // Geom2dInt curve-curve, Geom2dAPI_InterCurveCurve (ga_probe/ginter.rs)
    ("g2int_polygon", ginter::polygon_case),
    ("g2int_exact", ginter::exact_case),
    ("g2int_polypoly", ginter::polypoly_case),
    ("g2int_ginter", ginter::ginter_case),
    ("g2api_intercc", ginter::intercc_case),
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
