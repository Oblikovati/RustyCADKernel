//! Exact TKGeomBase numeric oracle counterpart of tools/oracle/tkgeombase_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod gb_probe;
use gb_probe::{gc, gce, gprop, hermit, lprop};

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
