//! Bounding-box operations split by state, accumulation, and queries.
mod accumulation;
mod limits;
pub use limits::BoxLimits3;
mod lines;
mod queries;
mod segment;
mod state;
mod transforms;
// C++ std::min/max retain the first operand on ties and unordered comparisons.
fn native_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}
fn native_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}
