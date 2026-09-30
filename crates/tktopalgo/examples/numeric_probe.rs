//! Exact TKTopAlgo numeric oracle counterpart of tools/oracle/tktopalgo_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod tp_probe;
use tp_probe::{
    brep_builder_api, brep_class, brep_lib_edge, brep_lib_wire, brep_top_adaptor, int_curves_face,
};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tktopalgo_probe.cxx (`probe_cases`); each
/// package adds its operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // BRepBuilderAPI (tp_probe/brep_builder_api.rs)
    ("tp_make_vertex", brep_builder_api::make_vertex_case),
    // BRepClass (tp_probe/brep_class.rs)
    ("brc_edge", brep_class::edge_case),
    ("brc_explorer", brep_class::explorer_case),
    ("brc_segment", brep_class::segment_case),
    ("brc_checkpoint", brep_class::checkpoint_case),
    ("brc_intersect", brep_class::intersect_case),
    ("brc_intersect2", brep_class::intersect2_case),
    ("brc_classify", brep_class::classify_case),
    ("brc_classify3d", brep_class::classify3d_case),
    ("brc_passive", brep_class::passive_case),
    ("brc_defaults", brep_class::defaults_case),
    // BRepTopAdaptor (tp_probe/brep_top_adaptor.rs)
    ("bta_fclass2d", brep_top_adaptor::fclass2d_case),
    ("bta_destroy", brep_top_adaptor::destroy_case),
    ("bta_topol", brep_top_adaptor::topol_case),
    ("bta_classify", brep_top_adaptor::classify_case),
    ("bta_tool", brep_top_adaptor::tool_case),
    ("bta_errors", brep_top_adaptor::errors_case),
    // IntCurvesFace (tp_probe/int_curves_face.rs)
    ("icf_line", int_curves_face::line_case),
    ("icf_curve", int_curves_face::curve_case),
    ("icf_shape", int_curves_face::shape_case),
    // BRepLib (tp_probe/brep_lib_*.rs)
    ("blm_vertex", brep_lib_edge::vertex_case),
    ("blm_edge", brep_lib_edge::edge_case),
    ("blm_edge2d", brep_lib_edge::edge2d_case),
    ("blm_polygon", brep_lib_wire::polygon_case),
    ("blm_wire", brep_lib_wire::wire_case),
    ("blm_solid", brep_lib_wire::solid_case),
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
