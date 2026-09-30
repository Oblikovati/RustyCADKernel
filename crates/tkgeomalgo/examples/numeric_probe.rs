//! Exact TKGeomAlgo numeric oracle counterpart of tools/oracle/tkgeomalgo_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod ga_probe;
use ga_probe::geom_fill_laws as gfl;
use ga_probe::geom_fill_pipe as gfg;
use ga_probe::geom_fill_placement as gfp;
use ga_probe::geom_fill_sweep as gfs;
use ga_probe::geom_fill_sweepalgo as gfw;
use ga_probe::geom_fill_trihedron as gft;
use ga_probe::int_curve_surface_inter as ics_inter;
use ga_probe::{app_blend, fair_curve, int_imp, int_surf, law, local_analysis, plate};
use ga_probe::{
    gcc_ana, gcc_ana_lines, gcc_ent, gcc_int, hatch, intf, intf_poly, intf_tool, toptrans,
};
use ga_probe::{geom_api, geom_api_fit, geom_api_surface, gordon};
use ga_probe::{geom_plate, geom_plate_build};
use ga_probe::{geom2d_api, geom2d_int, ginter, int_curve, int_imp_par_gen, int_res2d};
use ga_probe::{geom2d_gcc, geom2d_gcc_circles, geom2d_gcc_lines};
use ga_probe::{geom2d_hatch, geom2d_hatcher, hatch_gen};
use ga_probe::{gfill, gfill_cf, gfill_fill};
use ga_probe::{ics_interutils, ics_polyhedron, int_curve_surface};
use ga_probe::{int_patch_impimp, int_patch_impprm};
use ga_probe::{int_patch, int_patch_curves, int_patch_lines, int_patch_poly, int_patch_tools};
use ga_probe::{int_polyh, int_polyh_records, int_start, int_walk, int_walk_iw};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkgeomalgo_probe.cxx (`probe_cases`); each
/// package adds its operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // Intf (ga_probe/intf*.rs)
    ("intf_triangle", intf::triangle_case),
    ("intf_spoint", intf::spoint_case),
    ("intf_sline", intf::sline_case),
    ("intf_tzone", intf::tzone_case),
    ("intf_interference", intf::interference_case),
    ("intf_tool2d", intf_tool::tool2d_case),
    ("intf_tool3d", intf_tool::tool3d_case),
    ("intf_ipoly2d", intf_poly::ipoly2d_case),
    ("intf_ipp", intf_poly::ipp_case),
    // TopTrans (ga_probe/toptrans.rs)
    ("tt_curve", toptrans::curve_case),
    ("tt_surface", toptrans::surface_case),
    ("tt_states", toptrans::states_case),
    // GccEnt (ga_probe/gcc_ent.rs)
    ("gccent_position", gcc_ent::position_case),
    ("gccent_qualified", gcc_ent::qualified_case),
    // GccInt (ga_probe/gcc_int.rs)
    ("gccint_bisec", gcc_int::bisec_case),
    // GccAna (ga_probe/gcc_ana.rs)
    ("gccana_circ3tan", gcc_ana::circ3tan_case),
    ("gccana_2tanrad", gcc_ana::tanrad_case),
    ("gccana_2tanon", gcc_ana::tanon_case),
    ("gccana_tanonrad", gcc_ana::tanonrad_case),
    ("gccana_tancen", gcc_ana::tancen_case),
    ("gccana_lin2tan", gcc_ana_lines::lin2tan_case),
    ("gccana_lintanobl", gcc_ana_lines::lintanobl_case),
    ("gccana_linbisec", gcc_ana_lines::linbisec_case),
    ("gccana_bisec", gcc_ana_lines::bisec_case),
    // Hatch (ga_probe/hatch.rs)
    ("hatch_hatcher", hatch::hatcher_case),
    // IntRes2d (ga_probe/int_res2d.rs)
    ("intres2d_domain", int_res2d::domain_case),
    ("intres2d_insert", int_res2d::insert_case),
    ("intres2d_append", int_res2d::append_case),
    // IntImpParGen (ga_probe/int_imp_par_gen.rs)
    ("imppar_transition", int_imp_par_gen::transition_case),
    ("imppar_domain", int_imp_par_gen::domain_case),
    // IntCurve (ga_probe/int_curve.rs)
    ("intcurve_conics", int_curve::conics_case),
    ("intcurve_imppar", int_curve::imppar_case),
    ("intcurve_itool", int_curve::itool_case),
    ("intcurve_ptool", int_curve::ptool_case),
    ("intcurve_function", int_curve::function_case),
    ("intcurve_interval", int_curve::interval_case),
    // Geom2dInt (ga_probe/geom2d_int.rs)
    ("g2int_tool", geom2d_int::tool_case),
    ("g2int_proj", geom2d_int::proj_case),
    ("g2int_conic_curve", geom2d_int::conic_curve_case),
    ("g2int_functions", geom2d_int::functions_case),
    // Geom2dGcc (ga_probe/geom2d_gcc*.rs)
    ("g2gcc_qualified", geom2d_gcc::qualified_case),
    ("g2gcc_tool", geom2d_gcc::tool_case),
    ("g2gcc_func1", geom2d_gcc::func1_case),
    ("g2gcc_func2", geom2d_gcc::func2_case),
    ("g2gcc_func3", geom2d_gcc::func3_case),
    ("g2gcc_func4", geom2d_gcc::func4_case),
    ("g2gcc_lin2tan", geom2d_gcc_lines::lin2tan_case),
    ("g2gcc_lin2tan_iter", geom2d_gcc_lines::lin2tan_iter_case),
    ("g2gcc_lintanobl", geom2d_gcc_lines::lintanobl_case),
    ("g2gcc_lintanobl_iter", geom2d_gcc_lines::obl_iter_case),
    ("g2gcc_tancen", geom2d_gcc_circles::tancen_case),
    ("g2gcc_tancen_geo", geom2d_gcc_circles::tancen_geo_case),
    ("g2gcc_2tanrad", geom2d_gcc_circles::tanrad_case),
    ("g2gcc_2tanrad_geo", geom2d_gcc_circles::tanrad_geo_case),
    ("g2gcc_3tan", geom2d_gcc_circles::circ3tan_case),
    ("g2gcc_3tan_iter", geom2d_gcc_circles::circ3tan_iter_case),
    ("g2gcc_2tanon", geom2d_gcc_circles::tanon_case),
    ("g2gcc_2tanon_geo", geom2d_gcc_circles::tanon_geo_case),
    ("g2gcc_2tanon_iter", geom2d_gcc_circles::tanon_iter_case),
    // Geom2dAPI (ga_probe/geom2d_api.rs)
    ("g2api_extrema", geom2d_api::extrema_case),
    ("g2api_project", geom2d_api::project_case),
    ("g2api_interpolate", geom2d_api::interpolate_case),
    ("g2api_points_to_bspline", geom2d_api::bspline_fit_case),
    // Geom2dInt curve-curve, Geom2dAPI_InterCurveCurve (ga_probe/ginter.rs)
    ("g2int_polygon", ginter::polygon_case),
    ("g2int_exact", ginter::exact_case),
    ("g2int_polypoly", ginter::polypoly_case),
    ("g2int_ginter", ginter::ginter_case),
    ("g2api_intercc", ginter::intercc_case),
    // IntSurf (ga_probe/int_surf.rs)
    ("intsurf_quadric_point", int_surf::quadric_point_case),
    ("intsurf_quadric_uv", int_surf::quadric_uv_case),
    ("intsurf_transition", int_surf::transition_case),
    ("intsurf_same", int_surf::same_case),
    // Law (ga_probe/law.rs)
    ("law_bspline", law::bspline_case),
    ("law_edit", law::edit_case),
    ("law_function", law::function_case),
    ("law_mix", law::mix_case),
    ("law_interpolate", law::interpolate_case),
    // Plate (ga_probe/plate.rs)
    ("plate_solve", plate::solve_case),
    ("plate_constraint", plate::constraint_case),
    // LocalAnalysis (ga_probe/local_analysis.rs)
    ("la_curve", local_analysis::curve_case),
    ("la_surface", local_analysis::surface_case),
    // FairCurve (ga_probe/fair_curve.rs)
    ("fc_law", fair_curve::law_case),
    ("fc_density", fair_curve::density_case),
    ("fc_energy", fair_curve::energy_case),
    ("fc_batten", fair_curve::batten_case),
    // IntImp (ga_probe/int_imp.rs)
    ("ii_choix", int_imp::choix_case),
    ("ii_tangence", int_imp::tangence_case),
    ("ii_zerpar", int_imp::zerpar_case),
    ("ii_int2s", int_imp::int2s_case),
    ("ii_zerimp", int_imp::zerimp_case),
    ("ii_intcs", int_imp::intcs_case),
    // AppBlend (ga_probe/app_blend.rs)
    ("ab_appsurf", app_blend::appsurf_case),
    ("ab_context", app_blend::context_case),
    // HatchGen (ga_probe/hatch_gen.rs)
    ("hgen_element", hatch_gen::element_case),
    ("hgen_hatching", hatch_gen::hatching_case),
    ("hgen_domain", hatch_gen::domain_case),
    // Geom2dHatch (ga_probe/geom2d_hatch.rs, ga_probe/geom2d_hatcher.rs)
    ("g2hatch_local", geom2d_hatch::local_case),
    ("g2hatch_perform", geom2d_hatch::perform_case),
    ("g2hatch_intersect", geom2d_hatch::intersect_case),
    ("g2hatch_elements", geom2d_hatch::elements_case),
    ("g2hatch_classify", geom2d_hatch::classify_case),
    ("g2hatch_fclass", geom2d_hatch::fclass_case),
    ("g2hatch_hatcher", geom2d_hatcher::hatcher_case),
    ("g2hatch_hatching", geom2d_hatcher::hatching_case),
    // IntWalk (ga_probe/int_walk.rs, ga_probe/int_walk_iw.rs)
    ("pw_walk", int_walk::walk_case),
    ("pw_first", int_walk::first_case),
    ("pw_edit", int_walk::edit_case),
    ("pw_min", int_walk::min_case),
    ("iw_walk", int_walk_iw::walk_case),
    // IntCurveSurface (ga_probe/int_curve_surface*.rs)
    ("ics_records", int_curve_surface::records_case),
    ("ics_intersection", int_curve_surface::intersection_case),
    ("ics_hcurve", int_curve_surface::hcurve_case),
    ("ics_polygon", int_curve_surface::polygon_case),
    ("ics_polyhedron", ics_polyhedron::polyhedron_case),
    ("ics_utils", int_curve_surface::utils_case),
    ("ics_interference", ics_inter::interference_case),
    ("ics_exact", ics_inter::exact_case),
    ("ics_quadcurv", ics_inter::quadcurv_case),
    ("ics_hinter", ics_inter::hinter_case),
    ("ics_hinter_protected", ics_inter::hinter_protected_case),
    ("ics_estlim", ics_inter::estlim_case),
    ("ics_project", ics_inter::project_case),
    ("ics_interutils", ics_interutils::interutils_case),
    // GeomFill filling part (ga_probe/gfill*.rs)
    ("gfill_line", gfill::line_case),
    ("gfill_profiler", gfill::profiler_case),
    ("gfill_secgen", gfill::secgen_case),
    ("gfill_appsurf", gfill::appsurf_case),
    ("gfill_generator", gfill_fill::generator_case),
    ("gfill_scheme", gfill_fill::scheme_case),
    ("gfill_bezier", gfill_fill::bezier_case),
    ("gfill_bspline", gfill_fill::bspline_case),
    ("gfill_bound", gfill_cf::bound_case),
    ("gfill_corner", gfill_cf::corner_case),
    ("gfill_patch", gfill_cf::patch_case),
    ("gfill_cfill", gfill_cf::cfill_case),
    // IntStart (ga_probe/int_start.rs)
    ("is_sob", int_start::sob_case),
    ("is_inside", int_start::inside_case),
    // IntPolyh (ga_probe/int_polyh*.rs)
    ("ph_inter", int_polyh::inter_case),
    ("ph_maillage", int_polyh::maillage_case),
    ("ph_tricontact", int_polyh::tricontact_case),
    ("ph_edgecontact", int_polyh::edgecontact_case),
    ("ph_point", int_polyh_records::point_case),
    ("ph_triangle", int_polyh::triangle_case),
    ("ph_startpoint", int_polyh_records::startpoint_case),
    ("ph_sectionline", int_polyh_records::sectionline_case),
    ("ph_array", int_polyh_records::array_case),
    ("ph_tools", int_polyh_records::tools_case),
    ("ph_couple", int_polyh_records::couple_case),
    // GeomAPI (ga_probe/geom_api*.rs)
    ("gapi_ppc", geom_api::ppc_case),
    ("gapi_pps", geom_api::pps_case),
    ("gapi_ecc", geom_api::ecc_case),
    ("gapi_ecs", geom_api::ecs_case),
    ("gapi_ess", geom_api::ess_case),
    ("gapi_to2d", geom_api_fit::to2d_case),
    ("gapi_to3d", geom_api_fit::to3d_case),
    ("gapi_intcs", geom_api_fit::intcs_case),
    ("gapi_interpolate", geom_api_fit::interpolate_case),
    ("gapi_points_to_bspline", geom_api_fit::p2bs_case),
    ("gapi_p2bss", geom_api_surface::p2bss_case),
    // GeomFill Gordon (ga_probe/gordon.rs)
    ("gfill_gordon", gordon::gordon_case),
    ("gfill_network", gordon::network_case),
    // IntPatch (ga_probe/int_patch*.rs)
    ("ip_point", int_patch::point_case),
    ("ip_iwline", int_patch::iwline_case),
    ("ip_pathpoint", int_patch::pathpoint_case),
    ("ip_wline", int_patch_lines::wline_case),
    ("ip_rline", int_patch_lines::rline_case),
    ("ip_gline", int_patch_curves::gline_case),
    ("ip_aline", int_patch_curves::aline_case),
    ("ip_hinter", int_patch_tools::hinter_case),
    ("ip_hc2d", int_patch_tools::hc2d_case),
    ("ip_arcf", int_patch_tools::arcf_case),
    ("ip_csf", int_patch_tools::csf_case),
    ("ip_curvature", int_patch_tools::curvature_case),
    // IntPatch analytic intersections (ga_probe/int_patch_impimp.rs)
    ("ip_impimp", int_patch_impimp::impimp_case),
    ("ip_aline2wline", int_patch_impimp::aline2wline_case),
    ("ip_special", int_patch_impimp::special_case),
    ("ip_special_pole", int_patch_impimp::special_pole_case),
    // IntPatch quadric-parametric intersections (ga_probe/int_patch_impprm.rs)
    ("ip_impprm", int_patch_impprm::impprm_case),
    ("ip_rstint", int_patch_impprm::rstint_case),
    // GeomFill sweep (ga_probe/geom_fill_*.rs)
    ("gfs_shape", gfs::shape_case),
    ("gfs_circle", gfs::circle_case),
    ("gfs_convert", gfs::convert_case),
    ("gfs_tensor", gfs::tensor_case),
    ("gfs_trihedron", gft::law_case),
    ("gfs_section", gfl::section_case),
    ("gfs_location", gfl::location_case),
    ("gfs_placement", gfp::placement_case),
    ("gfs_sweepfunc", gfw::function_case),
    ("gfs_sweep", gfw::sweep_case),
    ("gfs_blendfunc", gfw::blend_case),
    ("gfs_sweepsec", gfg::sweepsec_case),
    ("gfs_pipe", gfg::pipe_case),
    ("gfs_surface", gfg::surface_case),
    ("ip_polyh", int_patch_poly::polyh_case),
    ("ip_bvh", int_patch_poly::bvh_case),
    ("ip_interf", int_patch_poly::interf_case),
    ("ip_polyline", int_patch_poly::polyline_case),
    ("ip_polyarc", int_patch_poly::polyarc_case),
    ("ip_t3bits", int_patch_poly::t3bits_case),
    // GeomPlate (ga_probe/geom_plate*.rs)
    ("gplate_point", geom_plate::point_case),
    ("gplate_curve", geom_plate::curve_case),
    ("gplate_curve_empty", geom_plate::curve_empty_case),
    ("gplate_surface", geom_plate::surface_case),
    ("gplate_average", geom_plate::average_case),
    ("gplate_normals", geom_plate::normals_case),
    ("gplate_halfspace", geom_plate::halfspace_case),
    ("gplate_build", geom_plate_build::build_case),
    ("gplate_approx", geom_plate_build::approx_case),
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
