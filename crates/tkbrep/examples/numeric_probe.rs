//! Exact TKBRep oracle counterpart of tools/oracle/tkbrep_probe.cxx: reads
//! `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod br_probe;
use br_probe::{
    bintools, bintools_sets, bintools_shape, brep_adaptor, brep_report, brep_tools,
    brep_tools_modif, brep_tools_reshape, topexp, topods, toptools,
};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkbrep_probe.cxx (`probe_cases`); each package
/// adds its operations, in the same order as the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // TopoDS (br_probe/topods.rs)
    ("br_topods", topods::topods_case),
    ("br_dump", topods::dump_case),
    // TopExp (br_probe/topexp.rs)
    ("br_explore", topexp::explore_case),
    ("br_maps", topexp::maps_case),
    ("br_vertices", topexp::vertices_case),
    // TopTools (br_probe/toptools.rs)
    ("br_shapeset", toptools::shapeset_case),
    ("br_setread", toptools::setread_case),
    ("br_locset", toptools::locset_case),
    ("br_locread", toptools::locread_case),
    // BRep (br_probe/brep.rs, br_probe/brep_report.rs)
    ("br_brep", brep_report::brep_case),
    ("br_brep_dump", brep_report::brep_dump_case),
    // BinTools (br_probe/bintools*.rs)
    ("br_bin_scalars", bintools::scalars_case),
    ("br_bin_ostream", bintools::ostream_case),
    ("br_bin_istream", bintools::istream_case),
    ("br_bin_geometry", bintools_sets::geometry_case),
    ("br_bin_shape", bintools_shape::shape_case),
    ("br_bin_read", bintools_shape::read_case),
    ("br_bin_truncate", bintools_shape::truncate_case),
    // BRepTools (br_probe/brep_tools*.rs)
    ("br_tools_text", brep_tools::text_case),
    ("br_tools_query", brep_tools::query_case),
    ("br_tools_mutate", brep_tools::mutate_case),
    ("br_tools_modif", brep_tools_modif::modif_case),
    ("br_tools_wire", brep_tools_modif::wire_case),
    ("br_tools_reshape", brep_tools_reshape::reshape_case),
    ("br_tools_history", brep_tools_reshape::history_case),
    ("br_tools_subst", brep_tools_reshape::subst_case),
    ("br_tools_quilt", brep_tools_reshape::quilt_case),
    ("br_tools_purge", brep_tools_reshape::purge_case),
    // BRepAdaptor and BRepLProp (br_probe/brep_adaptor.rs)
    ("br_adaptor", brep_adaptor::adaptor_case),
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
