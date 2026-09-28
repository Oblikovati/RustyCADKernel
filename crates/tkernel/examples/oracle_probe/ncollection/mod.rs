//! NCollection differential operations, one module per container family.
//!
//! Each family answers the operations it owns with the text the native twin in
//! `tools/oracle/tkernel_ncollection_<family>_probe.cxx` prints, or `None` for
//! an operation it does not own.
use std::str::SplitWhitespace;

mod allocators;
mod arrays;
mod flat_maps;
mod hash_maps;
mod lists;
mod trees;
mod utilities;
mod vec_mat;

/// Dispatch `op` to the family that owns it; `None` when no family does.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    arrays::evaluate(op, input)
        .or_else(|| lists::evaluate(op, input))
        .or_else(|| hash_maps::evaluate(op, input))
        .or_else(|| flat_maps::evaluate(op, input))
        .or_else(|| vec_mat::evaluate(op, input))
        .or_else(|| allocators::evaluate(op, input))
        .or_else(|| trees::evaluate(op, input))
        .or_else(|| utilities::evaluate(op, input))
}
