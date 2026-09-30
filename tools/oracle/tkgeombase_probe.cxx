// Exact numeric oracle for the TKGeomBase packages at pinned OCCT 3d097a0.
// Each input line is `operation numbers...`; tools/verify_tkgeombase_numeric.py compares the
// output with crates/tkgeombase/examples/numeric_probe.rs bit for bit. Each package adds
// its own tkgeombase_probe_<package>.cxx, included below, and its rows in probe_cases.
#include <map>
#include <memory>
#include <vector>
#include <iostream>
#include <sstream>
#include <cstring>
#include <cstdint>
#include <Standard_Failure.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2.hxx>
#include <gp_Ax22d.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Ax3.hxx>
#include <gp_Dir.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_Vec.hxx>
#include <gp_Vec2d.hxx>
#include <gp_Circ.hxx>
#include <gp_Circ2d.hxx>
#include <gp_Cone.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Elips.hxx>
#include <gp_Elips2d.hxx>
#include <gp_Hypr.hxx>
#include <gp_Hypr2d.hxx>
#include <gp_Lin.hxx>
#include <gp_Lin2d.hxx>
#include <gp_Parab.hxx>
#include <gp_Parab2d.hxx>
#include <gp_Pln.hxx>
#include <gp_Trsf.hxx>
#include <gp_Trsf2d.hxx>
#include "tkgeombase_probe_common.cxx"
#include "tkgeombase_probe_gce.cxx"
#include "tkgeombase_probe_gc.cxx"
#include "tkgeombase_probe_intana2d.cxx"
#include "tkgeombase_probe_intana.cxx"
#include "tkgeombase_probe_lprop.cxx"
#include "tkgeombase_probe_gprop.cxx"
#include "tkgeombase_probe_hermit.cxx"
#include "tkgeombase_probe_extrema_pc2d.cxx"
#include "tkgeombase_probe_extrema_pc.cxx"
#include "tkgeombase_probe_geomtools.cxx"
#include "tkgeombase_probe_femtool.cxx"
#include "tkgeombase_probe_femtool_curve.cxx"
#include "tkgeombase_probe_appparcurves.cxx"
#include "tkgeombase_probe_appparcurves_fit.cxx"
#include "tkgeombase_probe_bndlib.cxx"
#include "tkgeombase_probe_gcpnts.cxx"
#include "tkgeombase_probe_appcont.cxx"
#include "tkgeombase_probe_advapp2var_math.cxx"
#include "tkgeombase_probe_advapp2var_math2.cxx"
#include "tkgeombase_probe_advapp2var_eval.cxx"
#include "tkgeombase_probe_advapp2var_f2var.cxx"
#include "tkgeombase_probe_advapp2var_f2var2.cxx"
#include "tkgeombase_probe_advapp2var_class.cxx"
#include "tkgeombase_probe_geom2dconvert.cxx"
#include "tkgeombase_probe_extrema_el.cxx"
#include "tkgeombase_probe_extrema_pc_classic.cxx"
#include "tkgeombase_probe_extrema_cc.cxx"
#include "tkgeombase_probe_extrema_ps.cxx"
#include "tkgeombase_probe_extrema_cs.cxx"
#include "tkgeombase_probe_extrema_ss.cxx"
#include "tkgeombase_probe_geomconvert.cxx"
#include "tkgeombase_probe_geomconvert_ana.cxx"
#include "tkgeombase_probe_geomlib.cxx"
#include "tkgeombase_probe_geomlib_classes.cxx"
#include "tkgeombase_probe_approx_line.cxx"
#include "tkgeombase_probe_approx.cxx"
#include "tkgeombase_probe_approx_adaptors.cxx"
#include "tkgeombase_probe_approx_sweep.cxx"
#include "tkgeombase_probe_appdef.cxx"
#include "tkgeombase_probe_appdef_variational.cxx"
#include "tkgeombase_probe_projlib.cxx"
#include "tkgeombase_probe_projlib_curves.cxx"
#include "tkgeombase_probe_geomprojlib.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // gce (tkgeombase_probe_gce.cxx)
 {"gce_circ", gb_gce_circ_case},
 {"gce_circ2d", gb_gce_circ2d_case},
 {"gce_cone", gb_gce_cone_case},
 {"gce_cylinder", gb_gce_cylinder_case},
 {"gce_dir", gb_gce_dir_case},
 {"gce_dir2d", gb_gce_dir2d_case},
 {"gce_elips", gb_gce_elips_case},
 {"gce_elips2d", gb_gce_elips2d_case},
 {"gce_hypr", gb_gce_hypr_case},
 {"gce_hypr2d", gb_gce_hypr2d_case},
 {"gce_lin", gb_gce_lin_case},
 {"gce_lin2d", gb_gce_lin2d_case},
 {"gce_parab", gb_gce_parab_case},
 {"gce_parab2d", gb_gce_parab2d_case},
 {"gce_pln", gb_gce_pln_case},
 {"gce_mirror", gb_gce_mirror_case},
 {"gce_mirror2d", gb_gce_mirror2d_case},
 {"gce_rotation", gb_gce_rotation_case},
 {"gce_rotation2d", gb_gce_rotation2d_case},
 {"gce_scale", gb_gce_scale_case},
 {"gce_scale2d", gb_gce_scale2d_case},
 {"gce_translation", gb_gce_translation_case},
 {"gce_translation2d", gb_gce_translation2d_case},
 {"gce_status", gb_gce_status_case},
 // GC (tkgeombase_probe_gc.cxx)
 {"gc_arc_circle", gb_gc_arc_circle_case},
 {"gc_arc_ellipse", gb_gc_arc_ellipse_case},
 {"gc_arc_hyperbola", gb_gc_arc_hyperbola_case},
 {"gc_arc_parabola", gb_gc_arc_parabola_case},
 {"gc_arc_circle2d", gb_gc_arc_circle2d_case},
 {"gc_arc_ellipse2d", gb_gc_arc_ellipse2d_case},
 {"gc_arc_hyperbola2d", gb_gc_arc_hyperbola2d_case},
 {"gc_arc_parabola2d", gb_gc_arc_parabola2d_case},
 {"gc_circle", gb_gc_circle_case},
 {"gc_circle2d", gb_gc_circle2d_case},
 {"gc_ellipse", gb_gc_ellipse_case},
 {"gc_hyperbola", gb_gc_hyperbola_case},
 {"gc_ellipse2d", gb_gc_ellipse2d_case},
 {"gc_hyperbola2d", gb_gc_hyperbola2d_case},
 {"gc_parabola2d", gb_gc_parabola2d_case},
 {"gc_line", gb_gc_line_case},
 {"gc_line2d", gb_gc_line2d_case},
 {"gc_segment", gb_gc_segment_case},
 {"gc_segment2d", gb_gc_segment2d_case},
 {"gc_plane", gb_gc_plane_case},
 {"gc_conical", gb_gc_conical_case},
 {"gc_cylindrical", gb_gc_cylindrical_case},
 {"gc_trimmed_cone", gb_gc_trimmed_cone_case},
 {"gc_trimmed_cylinder", gb_gc_trimmed_cylinder_case},
 {"gc_mirror", gb_gc_mirror_case},
 {"gc_mirror2d", gb_gc_mirror2d_case},
 {"gc_rotation", gb_gc_rotation_case},
 {"gc_rotation2d", gb_gc_rotation2d_case},
 {"gc_scale", gb_gc_scale_case},
 {"gc_scale2d", gb_gc_scale2d_case},
 {"gc_translation", gb_gc_translation_case},
 {"gc_translation2d", gb_gc_translation2d_case},
 // IntAna2d (tkgeombase_probe_intana2d.cxx)
 {"ia2_lines", gb_ia2_lines_case},
 {"ia2_circles", gb_ia2_circles_case},
 {"ia2_line_circle", gb_ia2_line_circle_case},
 {"ia2_conic", gb_ia2_conic_case},
 {"ia2_conic_eval", gb_ia2_conic_eval_case},
 {"ia2_roots", gb_ia2_roots_case},
 {"ia2_outils", gb_ia2_outils_case},
 {"ia2_int_point", gb_ia2_int_point_case},
 // IntAna (tkgeombase_probe_intana.cxx)
 {"ia_quadric", gb_ia_quadric_case},
 {"ia_conic_quad", gb_ia_conic_quad_case},
 {"ia_conic_plane", gb_ia_conic_plane_case},
 {"ia_lin_torus", gb_ia_lin_torus_case},
 {"ia_qqgeo", gb_ia_qqgeo_case},
 {"ia_qq", gb_ia_qq_case},
 {"ia_curve", gb_ia_curve_case},
 // LProp and GeomLProp (tkgeombase_probe_lprop.cxx)
 {"lprop_cl2d", gb_lprop_cl2d_case},
 {"lprop_cl3d", gb_lprop_cl3d_case},
 {"lprop_cl3d_adaptor", gb_lprop_cl3d_adaptor_case},
 {"lprop_sl", gb_lprop_sl_case},
 {"lprop_curinf2d", gb_lprop_curinf2d_case},
 {"lprop_curandinf", gb_lprop_curandinf_case},
 {"lprop_continuity", gb_lprop_continuity_case},
 // GProp (tkgeombase_probe_gprop.cxx)
 {"gprop_points", gb_gprop_points_case},
 {"gprop_pequation", gb_gprop_pequation_case},
 {"gprop_elementary", gb_gprop_elementary_case},
 {"gprop_compose", gb_gprop_compose_case},
 {"gprop_hoperator", gb_gprop_hoperator_case},
 // Hermit (tkgeombase_probe_hermit.cxx)
 {"hermit", gb_hermit_case},
 // ExtremaPC2d, GCPnts_TangentialDeflection (tkgeombase_probe_extrema_pc2d.cxx)
 {"pc2d_curve", gb_pc2d_curve_case},
 {"pc2d_grid", gb_pc2d_grid_case},
 {"pc2d_uniform", gb_pc2d_uniform_case},
 {"pc2d_distance", gb_pc2d_distance_case},
 {"pc2d_endpoints", gb_pc2d_endpoints_case},
 {"gcpnts_td2d", gb_gcpnts_td2d_case},
 // ExtremaPC (tkgeombase_probe_extrema_pc.cxx)
 {"pc_curve", gb_pc_curve_case},
 {"pc_grid", gb_pc_grid_case},
 {"pc_distance", gb_pc_distance_case},
 {"pc_endpoints", gb_pc_endpoints_case},
 {"pc_project", gb_pc_project_case},
 {"gcpnts_td3d", gb_gcpnts_td3d_case},
 // GeomTools (tkgeombase_probe_geomtools*.cxx)
 {"gt_curve", gb_gt_curve_case},
 {"gt_curve2d", gb_gt_curve2d_case},
 {"gt_surface", gb_gt_surface_case},
 {"gt_read", gb_gt_read_case},
 {"gt_get_real", gb_gt_get_real_case},
 {"gt_set", gb_gt_set_case},
 {"gt_set_null", gb_gt_set_null_case},
 {"gt_set_read", gb_gt_set_read_case},
 {"gt_progress", gb_gt_progress_case},
 {"gt_debug_dump", gb_gt_debug_dump_case},
 {"gt_handler", gb_gt_handler_case},
 // FEmTool (tkgeombase_probe_femtool.cxx, tkgeombase_probe_femtool_curve.cxx)
 {"fem_elements", gb_fem_elements_case},
 {"fem_criterion", gb_fem_criterion_case},
 {"fem_profile", gb_fem_profile_case},
 {"fem_curve", gb_fem_curve_case},
 {"fem_assembly", gb_fem_assembly_case},
 // AppParCurves (tkgeombase_probe_appparcurves.cxx, tkgeombase_probe_appparcurves_fit.cxx)
 {"apc_bernstein", gb_apc_bernstein_case},
 {"apc_spline", gb_apc_spline_case},
 {"apc_multipoint", gb_apc_multipoint_case},
 {"apc_multicurve", gb_apc_multicurve_case},
 {"apc_ls", gb_apc_ls_case},
 {"apc_resol", gb_apc_resol_case},
 {"apc_function", gb_apc_function_case},
 {"apc_bspfunction", gb_apc_bspfunction_case},
 {"apc_gradient", gb_apc_gradient_case},
 {"apc_bspgradient", gb_apc_bspgradient_case},
 // GeomBndLib and BndLib (tkgeombase_probe_bndlib.cxx)
 {"bnd_curve3d", gb_bnd_curve3d_case},
 {"bnd_curve2d", gb_bnd_curve2d_case},
 {"bnd_surface", gb_bnd_surface_case},
 {"bnd_gp", gb_bnd_gp_case},
 {"bnd_helpers", gb_bnd_helpers_case},
 // CPnts and GCPnts (tkgeombase_probe_gcpnts.cxx)
 {"gcp_length", gb_gcp_length_case},
 {"cpnts_abscissa", gb_cpnts_abscissa_case},
 {"cpnts_udefl", gb_cpnts_udefl_case},
 {"gcp_abscissa", gb_gcp_abscissa_case},
 {"gcp_uabs", gb_gcp_uabs_case},
 {"gcp_quabs", gb_gcp_quabs_case},
 {"gcp_udefl", gb_gcp_udefl_case},
 {"gcp_qudefl", gb_gcp_qudefl_case},
 {"gcp_tdefl", gb_gcp_tdefl_case},
 {"gcp_arc_step", gb_gcp_arc_step_case},
 {"gcp_dist", gb_gcp_dist_case},
 // AppCont (tkgeombase_probe_appcont.cxx)
 {"acc_table", gb_acc_table_case},
 {"acc_offsets", gb_acc_offsets_case},
 {"acc_matrix", gb_acc_matrix_case},
 {"acc_ls", gb_acc_ls_case},
 // AdvApp2Var_MathBase (tkgeombase_probe_advapp2var_math.cxx, _math2.cxx)
 {"aa_norms", gb_aa_norms_case},
 {"aa_mmwprcs", gb_aa_mmwprcs_case},
 {"aa_mmapcmp", gb_aa_mmapcmp_case},
 {"aa_mmdrc11", gb_aa_mmdrc11_case},
 {"aa_mmfmca8", gb_aa_mmfmca8_case},
 {"aa_mmfmca9", gb_aa_mmfmca9_case},
 {"aa_mmfmcb5", gb_aa_mmfmcb5_case},
 {"aa_mmfmtb1", gb_aa_mmfmtb1_case},
 {"aa_mmfmcar", gb_aa_mmfmcar_case},
 {"aa_mmcglc1", gb_aa_mmcglc1_case},
 {"aa_mmbulld", gb_aa_mmbulld_case},
 {"aa_mvsheld", gb_aa_mvsheld_case},
 {"aa_mmcdriv", gb_aa_mmcdriv_case},
 {"aa_mmdrvck", gb_aa_mmdrvck_case},
 {"aa_mmcvctx", gb_aa_mmcvctx_case},
 {"aa_mdsptpt", gb_aa_mdsptpt_case},
 {"aa_mmaperx", gb_aa_mmaperx_case},
 {"aa_mmtrpjj", gb_aa_mmtrpjj_case},
 {"aa_mmgaus1", gb_aa_mmgaus1_case},
 {"aa_mmhjcan", gb_aa_mmhjcan_case},
 {"aa_mminltt", gb_aa_mminltt_case},
 {"aa_mmjacan", gb_aa_mmjacan_case},
 {"aa_mmjaccv", gb_aa_mmjaccv_case},
 {"aa_mmpobas", gb_aa_mmpobas_case},
 {"aa_mmpocrb", gb_aa_mmpocrb_case},
 {"aa_mmmpocur", gb_aa_mmmpocur_case},
 {"aa_mmposui", gb_aa_mmposui_case},
 {"aa_mmresol", gb_aa_mmresol_case},
 {"aa_mmrtptt", gb_aa_mmrtptt_case},
 {"aa_mmsrre2", gb_aa_mmsrre2_case},
 {"aa_mmunivt", gb_aa_mmunivt_case},
 {"aa_mmvncol", gb_aa_mmvncol_case},
 {"aa_mmarcin", gb_aa_mmarcin_case},
 {"aa_mmcvinv", gb_aa_mmcvinv_case},
 {"aa_mmmrslwd", gb_aa_mmmrslwd_case},
 // AdvApp2Var_ApproxF2var (tkgeombase_probe_advapp2var_f2var.cxx, _f2var2.cxx)
 {"aa_mma1her", gb_aa_mma1her_case},
 {"aa_mma2roo", gb_aa_mma2roo_case},
 {"aa_mma2jmx", gb_aa_mma2jmx_case},
 {"aa_mmapptt", gb_aa_mmapptt_case},
 {"aa_mma2ac1", gb_aa_mma2ac1_case},
 {"aa_mma2ac2", gb_aa_mma2ac2_case},
 {"aa_mma2ac3", gb_aa_mma2ac3_case},
 {"aa_mma2can", gb_aa_mma2can_case},
 {"aa_mma2cdi", gb_aa_mma2cdi_case},
 {"aa_mma2ds1", gb_aa_mma2ds1_case},
 {"aa_mma2ce1", gb_aa_mma2ce1_case},
 {"aa_mma2fnc", gb_aa_mma2fnc_case},
 {"aa_mma2fx6", gb_aa_mma2fx6_case},
 // AdvApp2Var classes (tkgeombase_probe_advapp2var_class.cxx)
 {"aa_context", gb_aa_context_case},
 {"aa_grid", gb_aa_grid_case},
 {"aa_patch", gb_aa_patch_case},
 {"aa_approx", gb_aa_approx_case},
 // Geom2dConvert (tkgeombase_probe_geom2dconvert.cxx)
 {"g2c_to_bspline", gb_g2c_to_bspline_case},
 {"g2c_split", gb_g2c_split_case},
 {"g2c_knot_split", gb_g2c_knot_split_case},
 {"g2c_bezier", gb_g2c_bezier_case},
 {"g2c_comp", gb_g2c_comp_case},
 {"g2c_concat", gb_g2c_concat_case},
 {"g2c_c0c1", gb_g2c_c0c1_case},
 {"g2c_approx", gb_g2c_approx_case},
 {"g2c_arcs", gb_g2c_arcs_case},
 {"g2c_ppoint", gb_g2c_ppoint_case},
 // Extrema elementary closed forms (tkgeombase_probe_extrema_el.cxx)
 {"ex_pon", gb_ex_pon_case},
 {"ex_pelc", gb_ex_pelc_case},
 {"ex_pelc2d", gb_ex_pelc2d_case},
 {"ex_elc", gb_ex_elc_case},
 {"ex_elc2d", gb_ex_elc2d_case},
 {"ex_pels", gb_ex_pels_case},
 {"ex_elcs", gb_ex_elcs_case},
 {"ex_elss", gb_ex_elss_case},
 // Extrema point-curve classes (tkgeombase_probe_extrema_pc_classic.cxx)
 {"ex_pc", gb_ex_pc_case},
 {"ex_pc2d", gb_ex_pc2d_case},
 {"ex_locpc", gb_ex_locpc_case},
 {"ex_locpc_owned", gb_ex_locpc_case},
 {"ex_locpc2d_owned", gb_ex_locpc2d_case},
 {"ex_locpc2d", gb_ex_locpc2d_case},
 {"ex_genpc", gb_ex_genpc_case},
 {"ex_genpc2d", gb_ex_genpc2d_case},
 {"ex_pcf", gb_ex_pcf_case},
 {"ex_pcf2d", gb_ex_pcf2d_case},
 {"ex_cloc", gb_ex_cloc_case},
 {"ex_cloc2d", gb_ex_cloc2d_case},
 {"ex_ctool", gb_ex_ctool_case},
 {"ex_ctool2d", gb_ex_ctool2d_case},
 // Extrema curve-curve classes (tkgeombase_probe_extrema_cc.cxx)
 {"ex_cc", gb_ex_cc_case},
 {"ex_cc2d", gb_ex_cc2d_case},
 {"ex_ecc", gb_ex_ecc_case},
 {"ex_ecc2d", gb_ex_ecc2d_case},
 {"ex_locecc", gb_ex_locecc_case},
 {"ex_fcc", gb_ex_fcc_case},
 {"ex_gofcc", gb_ex_gofcc_case},
 {"ex_gofcc2d", gb_ex_gofcc2d_case},
 // Extrema point-surface classes (tkgeombase_probe_extrema_ps.cxx)
 {"ex_fpsn", gb_ex_fpsn_case},
 {"ex_fpsd", gb_ex_fpsd_case},
 {"ex_glps", gb_ex_glps_case},
 {"ex_genps", gb_ex_genps_case},
 {"ex_ps", gb_ex_ps_case},
 {"ex_pexts", gb_ex_pexts_case},
 {"ex_prevs", gb_ex_prevs_case},
 // Extrema curve-surface classes (tkgeombase_probe_extrema_cs.cxx)
 {"ex_fecs", gb_ex_fecs_case},
 {"ex_gofcs", gb_ex_gofcs_case},
 {"ex_gofconics", gb_ex_gofconics_case},
 {"ex_gofcquad", gb_ex_gofcquad_case},
 {"ex_genecs", gb_ex_genecs_case},
 {"ex_cs", gb_ex_cs_case},
 // Extrema surface-surface classes (tkgeombase_probe_extrema_ss.cxx)
 {"ex_fess", gb_ex_fess_case},
 {"ex_genss", gb_ex_genss_case},
 {"ex_ss", gb_ex_ss_case},
 // GeomConvert conversions (tkgeombase_probe_geomconvert.cxx)
 {"gcv_to_bspline", gb_gcv_to_bspline_case},
 {"gcv_surf_to_bspline", gb_gcv_surf_to_bspline_case},
 {"gcv_split", gb_gcv_split_case},
 {"gcv_split_surface", gb_gcv_split_surface_case},
 {"gcv_knot_split", gb_gcv_knot_split_case},
 {"gcv_knot_split_surface", gb_gcv_knot_split_surface_case},
 {"gcv_bezier", gb_gcv_bezier_case},
 {"gcv_bezier_surface", gb_gcv_bezier_surface_case},
 {"gcv_comp", gb_gcv_comp_case},
 {"gcv_comp_bezier", gb_gcv_comp_bezier_case},
 {"gcv_concat", gb_gcv_concat_case},
 {"gcv_c0c1", gb_gcv_c0c1_case},
 {"gcv_approx_curve", gb_gcv_approx_curve_case},
 {"gcv_approx_surface", gb_gcv_approx_surface_case},
 // GeomConvert recognizers and units (tkgeombase_probe_geomconvert_ana.cxx)
 {"gcv_ana_curve", gb_gcv_ana_curve_case},
 {"gcv_ana_empty", gb_gcv_ana_empty_case},
 {"gcv_is_linear", gb_gcv_is_linear_case},
 {"gcv_get_line", gb_gcv_get_line_case},
 {"gcv_get_circle", gb_gcv_get_circle_case},
 {"gcv_ana_surf", gb_gcv_ana_surf_case},
 {"gcv_ana_surf_empty", gb_gcv_ana_surf_empty_case},
 {"gcv_same_surf", gb_gcv_same_surf_case},
 {"gcv_ls_dist", gb_gcv_ls_dist_case},
 {"gcv_units", gb_gcv_units_case},
 // GeomLib static members (tkgeombase_probe_geomlib.cxx)
 {"gl_to3d", gb_gl_to3d_case},
 {"gl_gtransform", gb_gl_gtransform_case},
 {"gl_same_range", gb_gl_same_range_case},
 {"gl_build_curve3d", gb_gl_build_curve3d_case},
 {"gl_adjust", gb_gl_adjust_case},
 {"gl_extend_curve", gb_gl_extend_curve_case},
 {"gl_extend_surf", gb_gl_extend_surf_case},
 {"gl_inertia", gb_gl_inertia_case},
 {"gl_arrays", gb_gl_arrays_case},
 {"gl_eval_max", gb_gl_eval_max_case},
 {"gl_cancel_denom", gb_gl_cancel_denom_case},
 {"gl_norm_estim", gb_gl_norm_estim_case},
 {"gl_is_closed", gb_gl_is_closed_case},
 {"gl_iso_line", gb_gl_iso_line_case},
 // GeomLib classes (tkgeombase_probe_geomlib_classes.cxx)
 {"gl_check_bspline", gb_gl_check_bspline_case},
 {"gl_cos", gb_gl_cos_case},
 {"gl_denom", gb_gl_denom_case},
 {"gl_interpolate", gb_gl_interpolate_case},
 {"gl_is_planar", gb_gl_is_planar_case},
 {"gl_log_sample", gb_gl_log_sample_case},
 {"gl_poly_func", gb_gl_poly_func_case},
 {"gl_tool", gb_gl_tool_case},
 {"gl_deviation", gb_gl_deviation_case},
 {"gl_make_curve", gb_gl_make_curve_case},
 // Approx drivers (tkgeombase_probe_approx_line.cxx, tkgeombase_probe_approx.cxx)
 {"ax_compute", gb_ax_compute_case},
 {"ax_bspl", gb_ax_bspl_case},
 {"ax_fit", gb_ax_fit_case},
 {"ax_mcurves", gb_ax_mcurves_case},
 // Approx adaptor classes and sweeps (tkgeombase_probe_approx_adaptors.cxx, _sweep.cxx)
 {"axa_curve3d", gb_axa_curve3d_case},
 {"axa_curve2d", gb_axa_curve2d_case},
 {"axa_cos", gb_axa_cos_case},
 {"axa_curvlin", gb_axa_curvlin_case},
 {"axa_curvilinear", gb_axa_curvilinear_case},
 {"axa_same", gb_axa_same_case},
 {"axs_sweep", gb_axs_sweep_case},
 // AppDef (tkgeombase_probe_appdef.cxx, tkgeombase_probe_appdef_variational.cxx)
 {"ad_mpc", gb_ad_mpc_case},
 {"ad_line", gb_ad_line_case},
 {"ad_compute", gb_ad_compute_case},
 {"ad_criteria", gb_ad_criteria_case},
 {"ad_var", gb_ad_var_case},
 // ProjLib (tkgeombase_probe_projlib.cxx, tkgeombase_probe_projlib_curves.cxx)
 {"pl_elem", gb_pl_elem_case},
 {"pl_static", gb_pl_static_case},
 {"pl_prjfunc", gb_pl_prjfunc_case},
 {"pl_prjresolve", gb_pl_prjresolve_case},
 {"pl_on_plane", gb_pl_on_plane_case},
 {"pl_approx", gb_pl_approx_case},
 {"pl_polar", gb_pl_polar_case},
 {"pl_projected", gb_pl_projected_case},
 {"pl_comp", gb_pl_comp_case},
 // GeomProjLib (tkgeombase_probe_geomprojlib.cxx)
 {"gpl_curve2d", gb_gpl_curve2d_case},
 {"gpl_project", gb_gpl_project_case},
 {"gpl_on_plane", gb_gpl_on_plane_case},
};
int main() {
 std::string line;
 while(std::getline(std::cin,line)) {
  std::istringstream input(line);std::string op;input>>op;
  if(op.empty())continue;
  auto found=probe_cases.find(op);
  if(found==probe_cases.end()) {std::cerr<<"Unknown probe operation: "<<op<<std::endl;return 2;}
  found->second(input);
 }
}
