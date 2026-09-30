//! Exact TKTopAlgo numeric oracle counterpart of tools/oracle/tktopalgo_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod tp_probe;
use tp_probe::{
    bisector, bisector_algo, brep_bnd_lib, brep_builder_api, brep_class, brep_class3d,
    brep_lib_edge, brep_lib_package, brep_lib_wire, brep_top_adaptor, int_curves_face, mat,
    mat_parts,
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
    // BRepBndLib (tp_probe/brep_bnd_lib.rs, shapes of tp_probe/shape.rs)
    ("bbl_add", brep_bnd_lib::add_case),
    // BRepClass3d (tp_probe/brep_class3d.rs, shapes of tp_probe/shape.rs)
    ("bc3_classify", brep_class3d::classify_case),
    ("bc3_infinite", brep_class3d::infinite_case),
    ("bc3_sclassifier", brep_class3d::sclassifier_case),
    ("bc3_segment", brep_class3d::segment_case),
    ("bc3_explorer", brep_class3d::explorer_case),
    ("bc3_findpoint", brep_class3d::findpoint_case),
    ("bc3_pointinface", brep_class3d::pointinface_case),
    ("bc3_uvpoint", brep_class3d::uvpoint_case),
    ("bc3_tree", brep_class3d::tree_case),
    ("bc3_inter3d", brep_class3d::inter3d_case),
    // BRepLib (tp_probe/brep_lib_*.rs)
    ("blm_vertex", brep_lib_edge::vertex_case),
    ("blm_edge", brep_lib_edge::edge_case),
    ("blm_edge2d", brep_lib_edge::edge2d_case),
    ("blm_polygon", brep_lib_wire::polygon_case),
    ("blm_wire", brep_lib_wire::wire_case),
    ("blm_solid", brep_lib_wire::solid_case),
    ("blp_build3d", brep_lib_package::build3d_case),
    ("blp_samerange", brep_lib_package::samerange_case),
    ("blp_edgetol", brep_lib_package::edgetol_case),
    ("blp_regularity", brep_lib_package::regularity_case),
    ("blp_continuity", brep_lib_package::continuity_case),
    ("blp_inner", brep_lib_package::inner_case),
    ("blp_sort", brep_lib_package::sort_case),
    ("blp_bounding", brep_lib_package::bounding_case),
    ("blp_validrange", brep_lib_package::validrange_case),
    ("blp_validrange2", brep_lib_package::validrange2_case),
    ("blp_pcurveplane", brep_lib_package::pcurveplane_case),
    ("blp_ccos", brep_lib_package::ccos_case),
    ("blp_shell", brep_lib_package::shell_case),
    ("blp_normals", brep_lib_package::normals_case),
    ("blp_deflection", brep_lib_package::deflection_case),
    ("blp_trinormals", brep_lib_package::trinormals_case),
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
