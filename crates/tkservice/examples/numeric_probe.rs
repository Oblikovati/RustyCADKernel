//! Exact TKService numeric oracle counterpart of tools/oracle/tkservice_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod sv_probe;
use sv_probe::{grid, hid, pixel};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkservice_probe.cxx (`probe_cases`); each
/// package adds its operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // Aspect grids (sv_probe/grid.rs)
    ("sv_rgrid", grid::rectangular_case),
    ("sv_cgrid", grid::circular_case),
    // WNT_HIDSpaceMouse (sv_probe/hid.rs)
    ("sv_hid", hid::hid_case),
    // Image_PixMap pixel conversions (sv_probe/pixel.rs)
    ("sv_pixel", pixel::pixel_case),
    ("sv_rawpixel", pixel::raw_pixel_case),
    ("sv_half", pixel::half_case),
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
