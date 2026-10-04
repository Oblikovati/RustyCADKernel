//! Observe private BeanFace helpers without advertising an unfinished public intersector.
pub use rustycad_tkbo::int_tools;
#[path = "../src/int_tools/bean_face/boxes.rs"]
mod bean_boxes;
#[path = "bo_probe/fclass_faces.rs"]
mod bean_faces;
#[path = "../src/int_tools/bean_face/fast_analytic.rs"]
mod bean_fast_analytic;
#[path = "../src/int_tools/bean_face/grid.rs"]
mod bean_grid;
#[path = "../src/int_tools/bean_face/line_plane.rs"]
mod bean_line_plane;
mod bean_private;
#[path = "../src/int_tools/bean_face/ranges.rs"]
mod bean_ranges;
#[path = "../src/int_tools/bean_face/empty_range.rs"]
mod empty_range;
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.expect("bean helper request");
        let mut fields = line.split_whitespace();
        let Some(operation) = fields.next() else {
            continue;
        };
        let values: Vec<f64> = fields
            .map(|value| value.parse().expect("bean numeric field"))
            .collect();
        bean_private::run(operation, &values);
        println!();
    }
}
