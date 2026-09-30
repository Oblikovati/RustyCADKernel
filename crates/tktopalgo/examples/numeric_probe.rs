//! Exact TKTopAlgo numeric oracle counterpart of tools/oracle/tktopalgo_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod tp_probe;
use tp_probe::{
    bisector, bisector_algo, brep_approx, brep_approx_parts, brep_approx_tool, brep_bnd_lib,
    brep_bnd_lib_obb, brep_builder_api, brep_builder_api_makers, brep_builder_api_modify,
    brep_builder_api_on_mesh, brep_check, brep_class, brep_class3d, brep_gprop, brep_gprop_frames,
    brep_gprop_tools, brep_int_curve_surface, brep_lib_edge, brep_lib_face, brep_lib_fuse,
    brep_lib_package, brep_lib_point_cloud, brep_lib_wire, brep_mat2d, brep_top_adaptor,
    brep_top_adaptor_domain, fast_sewing, int_curves_face, mat, mat_parts, sewing, sewing_helpers,
};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tktopalgo_probe.cxx (`probe_cases`); each
/// package adds its operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // BRepBuilderAPI (tp_probe/brep_builder_api.rs)
    ("tp_make_vertex", brep_builder_api::make_vertex_case),
    // BRepBuilderAPI makers and modifications (tp_probe/brep_builder_api_makers.rs,
    // tp_probe/brep_builder_api_modify.rs)
    ("bba_vertex", brep_builder_api_makers::vertex_case),
    ("bba_package", brep_builder_api_makers::package_case),
    ("bba_edge", brep_builder_api_makers::edge_case),
    ("bba_edge2d", brep_builder_api_makers::edge2d_case),
    ("bba_polygon", brep_builder_api_makers::polygon_case),
    ("bba_wire", brep_builder_api_makers::wire_case),
    ("bba_solid", brep_builder_api_makers::solid_case),
    ("bba_face", brep_builder_api_makers::face_case),
    ("bba_shell", brep_builder_api_makers::shell_case),
    ("bba_transform", brep_builder_api_modify::transform_case),
    ("bba_gtransform", brep_builder_api_modify::gtransform_case),
    ("bba_copy", brep_builder_api_modify::copy_case),
    ("bba_nurbs", brep_builder_api_modify::nurbs_case),
    ("bba_modify", brep_builder_api_modify::modify_case),
    ("bba_findplane", brep_builder_api_modify::findplane_case),
    ("bba_collect", brep_builder_api_modify::collect_case),
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
    // BRepCheck and BRepLib_ValidateEdge (tp_probe/brep_check.rs, shapes of
    // tp_probe/shapes.rs)
    ("bck_status", brep_check::status_case),
    ("bck_add", brep_check::add_case),
    ("bck_prec", brep_check::prec_case),
    ("bck_selfint", brep_check::selfint_case),
    ("bck_check", brep_check::check_case),
    ("ble_validate", brep_check::validate_case),
    ("bck_analyze", brep_check::analyze_case),
    // BRepBndLib (tp_probe/brep_bnd_lib.rs, shapes of tp_probe/shape.rs)
    ("bbl_add", brep_bnd_lib::add_case),
    // BRepBndLib::AddOBB (tp_probe/brep_bnd_lib_obb.rs)
    ("bbl_obb", brep_bnd_lib_obb::obb_case),
    // BRepLib_PointCloudShape (tp_probe/brep_lib_point_cloud.rs)
    ("blp_pointcloud", brep_lib_point_cloud::point_cloud_case),
    // BRepBuilderAPI_MakeShapeOnMesh (tp_probe/brep_builder_api_on_mesh.rs)
    ("bba_onmesh", brep_builder_api_on_mesh::on_mesh_case),
    // BRepMAT2d (tp_probe/brep_mat2d.rs)
    ("bm2_locus", brep_mat2d::locus_case),
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
    // BRepTopAdaptor domains of IntStart and IntPatch (tp_probe/brep_top_adaptor_domain.rs)
    ("bta_sob", brep_top_adaptor_domain::sob_case),
    ("bta_inside", brep_top_adaptor_domain::inside_case),
    (
        "bta_intersection",
        brep_top_adaptor_domain::intersection_case,
    ),
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
    ("blp_tolerances", brep_lib_package::tolerances_case),
    ("blp_orient", brep_lib_package::orient_case),
    ("blm_face", brep_lib_face::face_case),
    ("blm_facedegen", brep_lib_face::facedegen_case),
    ("blp_findsurface", brep_lib_face::findsurface_case),
    ("blp_sameparam", brep_lib_face::sameparam_case),
    ("blp_extendface", brep_lib_face::extendface_case),
    ("blp_fuseedges", brep_lib_fuse::fuseedges_case),
    // BRepGProp (tp_probe/brep_gprop.rs, brep_gprop_frames.rs, brep_gprop_tools.rs, shapes of
    // tp_probe/shape.rs)
    ("bgp_package", brep_gprop::package_case),
    ("bgp_cinert", brep_gprop::cinert_case),
    ("bgp_mesh_cinert", brep_gprop::mesh_cinert_case),
    ("bgp_mesh_nodes", brep_gprop::mesh_nodes_case),
    ("bgp_mesh_props", brep_gprop::mesh_props_case),
    ("bgp_mesh_calc", brep_gprop::mesh_calc_case),
    ("bgp_sinert", brep_gprop_frames::sinert_case),
    ("bgp_vinert", brep_gprop_frames::vinert_case),
    ("bgp_vinert_gk", brep_gprop_frames::vinert_gk_case),
    ("bgp_face", brep_gprop_tools::face_case),
    ("bgp_knots", brep_gprop_tools::knots_case),
    ("bgp_arrays", brep_gprop_tools::arrays_case),
    ("bgp_domain", brep_gprop_tools::domain_case),
    ("bgp_gauss", brep_gprop_tools::gauss_case),
    ("bgp_functions", brep_gprop_tools::functions_case),
    // BRepApprox (tp_probe/brep_approx*.rs, faces of tp_probe/face.rs)
    ("bra_approx", brep_approx::approx_case),
    ("bra_line", brep_approx::line_case),
    ("bra_mline", brep_approx_parts::mline_case),
    ("bra_sv", brep_approx_parts::sv_case),
    ("bra_zerimp", brep_approx_parts::zerimp_case),
    ("bra_int2s", brep_approx_parts::int2s_case),
    ("bra_tool", brep_approx_tool::tool_case),
    // BRepIntCurveSurface (tp_probe/brep_int_curve_surface.rs, shapes of tp_probe/shape.rs)
    ("bics_line", brep_int_curve_surface::line_case),
    ("bics_curve", brep_int_curve_surface::curve_case),
    ("bics_state", brep_int_curve_surface::state_case),
    // BRepBuilderAPI sewing (tp_probe/sewing.rs, tp_probe/fast_sewing.rs and
    // tp_probe/sewing_helpers.rs, shapes of tp_probe/shape.rs)
    ("bba_sew", sewing::sew_request),
    ("bba_fastsew", fast_sewing::fastsew_request),
    ("bba_vertexinspector", sewing_helpers::vertexinspector_case),
    ("bba_boxselector", sewing_helpers::boxselector_case),
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
