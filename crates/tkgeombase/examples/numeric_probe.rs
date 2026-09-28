//! Exact TKGeomBase numeric oracle counterpart of tools/oracle/tkgeombase_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod gb_probe;
use gb_probe::{
    adv_class, adv_f2var, adv_f2var2, adv_math, adv_math2, gc, gce, gprop, hermit, int_ana,
    int_ana2d, lprop, pc2d, pc3d,
};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkgeombase_probe.cxx (`probe_cases`); each
/// package adds its operations, in the same order as the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // gce (gb_probe/gce.rs)
    ("gce_circ", gce::circ_case),
    ("gce_circ2d", gce::circ2d_case),
    ("gce_cone", gce::cone_case),
    ("gce_cylinder", gce::cylinder_case),
    ("gce_dir", gce::dir_case),
    ("gce_dir2d", gce::dir2d_case),
    ("gce_elips", gce::elips_case),
    ("gce_elips2d", gce::elips2d_case),
    ("gce_hypr", gce::hypr_case),
    ("gce_hypr2d", gce::hypr2d_case),
    ("gce_lin", gce::lin_case),
    ("gce_lin2d", gce::lin2d_case),
    ("gce_parab", gce::parab_case),
    ("gce_parab2d", gce::parab2d_case),
    ("gce_pln", gce::pln_case),
    ("gce_mirror", gce::mirror_case),
    ("gce_mirror2d", gce::mirror2d_case),
    ("gce_rotation", gce::rotation_case),
    ("gce_rotation2d", gce::rotation2d_case),
    ("gce_scale", gce::scale_case),
    ("gce_scale2d", gce::scale2d_case),
    ("gce_translation", gce::translation_case),
    ("gce_translation2d", gce::translation2d_case),
    ("gce_status", gce::status_case),
    // GC (gb_probe/gc)
    ("gc_arc_circle", gc::arc_circle_case),
    ("gc_arc_ellipse", gc::arc_ellipse_case),
    ("gc_arc_hyperbola", gc::arc_hyperbola_case),
    ("gc_arc_parabola", gc::arc_parabola_case),
    ("gc_arc_circle2d", gc::arc_circle2d_case),
    ("gc_arc_ellipse2d", gc::arc_ellipse2d_case),
    ("gc_arc_hyperbola2d", gc::arc_hyperbola2d_case),
    ("gc_arc_parabola2d", gc::arc_parabola2d_case),
    ("gc_circle", gc::circle_case),
    ("gc_circle2d", gc::circle2d_case),
    ("gc_ellipse", gc::ellipse_case),
    ("gc_hyperbola", gc::hyperbola_case),
    ("gc_ellipse2d", gc::ellipse2d_case),
    ("gc_hyperbola2d", gc::hyperbola2d_case),
    ("gc_parabola2d", gc::parabola2d_case),
    ("gc_line", gc::line_case),
    ("gc_line2d", gc::line2d_case),
    ("gc_segment", gc::segment_case),
    ("gc_segment2d", gc::segment2d_case),
    ("gc_plane", gc::plane_case),
    ("gc_conical", gc::conical_case),
    ("gc_cylindrical", gc::cylindrical_case),
    ("gc_trimmed_cone", gc::trimmed_cone_case),
    ("gc_trimmed_cylinder", gc::trimmed_cylinder_case),
    ("gc_mirror", gc::mirror_case),
    ("gc_mirror2d", gc::mirror2d_case),
    ("gc_rotation", gc::rotation_case),
    ("gc_rotation2d", gc::rotation2d_case),
    ("gc_scale", gc::scale_case),
    ("gc_scale2d", gc::scale2d_case),
    ("gc_translation", gc::translation_case),
    ("gc_translation2d", gc::translation2d_case),
    // IntAna2d (gb_probe/int_ana2d.rs)
    ("ia2_lines", int_ana2d::ia2_lines_case),
    ("ia2_circles", int_ana2d::ia2_circles_case),
    ("ia2_line_circle", int_ana2d::ia2_line_circle_case),
    ("ia2_conic", int_ana2d::ia2_conic_case),
    ("ia2_conic_eval", int_ana2d::ia2_conic_eval_case),
    ("ia2_roots", int_ana2d::ia2_roots_case),
    ("ia2_outils", int_ana2d::ia2_outils_case),
    ("ia2_int_point", int_ana2d::ia2_int_point_case),
    // IntAna (gb_probe/int_ana)
    ("ia_quadric", int_ana::ia_quadric_case),
    ("ia_conic_quad", int_ana::ia_conic_quad_case),
    ("ia_conic_plane", int_ana::ia_conic_plane_case),
    ("ia_lin_torus", int_ana::ia_lin_torus_case),
    ("ia_qqgeo", int_ana::ia_qqgeo_case),
    ("ia_qq", int_ana::ia_qq_case),
    ("ia_curve", int_ana::ia_curve_case),
    // LProp and GeomLProp (gb_probe/lprop)
    ("lprop_cl2d", lprop::cl2d_case),
    ("lprop_cl3d", lprop::cl3d_case),
    ("lprop_cl3d_adaptor", lprop::cl3d_adaptor_case),
    ("lprop_sl", lprop::sl_case),
    ("lprop_curinf2d", lprop::curinf2d_case),
    ("lprop_curandinf", lprop::curandinf_case),
    ("lprop_continuity", lprop::continuity_case),
    // GProp (gb_probe/gprop.rs)
    ("gprop_points", gprop::points_case),
    ("gprop_pequation", gprop::pequation_case),
    ("gprop_elementary", gprop::elementary_case),
    ("gprop_compose", gprop::compose_case),
    ("gprop_hoperator", gprop::hoperator_case),
    // Hermit (gb_probe/hermit.rs)
    ("hermit", hermit::hermit_case),
    // ExtremaPC2d, GCPnts_TangentialDeflection (gb_probe/pc2d.rs)
    ("pc2d_curve", pc2d::curve_case),
    ("pc2d_grid", pc2d::grid_case),
    ("pc2d_uniform", pc2d::uniform_case),
    ("pc2d_distance", pc2d::distance_case),
    ("pc2d_endpoints", pc2d::endpoints_case),
    ("gcpnts_td2d", pc2d::td2d_case),
    // ExtremaPC (gb_probe/pc3d.rs)
    ("pc_curve", pc3d::curve_case),
    ("pc_grid", pc3d::grid_case),
    ("pc_distance", pc3d::distance_case),
    ("pc_endpoints", pc3d::endpoints_case),
    ("pc_project", pc3d::project_case),
    ("gcpnts_td3d", pc3d::td3d_case),
    // AdvApp2Var_MathBase (gb_probe/adv_math.rs, adv_math2.rs)
    ("aa_norms", adv_math::norms_case),
    ("aa_mmwprcs", adv_math::mmwprcs_case),
    ("aa_mmapcmp", adv_math::mmapcmp_case),
    ("aa_mmdrc11", adv_math::mmdrc11_case),
    ("aa_mmfmca8", adv_math::mmfmca8_case),
    ("aa_mmfmca9", adv_math::mmfmca9_case),
    ("aa_mmfmcb5", adv_math::mmfmcb5_case),
    ("aa_mmfmtb1", adv_math::mmfmtb1_case),
    ("aa_mmfmcar", adv_math::mmfmcar_case),
    ("aa_mmcglc1", adv_math::mmcglc1_case),
    ("aa_mmbulld", adv_math::mmbulld_case),
    ("aa_mvsheld", adv_math::mvsheld_case),
    ("aa_mmcdriv", adv_math::mmcdriv_case),
    ("aa_mmdrvck", adv_math::mmdrvck_case),
    ("aa_mmcvctx", adv_math::mmcvctx_case),
    ("aa_mdsptpt", adv_math::mdsptpt_case),
    ("aa_mmaperx", adv_math::mmaperx_case),
    ("aa_mmtrpjj", adv_math::mmtrpjj_case),
    ("aa_mmgaus1", adv_math::mmgaus1_case),
    ("aa_mmhjcan", adv_math2::mmhjcan_case),
    ("aa_mminltt", adv_math2::mminltt_case),
    ("aa_mmjacan", adv_math2::mmjacan_case),
    ("aa_mmjaccv", adv_math2::mmjaccv_case),
    ("aa_mmpobas", adv_math2::mmpobas_case),
    ("aa_mmpocrb", adv_math2::mmpocrb_case),
    ("aa_mmmpocur", adv_math2::mmmpocur_case),
    ("aa_mmposui", adv_math2::mmposui_case),
    ("aa_mmresol", adv_math2::mmresol_case),
    ("aa_mmrtptt", adv_math2::mmrtptt_case),
    ("aa_mmsrre2", adv_math2::mmsrre2_case),
    ("aa_mmunivt", adv_math2::mmunivt_case),
    ("aa_mmvncol", adv_math2::mmvncol_case),
    ("aa_mmarcin", adv_math2::mmarcin_case),
    ("aa_mmcvinv", adv_math2::mmcvinv_case),
    ("aa_mmmrslwd", adv_math2::mmmrslwd_case),
    // AdvApp2Var_ApproxF2var (gb_probe/adv_f2var.rs, adv_f2var2.rs)
    ("aa_mma1her", adv_f2var::mma1her_case),
    ("aa_mma2roo", adv_f2var::mma2roo_case),
    ("aa_mma2jmx", adv_f2var::mma2jmx_case),
    ("aa_mmapptt", adv_f2var::mmapptt_case),
    ("aa_mma2ac1", adv_f2var::mma2ac1_case),
    ("aa_mma2ac2", adv_f2var::mma2ac2_case),
    ("aa_mma2ac3", adv_f2var::mma2ac3_case),
    ("aa_mma2can", adv_f2var::mma2can_case),
    ("aa_mma2cdi", adv_f2var::mma2cdi_case),
    ("aa_mma2ds1", adv_f2var::mma2ds1_case),
    ("aa_mma2ce1", adv_f2var2::mma2ce1_case),
    ("aa_mma2fnc", adv_f2var2::mma2fnc_case),
    ("aa_mma2fx6", adv_f2var2::mma2fx6_case),
    // AdvApp2Var classes (gb_probe/adv_class.rs)
    ("aa_context", adv_class::context_case),
    ("aa_grid", adv_class::grid_case),
    ("aa_patch", adv_class::patch_case),
    ("aa_approx", adv_class::approx_case),
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
