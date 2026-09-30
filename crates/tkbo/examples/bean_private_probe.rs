//! Observe private BeanFace helpers without advertising an unfinished public intersector.
pub use rustycad_tkbo::int_tools;
mod bean_private;
#[path = "../src/int_tools/bean_face/ranges.rs"]
mod bean_ranges;
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
