//! Exact TKShHealing numeric oracle counterpart of tools/oracle/tkshhealing_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod sh_probe;
use sh_probe::analysis_basic;
use sh_probe::analysis_edge;
use sh_probe::canonical;
use sh_probe::edge_outputs;
use sh_probe::fix_basic;
use sh_probe::{
    complex_curve, composite_surface, explorer, registrator, reshape, shape_build, shape_extend,
    wire_data,
};
use sh_probe::{modification, shape_custom, shape_upgrade};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkshhealing_probe.cxx (`probe_cases`); each
/// package adds its operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    (
        "sh_split_surface",
        sh_probe::split_surface::split_surface_case,
    ),
    (
        "sh_split_curve3d",
        sh_probe::split_curve_3d::split_curve_3d_case,
    ),
    (
        "sh_split_curve2d",
        sh_probe::split_curve_2d::split_curve_2d_case,
    ),
    (
        "sh_fix_small_curves",
        sh_probe::fix_small_curves::fix_small_curves_case,
    ),
    (
        "sh_transfer_parameters",
        sh_probe::transfer_parameters::transfer_parameters_case,
    ),
    ("sh_edge_divide", sh_probe::edge_divide::edge_divide_case),
    ("sh_free_bounds", sh_probe::free_bounds::free_bounds_case),
    ("sh_shell_sewing", sh_probe::shell_sewing::shell_sewing_case),
    (
        "sh_analysis_planar",
        sh_probe::shape_analysis_curve::planar_case,
    ),
    (
        "sh_analysis_bounds",
        sh_probe::shape_analysis_curve::bounds_case,
    ),
    (
        "sh_analysis_validate",
        sh_probe::shape_analysis_curve::validate_case,
    ),
    (
        "sh_analysis_seam",
        sh_probe::shape_analysis_curve::seam_case,
    ),
    (
        "sh_analysis_samples3d",
        sh_probe::shape_analysis_curve::sampling_3d_case,
    ),
    (
        "sh_analysis_closure",
        sh_probe::shape_analysis_curve::closure_case,
    ),
    (
        "sh_analysis_samples2d",
        sh_probe::shape_analysis_curve::sampling_2d_case,
    ),
    (
        "sh_analysis_projection",
        sh_probe::shape_analysis_curve::projection_case,
    ),
    (
        "sh_analysis_period",
        sh_probe::shape_analysis_curve::period_case,
    ),
    (
        "sh_project_curve",
        sh_probe::project_curve::project_curve_case,
    ),
    (
        "sh_construct",
        sh_probe::shape_construct::construct_curve_case,
    ),
    (
        "sh_analysis_surface",
        sh_probe::analysis_surface::analysis_surface_case,
    ),
    ("sh_canonical", canonical::canonical_case),
    ("sh_edgeoutputs", edge_outputs::edgeoutputs_case),
    ("sh_edgeana", analysis_edge::edgeana_case),
    // ShapeExtend (sh_probe/shape_extend.rs)
    ("sh_status", shape_extend::status_case),
    ("sh_msg", shape_extend::message_case),
    // ShapeExtend_WireData (sh_probe/wire_data.rs)
    ("sh_wiredata", wire_data::wiredata_case),
    // ShapeExtend_Explorer (sh_probe/explorer.rs)
    ("sh_explorer", explorer::explorer_case),
    // ShapeExtend registrators (sh_probe/registrator.rs)
    ("sh_registrator", registrator::registrator_case),
    // ShapeExtend_ComplexCurve (sh_probe/complex_curve.rs)
    ("sh_complex", complex_curve::complex_case),
    // ShapeExtend_CompositeSurface (sh_probe/composite_surface.rs)
    ("sh_composite", composite_surface::composite_case),
    // ShapeBuild (sh_probe/shape_build.rs, sh_probe/reshape.rs)
    ("sh_vertex", shape_build::vertex_case),
    ("sh_edge", shape_build::edge_case),
    ("sh_reshape", reshape::reshape_case),
    // ShapeAnalysis_Geom, _ShapeTolerance, _Shell, _WireOrder, _FreeBoundData (sh_probe/analysis_basic.rs)
    ("sh_geom", analysis_basic::geom_case),
    ("sh_shapetol", analysis_basic::shapetol_case),
    ("sh_shell", analysis_basic::shell_case),
    ("sh_wireorder", analysis_basic::wireorder_case),
    ("sh_fbdata", analysis_basic::fbdata_case),
    // ShapeFix_Root, _ShapeTolerance, _EdgeConnect (sh_probe/fix_basic.rs)
    ("sh_fixroot", fix_basic::fixroot_case),
    ("sh_fixtol", fix_basic::fixtol_case),
    ("sh_edgeconnect", fix_basic::edgeconnect_case),
    // ShapeCustom (sh_probe/shape_custom.rs, sh_probe/modification.rs)
    ("sc_curve2d", shape_custom::curve2d_case),
    ("sc_restriction", shape_custom::restriction_case),
    ("sc_modif", modification::modif_case),
    // ShapeUpgrade (sh_probe/shape_upgrade.rs)
    ("su_c0split", shape_upgrade::c0split_case),
    ("su_tool", shape_upgrade::tool_case),
    ("su_removeloc", shape_upgrade::removeloc_case),
    ("su_splitcurve", shape_upgrade::splitcurve_case),
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
