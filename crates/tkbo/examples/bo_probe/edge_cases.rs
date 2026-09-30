//! Reuse the public EdgeEdge histories in the regular TKBO oracle gate.
use super::{integer, real};
use rustycad_tkernel::standard::Failure;
#[path = "../edge_private/fixtures.rs"]
pub(super) mod fixtures;
#[path = "../edge_private/full.rs"]
mod full;
fn attempt(result: Result<(), Failure>) {
    if let Err(error) = result {
        print!("{} ", error.exception_type());
    }
}
pub(super) fn case(values: &[f64]) {
    attempt(full::case(values));
}
