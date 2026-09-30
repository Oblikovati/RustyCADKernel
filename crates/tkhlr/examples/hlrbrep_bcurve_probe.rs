//! Independent complete BCurveTool oracle.
#[allow(dead_code, unused_imports)]
#[path = "../../tkgeomalgo/examples/ga_probe/common.rs"]
mod common;
mod hlrbrep_bcurve;
#[allow(dead_code)]
#[path = "../../tkg2d/examples/g2_probe/print.rs"]
mod print;
fn main() {
    use std::io::{self, BufRead};
    for line in io::stdin().lock().lines() {
        let line = line.expect("BCurve oracle input");
        let mut fields = line.split_whitespace();
        let Some(operation) = fields.next() else {
            continue;
        };
        if operation != "bcurve" {
            eprintln!("Unknown probe operation: {operation}");
            std::process::exit(2);
        }
        let values: Vec<f64> = fields
            .map(|v| v.parse().expect("BCurve numeric input"))
            .collect();
        hlrbrep_bcurve::case(&values);
    }
}
