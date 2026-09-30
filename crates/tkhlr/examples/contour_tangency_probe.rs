//! Isolated private ComputeTangency oracle; no Contap_Contour public API claims.
pub use rustycad_tkhlr::contap;
#[allow(dead_code, unused_imports)]
#[path = "../../tkgeomalgo/examples/ga_probe/common.rs"]
mod common;
#[allow(dead_code)]
#[path = "hl_probe/contana.rs"]
mod contana;
#[allow(dead_code)]
#[path = "hl_probe/contap_surface.rs"]
mod contap_surface;
mod contour_tangency;
#[allow(dead_code)]
#[path = "../../tkg2d/examples/g2_probe/print.rs"]
mod print;
#[path = "../src/contap/contour/tangency/mod.rs"]
mod tangency;
fn main() {
    use std::io::{self, BufRead};
    for line in io::stdin().lock().lines() {
        let line = line.expect("private tangency input");
        let mut fields = line.split_whitespace();
        let Some(operation) = fields.next() else {
            continue;
        };
        if operation != "tangency" {
            eprintln!("Unknown probe operation: {operation}");
            std::process::exit(2);
        }
        let values: Vec<f64> = fields
            .map(|field| field.parse().expect("numeric tangency field"))
            .collect();
        contour_tangency::case(&values);
    }
}
