//! Exact TKOffset numeric oracle counterpart of tools/oracle/tkoffset_probe.cxx: reads
//! `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod of_probe;
use of_probe::{
    analyse, bitgte, draft_angle, inter3d, make_offset, make_pipe_shell, maker, middle_path,
    offset, thru_sections, tool, wrappers,
};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkoffset_probe.cxx (`probe_cases`); each package
/// adds its operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // BRepOffsetAPI (of_probe/make_offset.rs, of_probe/make_pipe_shell.rs,
    // of_probe/thru_sections.rs)
    ("oa_make_offset", make_offset::make_offset_case),
    ("oa_pipe_shell", make_pipe_shell::pipe_shell_case),
    ("oa_thru_sections", thru_sections::thru_sections_case),
    // The thin BRepOffsetAPI wrappers (of_probe/wrappers.rs)
    ("oa_pipe", wrappers::pipe_case),
    ("oa_evolved", wrappers::evolved_case),
    ("oa_draft", wrappers::draft_case),
    ("oa_filling", wrappers::filling_case),
    ("oa_normal_projection", wrappers::normal_projection_case),
    ("oa_contigous", wrappers::contigous_case),
    // BRepOffset_Tool (of_probe/tool.rs)
    ("ot_inter3d", tool::inter3d_case),
    ("ot_enlarge", tool::enlarge_case),
    ("ot_inter2d", tool::inter2d_case),
    ("ot_queries", tool::queries_case),
    ("ot_deboucle", tool::deboucle_case),
    // BRepOffset_Analyse (of_probe/analyse.rs)
    ("oan_analyse", analyse::analyse_case),
    // BRepOffset_Offset (of_probe/offset.rs)
    ("oo_faces", offset::faces_case),
    ("oo_pipes", offset::pipes_case),
    ("oo_tubes", offset::tubes_case),
    // BRepOffset_Inter3d (of_probe/inter3d.rs)
    ("oi_inter", inter3d::inter_case),
    // BRepOffset_MakeOffset (of_probe/maker.rs)
    ("om_offset", maker::offset_case),
    ("om_simple", maker::simple_case),
    ("om_face_offsets", maker::face_offsets_case),
    // BiTgte_Blend (of_probe/bitgte.rs)
    ("obt_blend", bitgte::blend_case),
    // BRepOffsetAPI_MiddlePath (of_probe/middle_path.rs)
    ("oa_middle", middle_path::middle_case),
    ("oa_middle_prim", middle_path::middle_prim_case),
    // BRepOffsetAPI_DraftAngle and Draft (of_probe/draft_angle.rs)
    ("oa_draft_angle", draft_angle::draft_angle_case),
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
