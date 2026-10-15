//! Exact TKFillet numeric oracle counterpart of tools/oracle/tkfillet_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod fl_probe;
use fl_probe::{
    api, app_func, blend_func, brep_blend, builder0, builder0_solve, builder0_topo, chamfer,
    chfi_ds, chfi2d, chfi3d, chfikpart, fillet, line_builders, rst_rst, surf_rst, walking,
};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkfillet_probe.cxx (`probe_cases`); each
/// package adds its operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // ChFiDS (fl_probe/chfi_ds.rs)
    ("fl_spine", chfi_ds::spine_case),
    ("fl_arc_spine", chfi_ds::arc_spine_case),
    ("fl_fil_law", chfi_ds::fil_law_case),
    // BlendFunc (fl_probe/blend_func.rs)
    ("fl_const_rad", fillet::const_rad_case),
    ("fl_const_rad_inv", blend_func::const_rad_inv_case),
    ("fl_evol_rad", fillet::evol_rad_case),
    ("fl_evol_rad_inv", blend_func::evol_rad_inv_case),
    ("fl_chamfer", chamfer::chamfer_case),
    ("fl_chamfer_inv", chamfer::chamfer_inv_case),
    ("fl_rad_inv", brep_blend::rad_inv_case),
    ("fl_blend_tool", brep_blend::blend_tool_case),
    ("fl_surf_rst", surf_rst::surf_rst_case),
    ("fl_rst_rst", rst_rst::rst_rst_case),
    ("fl_app_func", app_func::app_func_case),
    ("fl_walking", walking::walking_case),
    ("fl_surf_rst_walk", line_builders::surf_rst_walk_case),
    ("fl_rst_rst_walk", line_builders::rst_rst_walk_case),
    // ChFi2d (fl_probe/chfi2d.rs)
    ("fl_chfi2d_fillet", chfi2d::fillet_case),
    ("fl_chfi2d_chamfer", chfi2d::chamfer_case),
    ("fl_chfi2d_builder", chfi2d::builder_case),
    // ChFi3d (fl_probe/chfi3d.rs)
    ("fl_hatching_domains", chfi3d::hatching_domains_case),
    ("fl_concave_side", chfi3d::concave_side_case),
    ("fl_search_sing", chfi3d::search_sing_case),
    // ChFi3d_Builder_0 (fl_probe/builder0.rs)
    ("fl_b0_scalars", builder0::scalars_case),
    ("fl_bound_srf", builder0::bound_srf_case),
    ("fl_bound_fac", builder0::bound_fac_case),
    ("fl_enlarge_box", builder0::enlarge_box_case),
    ("fl_b0_faces", builder0::faces_case),
    ("fl_b0_spines", builder0::spines_case),
    ("fl_inter_plane_edge", builder0::inter_plane_edge_case),
    ("fl_b0_pcurves", builder0::pcurves_case),
    ("fl_reparam_pcurv", builder0::reparam_pcurv_case),
    ("fl_project_pcurv", builder0::project_pcurv_case),
    ("fl_b0_bounds", builder0::bounds_case),
    ("fl_compute_arete", builder0_solve::compute_arete_case),
    ("fl_fil_common_point", builder0_solve::fil_common_point_case),
    ("fl_b0_stripes", builder0_solve::stripes_case),
    ("fl_b0_tols", builder0_solve::tols_case),
    ("fl_int_cs", builder0_solve::int_cs_case),
    ("fl_computes_int_pc", builder0_solve::computes_int_pc_case),
    ("fl_b0_inter2d", builder0_solve::inter2d_case),
    ("fl_b0_topology", builder0_topo::topology_case),
    ("fl_b0_smooth", builder0_topo::smooth_case),
    ("fl_compute_curves", builder0_topo::compute_curves_case),
    ("fl_int_traces", builder0_solve::int_traces_case),
    ("fl_el_spine", builder0_topo::el_spine_case),
    // ChFiKPart (fl_probe/chfikpart.rs)
    ("fl_kpart_fcts", chfikpart::fcts_case),
    ("fl_corner_spine", chfikpart::corner_spine_case),
    ("fl_kpart_make", chfikpart::make_case),
    ("fl_kpart_corner", chfikpart::corner_case),
    // BRepFilletAPI (fl_probe/api.rs)
    ("fl_make_fillet", api::make_fillet_case),
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
