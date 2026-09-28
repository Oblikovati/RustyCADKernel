//! Exact Geom (TKG3d) numeric oracle counterpart of tools/oracle/tkg3d_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod g3_probe;
use g3_probe::{
    advapprox, bezier_surface, bspl, bspline_surface, conics, curves, geom_adaptor_cases as a3,
    offset_surface, simple, surfaces, swept,
};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkg3d_probe.cxx (`probe_cases`).
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    ("g3_point", simple::point_case),
    ("g3_direction", simple::direction_case),
    ("g3_vector", simple::vector_case),
    ("g3_axis1", simple::axis1_case),
    ("g3_axis2", simple::axis2_case),
    ("g3_trsf", simple::trsf_case),
    ("g3_curve", curves::curve_case),
    ("g3_transform", curves::transform_case),
    ("g3_line", curves::line_case),
    ("g3_conic", conics::conic_case),
    ("g3_conic_build", conics::conic_build_case),
    ("g3_trimmed", curves::trimmed_case),
    ("g3_offset_utils", curves::offset_utils_case),
    ("g3_offset", curves::offset_case),
    ("g3_surface", surfaces::surface_case),
    ("g3_surface_moves", surfaces::surface_moves_case),
    ("g3_elementary", surfaces::elementary_case),
    ("g3_elementary_build", surfaces::elementary_build_case),
    ("g3_swept", swept::swept_case),
    ("g3_surface_evalrep", swept::surface_evalrep_case),
    ("g3_advapprox", advapprox::advapprox_case),
    ("g3_advapprox_static", advapprox::advapprox_static_case),
    ("g3_advapprox_simple", advapprox::advapprox_simple_case),
    ("g3_advapprox_cut", advapprox::advapprox_cut_case),
    ("g3_bezier", bspl::bezier_case),
    ("g3_bspline", bspl::bspline_case),
    ("g3_evalrep", bspl::evalrep_case),
    ("g3_evalrep_edit", bspl::evalrep_edit_case),
    ("g3_bezier_surface", bezier_surface::bezier_surface_case),
    (
        "g3_bezier_surface_evalrep",
        bezier_surface::bezier_surface_evalrep_case,
    ),
    ("g3_bspline_surface", bspline_surface::bspline_surface_case),
    (
        "g3_bspline_surface_edit",
        bspline_surface::bspline_surface_edit_case,
    ),
    ("g3_rect_trim", offset_surface::rect_trim_case),
    ("g3_offset_surface", offset_surface::offset_surface_case),
    ("a3_base", a3::base_case),
    ("a3_curve", a3::curve_case),
    ("a3_curve_load", a3::curve_load_case),
    ("a3_curve_evalrep", a3::curve_evalrep_case),
    ("a3_surface", a3::surface_case),
    ("a3_surface_load", a3::surface_load_case),
    ("a3_surface_evalrep", a3::surface_evalrep_case),
    ("a3_extrusion", a3::extrusion_case),
    ("a3_revolution", a3::revolution_case),
    ("a3_transformed", a3::transformed_case),
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
