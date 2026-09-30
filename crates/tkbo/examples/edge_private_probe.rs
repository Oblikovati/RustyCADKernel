//! Isolated EdgeEdge helper oracle; no complete public API is implied.
pub use rustycad_tkbo::int_tools;
#[path = "../src/int_tools/edge_edge/distance.rs"]
mod distance;
#[path = "../src/int_tools/edge_edge/distance_search.rs"]
mod distance_search;
#[path = "../src/int_tools/edge_edge/best_solution.rs"]
mod edge_best;
#[path = "../src/int_tools/edge_edge/coincidence.rs"]
mod edge_coincidence;
#[path = "../src/int_tools/edge_edge/intersection.rs"]
mod edge_intersection;
#[path = "../src/int_tools/edge_edge/line_line.rs"]
mod edge_lines;
#[path = "../src/int_tools/edge_edge/math.rs"]
mod edge_math;
#[path = "../src/int_tools/edge_edge/parameters.rs"]
mod edge_parameters;
mod edge_private;
#[path = "../src/int_tools/edge_edge/resolution.rs"]
mod edge_resolution;
#[path = "bo_probe/tools_geometry.rs"]
mod tools_geometry;
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.expect("edge helper request");
        let mut fields = line.split_whitespace();
        let Some(operation) = fields.next() else {
            continue;
        };
        let values: Vec<f64> = fields
            .map(|v| v.parse().expect("edge helper numeric field"))
            .collect();
        edge_private::run(operation, &values);
        println!();
    }
}
