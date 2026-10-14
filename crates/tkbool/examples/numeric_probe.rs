//! Exact TKBool numeric oracle counterpart of tools/oracle/tkbool_probe.cxx: reads
//! `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod bl_probe;
use bl_probe::{
    brep_algo, brep_fill_advanced_evolved, brep_fill_compatible, brep_fill_draft,
    brep_fill_filling, brep_fill_generator, brep_fill_location_law, brep_fill_multi_line,
    brep_fill_offset_wire, brep_fill_package, brep_fill_pipe, brep_fill_pipe_shell,
    brep_fill_section, brep_fill_section_law, brep_fill_sweep, brep_proj, top_ope_brep_build,
    top_ope_brep_ds, top_ope_brep_ds_build, top_ope_brep_tool_classifier,
    top_ope_brep_tool_curve_tool, top_ope_brep_tool_shapes,
};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkbool_probe.cxx (`probe_cases`); each package adds
/// its operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // BRepFill (bl_probe/brep_fill_generator.rs, bl_probe/brep_fill_package.rs,
    // bl_probe/brep_fill_compatible.rs, bl_probe/brep_fill_section_law.rs,
    // bl_probe/brep_fill_location_law.rs, bl_probe/brep_fill_section.rs,
    // bl_probe/brep_fill_sweep.rs, bl_probe/brep_fill_pipe_shell.rs,
    // bl_probe/brep_fill_pipe.rs, bl_probe/brep_fill_draft.rs,
    // bl_probe/brep_fill_filling.rs, bl_probe/brep_fill_offset_wire.rs,
    // bl_probe/brep_fill_multi_line.rs, bl_probe/brep_fill_advanced_evolved.rs)
    ("bf_generator", brep_fill_generator::generator_case),
    ("bf_face", brep_fill_package::face_case),
    ("bf_shell", brep_fill_package::shell_case),
    ("bf_axe", brep_fill_package::axe_case),
    ("bf_search_origin", brep_fill_package::search_origin_case),
    ("bf_compute_acr", brep_fill_package::compute_acr_case),
    ("bf_insert_acr", brep_fill_package::insert_acr_case),
    ("bf_compatible", brep_fill_compatible::compatible_case),
    ("bf_shape_law", brep_fill_section_law::shape_law_case),
    ("bf_nsections", brep_fill_section_law::nsections_case),
    ("bf_location_law", brep_fill_location_law::location_law_case),
    ("bf_section", brep_fill_section::section_case),
    (
        "bf_section_placement",
        brep_fill_section::section_placement_case,
    ),
    ("bf_sweep", brep_fill_sweep::sweep_case),
    ("bf_pipe_shell", brep_fill_pipe_shell::pipe_shell_case),
    ("bf_pipe", brep_fill_pipe::pipe_case),
    ("bf_draft", brep_fill_draft::draft_case),
    ("bf_filling", brep_fill_filling::filling_case),
    ("bf_offset_wire", brep_fill_offset_wire::offset_wire_case),
    ("bf_multi_line", brep_fill_multi_line::multi_line_case),
    ("bf_trim_surface", brep_fill_multi_line::trim_surface_case),
    ("bf_evolved", bl_probe::brep_fill_evolved::evolved_case),
    (
        "bf_evolved_empty",
        bl_probe::brep_fill_evolved::evolved_empty_case,
    ),
    (
        "bf_advanced_evolved",
        brep_fill_advanced_evolved::advanced_evolved_case,
    ),
    // BRepAlgo (bl_probe/brep_algo.rs)
    ("ba_concat", brep_algo::concat_case),
    ("ba_concat_c0", brep_algo::concat_c0_case),
    ("ba_convert", brep_algo::convert_case),
    ("ba_valid", brep_algo::valid_case),
    ("ba_face_restrictor", brep_algo::face_restrictor_case),
    ("ba_cut_edge", brep_algo::cut_edge_case),
    ("ba_loop", brep_algo::loop_case),
    // BRepAlgo_NormalProjection and BRepProj (bl_probe/brep_proj.rs)
    ("ba_normal_projection", brep_proj::normal_projection_case),
    ("bp_projection", brep_proj::projection_case),
    // TopOpeBRepTool_CurveTool (bl_probe/top_ope_brep_tool_curve_tool.rs)
    (
        "tt_make_curves",
        top_ope_brep_tool_curve_tool::make_curves_case,
    ),
    (
        "tt_pcurve_on_face",
        top_ope_brep_tool_curve_tool::pcurve_on_face_case,
    ),
    // TopOpeBRepDS (bl_probe/top_ope_brep_ds.rs)
    ("ds_transition", top_ope_brep_ds::transition_case),
    ("ds_kind", top_ope_brep_ds::kind_case),
    (
        "ds_interference_tool",
        bl_probe::interference_tool::interference_tool_case,
    ),
    (
        "ds_point_explorer",
        bl_probe::point_explorer::point_explorer_case,
    ),
    (
        "ds_point_explorer_mutation",
        bl_probe::point_explorer::point_explorer_mutation_case,
    ),
    (
        "ds_curve_explorer",
        bl_probe::curve_explorer::curve_explorer_case,
    ),
    (
        "ds_curve_explorer_mutation",
        bl_probe::curve_explorer::curve_explorer_mutation_case,
    ),
    // TopOpeBRepTool shape helpers (bl_probe/top_ope_brep_tool_shapes.rs)
    ("tt_shape_tool", top_ope_brep_tool_shapes::shape_tool_case),
    ("tt_project", top_ope_brep_tool_shapes::project_case),
    ("tt_geometry", top_ope_brep_tool_shapes::geometry_case),
    ("tt_fc2d", top_ope_brep_tool_shapes::fc2d_case),
    ("tt_hbox", top_ope_brep_tool_shapes::hbox_case),
    // TopOpeBRepDS data structure and build tool (bl_probe/top_ope_brep_ds_build.rs)
    ("ds_recompute", top_ope_brep_ds_build::recompute_case),
    ("ds_same_domain", top_ope_brep_ds_build::same_domain_case),
    ("ds_sort", top_ope_brep_ds_build::sort_case),
    (
        "tt_solid_classifier",
        top_ope_brep_tool_classifier::solid_classifier_case,
    ),
    (
        "tt_shape_classifier",
        top_ope_brep_tool_classifier::shape_classifier_case,
    ),
    ("tt_fsc", top_ope_brep_tool_classifier::fsc_case),
    ("tt_tggeom", top_ope_brep_tool_classifier::tggeom_case),
    ("tt_tool", bl_probe::top_ope_brep_tool_tool::tool_case),
    (
        "tt_topology",
        bl_probe::top_ope_brep_tool_topology::topology_case,
    ),
    (
        "tt_make_transition",
        bl_probe::top_ope_brep_tool_tool::make_transition_case,
    ),
    (
        "tt_mktondge",
        bl_probe::top_ope_brep_tool_tool::mktondge_case,
    ),
    ("tt_classi", bl_probe::top_ope_brep_tool_regu::classi_case),
    (
        "tt_regularize",
        bl_probe::top_ope_brep_tool_regu::regularize_case,
    ),
    ("tt_corriso", bl_probe::top_ope_brep_tool_regu::corriso_case),
    (
        "tt_fuse_edges",
        bl_probe::top_ope_brep_tool_regu::fuse_edges_case,
    ),
    (
        "tt_purge_internal",
        bl_probe::top_ope_brep_tool_regu::purge_internal_case,
    ),
    ("tb_wire_to_face", top_ope_brep_build::wire_to_face_case),
    ("tb_face_builder", top_ope_brep_build::face_builder_case),
    ("tb_wec", top_ope_brep_build::wec_case),
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
