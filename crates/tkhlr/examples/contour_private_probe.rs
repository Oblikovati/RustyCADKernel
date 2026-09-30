//! Isolated oracle for private contour helpers; no public API completion is implied.
pub use rustycad_tkhlr::contap;
use std::io::{self, BufRead};
#[path = "../src/contap/contour/boundary_points.rs"]
mod boundary_points;
#[allow(dead_code, unused_imports)]
#[path = "../../tkgeomalgo/examples/ga_probe/common.rs"]
mod common;
#[allow(dead_code)]
#[path = "hl_probe/contana.rs"]
mod contana;
#[allow(dead_code)]
#[path = "hl_probe/contap_surface.rs"]
mod contap_surface;
mod contour_private;
#[path = "../src/contap/contour/line_constructor.rs"]
mod line_constructor;
#[path = "../src/contap/contour/math_helpers.rs"]
mod math_helpers;
#[allow(dead_code)]
#[path = "../../tkg2d/examples/g2_probe/print.rs"]
mod print;
#[path = "../src/contap/contour/restriction_internal.rs"]
mod restriction_internal;
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.expect("private contour probe input");
        let mut fields = line.split_whitespace();
        let Some(operation) = fields.next() else {
            continue;
        };
        let values: Vec<f64> = fields
            .map(|value| value.parse().expect("contour numeric field"))
            .collect();
        match operation {
            "contour_attachment" => contour_private::attachment_case(&values),
            "contour_internal" => contour_private::internal_case(&values),
            "contour_math" => contour_private::math_case(&values),
            "contour_sections" => contour_private::sections_case(&values),
            _ => {
                eprintln!("Unknown probe operation: {operation}");
                std::process::exit(2);
            }
        }
    }
}
