//! Exact TKShHealing numeric oracle counterpart of tools/oracle/tkshhealing_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod sh_probe;
use sh_probe::{
    complex_curve, composite_surface, explorer, registrator, reshape, shape_build, shape_extend,
    wire_data,
};
use sh_probe::{modification, shape_custom};

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
    // ShapeCustom (sh_probe/shape_custom.rs, sh_probe/modification.rs)
    ("sc_curve2d", shape_custom::curve2d_case),
    ("sc_restriction", shape_custom::restriction_case),
    ("sc_modif", modification::modif_case),
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
