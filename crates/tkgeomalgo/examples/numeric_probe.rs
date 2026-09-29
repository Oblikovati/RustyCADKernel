//! Exact TKGeomAlgo numeric oracle counterpart of tools/oracle/tkgeomalgo_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod ga_probe;
use ga_probe::{gcc_ana, gcc_ana_lines, gcc_ent, gcc_int, hatch, intf};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkgeomalgo_probe.cxx (`probe_cases`); each
/// package adds its operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // Intf (ga_probe/intf.rs)
    ("intf_triangle", intf::triangle_case),
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
