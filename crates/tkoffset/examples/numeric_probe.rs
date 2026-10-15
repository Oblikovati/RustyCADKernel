//! Exact TKOffset numeric oracle counterpart of tools/oracle/tkoffset_probe.cxx: reads
//! `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod of_probe;
use of_probe::{make_offset, make_pipe_shell, thru_sections, wrappers};

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
