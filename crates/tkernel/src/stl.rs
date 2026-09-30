//! Exact ports of the libstdc++ 13 algorithms whose unstable element order is
//! observable in OCCT results on the Linux/GCC oracle profile.
//!
//! `std::sort` and `std::partition` do not preserve the order of equivalent
//! elements. OCCT code such as `BVH_QuickSorter` and `BVH_RadixSorter` applies the
//! resulting permutation in place, so the order of ties is part of the observable
//! output. Rust's own sorts use other algorithms and resolve ties differently.
//!
//! As in C++, the comparator must be a strict weak ordering. Out-of-contract
//! comparators, which are undefined behavior upstream, panic on a bounds check here.
//!
//! The `<random>` engine and distribution OCCT draws from are here too: `std::mt19937`
//! (fixed by the standard) and libstdc++'s `std::uniform_real_distribution<double>`
//! (its `generate_canonical` is library-specific).
mod heap;
mod mt19937;
mod partition;
mod sort;
mod uniform_real;
pub use mt19937::MersenneTwister;
pub use partition::partition;
pub use sort::sort_by;
pub use uniform_real::{UniformRealDistribution, generate_canonical};
