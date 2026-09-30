//! Exact TKShHealing numeric oracle counterpart of tools/oracle/tkshhealing_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod sh_probe;
use sh_probe::analysis_basic;
use sh_probe::fix_basic;
use sh_probe::{
    complex_curve, composite_surface, explorer, registrator, reshape, shape_build, shape_extend,
    wire_data,
};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkshhealing_probe.cxx (`probe_cases`); each
/// package adds its operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
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
