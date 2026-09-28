//! Exact Geom2d, Adaptor2d and Geom2dAdaptor (TKG2d) numeric oracle counterpart of tools/oracle/tkg2d_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod g2_probe;
use g2_probe::{adaptor_cases, bspl, curves, eval, simple};

fn main() {
    for line in io::stdin().lock().lines() {
        let line: String = line.expect("probe input line");
        let mut fields = line.split_whitespace();
        let Some(op) = fields.next() else { continue };
        let values: Vec<f64> = fields
            .map(|field| field.parse().expect("numeric probe field"))
            .collect();
        match op {
            "g2_point" => simple::point_case(&values),
            "g2_direction" => simple::direction_case(&values),
            "g2_vector" => simple::vector_case(&values),
            "g2_axis" => simple::axis_case(&values),
            "g2_trsf" => simple::trsf_case(&values),
            "g2_curve" => curves::curve_case(&values),
            "g2_transform" => curves::transform_case(&values),
            "g2_conic" => curves::conic_case(&values),
            "g2_trimmed" => curves::trimmed_case(&values),
            "g2_offset" => curves::offset_case(&values),
            "g2_offset_utils" => curves::offset_utils_case(&values),
            "g2_bezier" => bspl::bezier_case(&values),
            "g2_bspline" => bspl::bspline_case(&values),
            "g2_evalrep" => bspl::evalrep_case(&values),
            "a2_adaptor" => adaptor_cases::adaptor_case(&values),
            "a2_load" => adaptor_cases::load_case(&values),
            "a2_offset_load" => adaptor_cases::offset_load_case(&values),
            "a2_evalrep" => adaptor_cases::evalrep_case(&values),
            "g2_eval" => eval::eval_case(&values),
            "g2_hash" => eval::hash_case(&values),
            "g2_hash_frame" => eval::hash_frame_case(&values),
            _ => {
                eprintln!("Unknown probe operation: {op}");
                std::process::exit(2);
            }
        }
    }
}
